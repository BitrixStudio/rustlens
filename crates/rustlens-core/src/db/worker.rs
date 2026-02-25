use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use crate::db::{DbCmd, DbEvt};

pub async fn run(mut cmd_rx: mpsc::Receiver<DbCmd>, evt_tx: mpsc::Sender<DbEvt>) -> Result<()> {
    let mut pool: Option<PgPool> = None;

    while let Some(cmd) = cmd_rx.recv().await {
        #[cfg(debug_assertions)]
        eprintln!("[worker] cmd: {:?}", cmd);

        match cmd {
            DbCmd::Connect { database_url } => {
                let _ = evt_tx.send(DbEvt::Status("Connecting…".into())).await;

                let connect_fut = PgPoolOptions::new()
                    .max_connections(6)
                    .connect(&database_url);

                match timeout(Duration::from_secs(5), connect_fut).await {
                    Ok(Ok(p)) => {
                        pool = Some(p);
                        let _ = evt_tx.send(DbEvt::Status("Connected.".into())).await;
                    }
                    Ok(Err(e)) => {
                        pool = None;
                        let _ = evt_tx
                            .send(DbEvt::Error(format!("DB connect failed: {e}")))
                            .await;
                    }
                    Err(_) => {
                        pool = None;
                        let _ = evt_tx
                            .send(DbEvt::Error("DB connect timed out.".into()))
                            .await;
                    }
                }
            }

            DbCmd::LoadTables { schema } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };
                let tables = crate::db::postgres::load_tables(pool, &schema).await?;
                let _ = evt_tx.send(DbEvt::TablesLoaded { tables }).await;
            }

            DbCmd::LoadSqlMeta { schema } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                // If you implemented LoadSqlMeta event, call those.
                // Otherwise just load tables for now.
                let tables = crate::db::postgres::load_tables(pool, &schema).await?;
                let _ = evt_tx.send(DbEvt::TablesLoaded { tables }).await;
            }

            DbCmd::LoadTablePage {
                schema,
                table,
                page,
                page_size,
            } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                let (columns, rows) =
                    crate::db::postgres::load_table_page(pool, &schema, &table, page, page_size)
                        .await?;

                let _ = evt_tx
                    .send(DbEvt::QueryResult {
                        columns,
                        rows,
                        info: format!("Loaded page {}", page + 1),
                    })
                    .await;
            }

            DbCmd::ExecuteSql { sql } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                match crate::db::postgres::execute_sql(pool, &sql).await? {
                    crate::db::postgres::SqlExecResult::Rows { columns, rows } => {
                        let _ = evt_tx
                            .send(DbEvt::QueryResult {
                                columns,
                                rows,
                                info: "Query OK".into(),
                            })
                            .await;
                    }
                    crate::db::postgres::SqlExecResult::Command { info } => {
                        let _ = evt_tx.send(DbEvt::SqlExecuted { info }).await;
                    }
                }
            }
        }
    }

    Ok(())
}
