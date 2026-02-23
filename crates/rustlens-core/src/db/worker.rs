use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::mpsc;

use crate::db::{DbCmd, DbEvt};

pub async fn run(
    database_url: String,
    mut cmd_rx: mpsc::Receiver<DbCmd>,
    evt_tx: mpsc::Sender<DbEvt>,
) -> Result<()> {
    // We don't use `?` here because we want to report the error
    // back to the UI instead of crashing the worker task.
    let pool = match PgPoolOptions::new()
        .max_connections(6)
        .connect(&database_url)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            let _ = evt_tx
                .send(DbEvt::Error(format!("DB connect failed: {e}")))
                .await;
            return Ok(()); // terminate worker gracefully
        }
    };

    let _ = evt_tx.send(DbEvt::Status("Connected.".into())).await;

    while let Some(cmd) = cmd_rx.recv().await {
        let result: Result<()> = match cmd {
            DbCmd::LoadTables { schema } => {
                let tables = crate::db::postgres::load_tables(&pool, &schema).await?;
                let _ = evt_tx.send(DbEvt::TablesLoaded { tables }).await;
                Ok(())
            }

            DbCmd::LoadTablePage {
                schema,
                table,
                page,
                page_size,
            } => {
                let (columns, rows) =
                    crate::db::postgres::load_table_page(&pool, &schema, &table, page, page_size)
                        .await?;

                let _ = evt_tx
                    .send(DbEvt::QueryResult {
                        columns,
                        rows,
                        info: format!("Loaded page {}", page + 1),
                    })
                    .await;

                Ok(())
            }

            DbCmd::ExecuteSql { sql } => {
                match crate::db::postgres::execute_sql(&pool, &sql).await? {
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

                Ok(())
            }
        };

        if let Err(e) = result {
            let _ = evt_tx.send(DbEvt::Error(e.to_string())).await;
        }
    }

    Ok(())
}
