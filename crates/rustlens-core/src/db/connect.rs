use anyhow::{Context, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::time::{timeout, Duration};

pub async fn connect(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(6)
        .connect(database_url)
        .await
        .context("failed to connect to database")
}
