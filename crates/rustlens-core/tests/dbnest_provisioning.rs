use anyhow::{Context, Result};
use rustlens_core::model::connection::{Driver, ProfileSource};
use rustlens_core::provision::dbnest::{create_postgres_docker, CreatePostgresDocker};
use uuid::Uuid;

struct DbnestCleanup {
    instance_id: Option<String>,
}

impl DbnestCleanup {
    fn new(instance_id: String) -> Self {
        Self {
            instance_id: Some(instance_id),
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        if let Some(instance_id) = self.instance_id.take() {
            dbnest_core::remove_instance(&instance_id, true, true)
                .with_context(|| format!("failed to remove dbnest instance {instance_id}"))?;
        }

        Ok(())
    }
}

impl Drop for DbnestCleanup {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[tokio::test]
#[ignore = "requires Docker access; provisions and removes a real dbnest PostgreSQL container"]
async fn provisions_postgres_docker_and_connects_with_rustlens_core() -> Result<()> {
    let suffix = Uuid::new_v4().simple().to_string();
    let database = format!("rustlens_{suffix}");
    let profile = create_postgres_docker(CreatePostgresDocker {
        profile_name: format!("rustlens-live-{suffix}"),
        user: "rustlens".to_string(),
        password: "rustlens".to_string(),
        database,
        image: Some("postgres:16-alpine".to_string()),
        schema: "public".to_string(),
        page_size: 50,
    })?;

    let instance_id = profile
        .dbnest_instance_id
        .clone()
        .context("dbnest profile did not include instance id")?;
    let mut cleanup = DbnestCleanup::new(instance_id);

    assert_eq!(profile.driver, Driver::Postgres);
    assert_eq!(profile.source, ProfileSource::Dbnest);
    assert!(profile.database_url.starts_with("postgres://"));

    let pool = rustlens_core::db::connect::connect(&profile.database_url).await?;
    let value: i32 = sqlx::query_scalar("select 1").fetch_one(&pool).await?;
    assert_eq!(value, 1);
    pool.close().await;

    cleanup.cleanup()?;
    Ok(())
}
