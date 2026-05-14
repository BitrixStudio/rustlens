use anyhow::{anyhow, Result};
use rustlens_core::db::{self, postgres::SqlExecResult};
use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

struct TestDb {
    _container: ContainerAsync<Postgres>,
    pool: PgPool,
    database_url: String,
}

async fn start_postgres() -> Result<TestDb> {
    let container = Postgres::default()
        .with_db_name("rustlens")
        .with_user("postgres")
        .with_password("postgres")
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(5432).await?;
    let database_url = format!("postgres://postgres:postgres@{host}:{port}/rustlens");
    let pool = rustlens_core::db::connect::connect(&database_url).await?;

    Ok(TestDb {
        _container: container,
        pool,
        database_url,
    })
}

async fn seed(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        create schema app;
        create table app.users (
            id int primary key,
            name text not null,
            active bool not null,
            score real not null
        );
        insert into app.users (id, name, active, score)
        values (1, 'Ada', true, 10.5), (2, 'Linus', false, 20.25);
        create table app.empty_table (
            id bigint primary key,
            payload jsonb
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::test]
#[ignore = "requires Docker/testcontainers"]
async fn loads_metadata_and_table_pages() -> Result<()> {
    let db = start_postgres().await?;
    seed(&db.pool).await?;

    let tables = db::postgres::load_tables(&db.pool, "app").await?;
    assert_eq!(tables, vec!["empty_table", "users"]);

    let columns = db::postgres::load_columns(&db.pool, "app").await?;
    assert_eq!(
        columns,
        vec![
            (
                "empty_table".to_string(),
                vec!["id".to_string(), "payload".to_string()]
            ),
            (
                "users".to_string(),
                vec![
                    "id".to_string(),
                    "name".to_string(),
                    "active".to_string(),
                    "score".to_string()
                ]
            )
        ]
    );

    let (columns, rows) = db::postgres::load_table_page(&db.pool, "app", "users", 0, 1).await?;
    assert_eq!(columns, vec!["id", "name", "active", "score"]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], vec!["1", "Ada", "true", "10.5"]);

    let (columns, rows) =
        db::postgres::load_table_page(&db.pool, "app", "empty_table", 0, 10).await?;
    assert_eq!(columns, vec!["id", "payload"]);
    assert!(rows.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "requires Docker/testcontainers"]
async fn executes_queries_and_commands() -> Result<()> {
    let db = start_postgres().await?;
    seed(&db.pool).await?;

    match db::postgres::execute_sql(&db.pool, "select id, name from app.users where id < 0").await?
    {
        SqlExecResult::Rows { columns, rows } => {
            assert_eq!(columns, vec!["id", "name"]);
            assert!(rows.is_empty());
        }
        SqlExecResult::Command { info } => return Err(anyhow!("expected rows, got {info}")),
    }

    match db::postgres::execute_sql(&db.pool, "update app.users set name = 'Grace' where id = 1")
        .await?
    {
        SqlExecResult::Command { info } => assert!(info.contains("1 rows affected")),
        SqlExecResult::Rows { .. } => return Err(anyhow!("expected command result")),
    }

    match db::postgres::execute_sql(
        &db.pool,
        "insert into app.users (id, name, active, score) values (3, 'Edsger', true, 30.0) returning id",
    )
    .await?
    {
        SqlExecResult::Rows { columns, rows } => {
            assert_eq!(columns, vec!["id"]);
            assert_eq!(rows, vec![vec!["3".to_string()]]);
        }
        SqlExecResult::Command { info } => return Err(anyhow!("expected rows, got {info}")),
    }

    Ok(())
}

#[tokio::test]
#[ignore = "requires Docker/testcontainers"]
async fn worker_reports_errors_and_continues() -> Result<()> {
    let db = start_postgres().await?;
    seed(&db.pool).await?;

    let (cmd_tx, cmd_rx) = mpsc::channel(16);
    let (evt_tx, mut evt_rx) = mpsc::channel(16);

    tokio::spawn(async move {
        let _ = db::worker::run(cmd_rx, evt_tx).await;
    });

    cmd_tx
        .send(db::DbCmd::LoadTables {
            schema: "app".to_string(),
        })
        .await?;
    assert!(matches!(
        recv_event(&mut evt_rx).await?,
        db::DbEvt::Error(_)
    ));

    cmd_tx
        .send(db::DbCmd::Connect {
            database_url: db.database_url.clone(),
        })
        .await?;
    recv_until(&mut evt_rx, |evt| matches!(evt, db::DbEvt::Connected)).await?;

    cmd_tx
        .send(db::DbCmd::ExecuteSql {
            request_id: None,
            sql: "select from definitely invalid".to_string(),
        })
        .await?;
    recv_until(&mut evt_rx, |evt| matches!(evt, db::DbEvt::Error(_))).await?;

    cmd_tx
        .send(db::DbCmd::LoadSqlMeta {
            schema: "app".to_string(),
        })
        .await?;
    let evt = recv_until(&mut evt_rx, |evt| {
        matches!(evt, db::DbEvt::SqlMetaLoaded { .. })
    })
    .await?;

    match evt {
        db::DbEvt::SqlMetaLoaded {
            tables, columns, ..
        } => {
            assert_eq!(tables, vec!["empty_table", "users"]);
            assert_eq!(columns.len(), 2);
        }
        other => return Err(anyhow!("unexpected event: {other:?}")),
    }

    Ok(())
}

async fn recv_event(rx: &mut mpsc::Receiver<db::DbEvt>) -> Result<db::DbEvt> {
    timeout(Duration::from_secs(10), rx.recv())
        .await?
        .ok_or_else(|| anyhow!("DB event channel closed"))
}

async fn recv_until(
    rx: &mut mpsc::Receiver<db::DbEvt>,
    mut matches: impl FnMut(&db::DbEvt) -> bool,
) -> Result<db::DbEvt> {
    loop {
        let evt = recv_event(rx).await?;
        if matches(&evt) {
            return Ok(evt);
        }
    }
}
