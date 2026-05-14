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
                        let _ = evt_tx.send(DbEvt::Connected).await;
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
                match crate::db::postgres::load_tables(pool, &schema).await {
                    Ok(tables) => {
                        let _ = evt_tx.send(DbEvt::TablesLoaded { tables }).await;
                    }
                    Err(e) => {
                        let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
                    }
                }
            }

            DbCmd::LoadSqlMeta { schema } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                load_sql_meta(pool, schema, &evt_tx).await;
            }

            DbCmd::LoadTablePage {
                request_id,
                schema,
                table,
                page,
                page_size,
            } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                match crate::db::postgres::load_table_page(pool, &schema, &table, page, page_size)
                    .await
                {
                    Ok((columns, rows)) => {
                        let _ = evt_tx
                            .send(DbEvt::QueryResult {
                                request_id,
                                columns,
                                rows,
                                info: format!("Loaded page {}", page + 1),
                            })
                            .await;
                    }
                    Err(e) => {
                        let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
                    }
                }
            }

            DbCmd::ExecuteSql { request_id, sql } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                match crate::db::postgres::execute_sql(pool, &sql).await {
                    Ok(crate::db::postgres::SqlExecResult::Rows { columns, rows }) => {
                        let _ = evt_tx
                            .send(DbEvt::QueryResult {
                                request_id,
                                columns,
                                rows,
                                info: "Query OK".into(),
                            })
                            .await;
                    }
                    Ok(crate::db::postgres::SqlExecResult::Command { info }) => {
                        let _ = evt_tx.send(DbEvt::SqlExecuted { request_id, info }).await;
                    }
                    Err(e) => {
                        let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
                    }
                }
            }

            DbCmd::ExecuteSqlBatch {
                request_id,
                statements,
                refresh_schema,
            } => {
                let Some(pool) = pool.as_ref() else {
                    let _ = evt_tx.send(DbEvt::Error("Not connected.".into())).await;
                    continue;
                };

                if statements.is_empty() {
                    let _ = evt_tx
                        .send(DbEvt::SqlExecuted {
                            request_id,
                            info: "No SQL statements to execute.".into(),
                        })
                        .await;
                    continue;
                }

                match crate::db::postgres::execute_sql_batch(pool, &statements).await {
                    Ok(rows_affected) => {
                        let statement_count = statements.len();
                        let _ = evt_tx
                            .send(DbEvt::SqlExecuted {
                                request_id,
                                info: format!(
                                    "Batch OK. {statement_count} statement(s) executed. {rows_affected} rows affected."
                                ),
                            })
                            .await;
                        if let Some(schema) = refresh_schema {
                            load_sql_meta(pool, schema, &evt_tx).await;
                        }
                    }
                    Err(e) => {
                        let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn load_sql_meta(pool: &PgPool, schema: String, evt_tx: &mpsc::Sender<DbEvt>) {
    match crate::db::postgres::load_tables(pool, &schema).await {
        Ok(tables) => match crate::db::postgres::load_columns(pool, &schema).await {
            Ok(columns) => {
                let _ = evt_tx
                    .send(DbEvt::SqlMetaLoaded {
                        schema,
                        tables,
                        columns,
                    })
                    .await;
            }
            Err(e) => {
                let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
            }
        },
        Err(e) => {
            let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
        }
    }
}
