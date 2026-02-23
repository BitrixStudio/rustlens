use crate::util::value_fmt::cell_to_string;
use anyhow::Result;
use sqlx::{Column, PgPool, Row as _};

pub async fn load_tables(pool: &PgPool, schema: &str) -> Result<Vec<String>> {
    let rows = sqlx::query(
        r#"
        select tablename
        from pg_catalog.pg_tables
        where schemaname = $1
        order by tablename
        "#,
    )
    .bind(schema)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| r.get::<String, _>("tablename"))
        .collect())
}

fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub async fn load_table_page(
    pool: &PgPool,
    schema: &str,
    table: &str,
    page: i64,
    page_size: i64,
) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let offset = page * page_size;
    let sql = format!(
        "select * from {}.{} limit $1 offset $2",
        quote_ident(schema),
        quote_ident(table),
    );

    let rows = sqlx::query(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    let columns: Vec<String> = rows
        .get(0)
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
        .unwrap_or_else(|| vec![]);

    // Values (generic display for MVP phase)
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let mut vals = Vec::with_capacity(columns.len());
        for i in 0..columns.len() {
            vals.push(cell_to_string(&r, i));
        }
        out.push(vals);
    }

    Ok((columns, out))
}

pub enum SqlExecResult {
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Command {
        info: String,
    },
}

pub async fn execute_sql(pool: &PgPool, sql: &str) -> Result<SqlExecResult> {
    // MVP strategy:
    // Try fetch_all -> if it fails due to "no rows returned" then fall back to execute.
    // sqlx doesn't give a universal "is this SELECT" without parsing; this is simple and works.
    // Should be refactored, this is only concept for development
    match sqlx::query(sql).fetch_all(pool).await {
        Ok(rows) => {
            let columns: Vec<String> = rows
                .get(0)
                .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
                .unwrap_or_else(|| vec![]);

            let mut out = Vec::with_capacity(rows.len());
            for r in rows {
                let mut vals = Vec::with_capacity(columns.len());
                for i in 0..columns.len() {
                    vals.push(cell_to_string(&r, i));
                }
                out.push(vals);
            }

            Ok(SqlExecResult::Rows { columns, rows: out })
        }
        Err(_) => {
            let res = sqlx::query(sql).execute(pool).await?;
            Ok(SqlExecResult::Command {
                info: format!("OK. {} rows affected.", res.rows_affected()),
            })
        }
    }
}
