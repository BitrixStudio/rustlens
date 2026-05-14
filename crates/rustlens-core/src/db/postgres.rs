use crate::util::value_fmt::cell_to_string;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use sqlx::{Column, Executor, PgPool, Row as _};

pub async fn load_tables(pool: &PgPool, schema: &str) -> Result<Vec<String>> {
    let exists = schema_exists(pool, schema).await?;

    if !exists {
        return Err(anyhow!(r#"schema "{}" does not exist"#, schema));
    }
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

pub async fn load_columns(pool: &PgPool, schema: &str) -> Result<Vec<(String, Vec<String>)>> {
    let exists = schema_exists(pool, schema).await?;

    if !exists {
        return Err(anyhow!(r#"schema "{}" does not exist"#, schema));
    }

    let rows = sqlx::query(
        r#"
        select table_name, column_name
        from information_schema.columns
        where table_schema = $1
        order by table_name, ordinal_position
        "#,
    )
    .bind(schema)
    .fetch_all(pool)
    .await?;

    let mut out: Vec<(String, Vec<String>)> = Vec::new();

    for r in rows {
        let t: String = r.get("table_name");
        let c: String = r.get("column_name");

        if out.last().map(|(tt, _)| tt.as_str()) == Some(t.as_str()) {
            out.last_mut().unwrap().1.push(c);
        } else {
            out.push((t, vec![c]));
        }
    }

    Ok(out)
}

pub async fn load_table_page(
    pool: &PgPool,
    schema: &str,
    table: &str,
    page: i64,
    page_size: i64,
) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    if page < 0 {
        return Err(anyhow!("page must be greater than or equal to 0"));
    }
    if page_size <= 0 {
        return Err(anyhow!("page_size must be greater than 0"));
    }

    let offset = page
        .checked_mul(page_size)
        .ok_or_else(|| anyhow!("page offset overflow"))?;
    let sql = format!(
        "select * from {}.{} limit $1 offset $2",
        quote_ident(schema),
        quote_ident(table),
    );

    let columns: Vec<String> = pool
        .describe(&sql)
        .await?
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let rows = sqlx::query(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

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
    let columns: Vec<String> = match pool.describe(sql).await {
        Ok(describe) => describe
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect(),
        Err(_) => Vec::new(),
    };

    if columns.is_empty() {
        let res = sqlx::query(sql).execute(pool).await?;
        return Ok(SqlExecResult::Command {
            info: format!("OK. {} rows affected.", res.rows_affected()),
        });
    }

    let rows = sqlx::query(sql).fetch_all(pool).await?;

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

pub async fn execute_sql_batch(pool: &PgPool, statements: &[String]) -> Result<u64> {
    let mut tx = pool.begin().await?;
    let mut rows_affected = 0;

    for (idx, statement) in statements.iter().enumerate() {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }

        let result = sqlx::query(trimmed)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("statement {} failed:\n{}", idx + 1, statement))?;
        rows_affected += result.rows_affected();
    }

    tx.commit().await?;
    Ok(rows_affected)
}

async fn schema_exists(pool: &PgPool, schema: &str) -> Result<bool> {
    let exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM information_schema.schemata
            WHERE schema_name = $1
        )
        "#,
    )
    .bind(schema)
    .fetch_one(pool)
    .await?;

    Ok(exists)
}
