use anyhow::Result;
use uuid::Uuid;

use crate::model::connection::{ConnectionProfile, Driver, ProfileSource};

#[derive(Debug, Clone)]
pub struct CreatePostgresDocker {
    pub profile_name: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub image: Option<String>,
    pub schema: String,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub struct CreateSqlite {
    pub profile_name: String,
    pub path: Option<std::path::PathBuf>,
}

pub fn create_postgres_docker(opts: CreatePostgresDocker) -> Result<ConnectionProfile> {
    let instance = dbnest_core::provision(dbnest_core::InstanceSpec {
        engine: dbnest_core::Engine::Postgres,
        sqlite: None,
        postgres: Some(dbnest_core::PostgresSpec {
            user: opts.user,
            password: opts.password,
            db: opts.database,
            image: opts.image,
        }),
    })?;

    Ok(profile_from_dbnest_instance(
        opts.profile_name,
        Driver::Postgres,
        opts.schema,
        opts.page_size,
        &instance,
    ))
}

pub fn create_sqlite(opts: CreateSqlite) -> Result<ConnectionProfile> {
    let instance = dbnest_core::provision(dbnest_core::InstanceSpec {
        engine: dbnest_core::Engine::Sqlite,
        sqlite: Some(dbnest_core::SqliteSpec { path: opts.path }),
        postgres: None,
    })?;

    Ok(profile_from_dbnest_instance(
        opts.profile_name,
        Driver::Sqlite,
        "main".to_string(),
        200,
        &instance,
    ))
}

pub fn profile_from_dbnest_instance(
    name: String,
    driver: Driver,
    schema: String,
    page_size: i64,
    instance: &dbnest_core::Instance,
) -> ConnectionProfile {
    ConnectionProfile {
        id: Uuid::new_v4(),
        name,
        driver,
        database_url: instance.connection.database_url.clone(),
        schema: Some(schema),
        page_size: Some(page_size),
        source: ProfileSource::Dbnest,
        dbnest_instance_id: Some(instance.id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::profile_from_dbnest_instance;
    use crate::model::connection::{Driver, ProfileSource};

    #[test]
    fn maps_dbnest_instance_to_profile() {
        let instance = dbnest_core::Instance {
            id: "abc".to_string(),
            engine: dbnest_core::Engine::Postgres,
            backend: dbnest_core::Backend::Container,
            created_at: time::OffsetDateTime::now_utc(),
            connection: dbnest_core::ConnectionInfo {
                database_url: "postgres://localhost/app".to_string(),
                host: Some("localhost".to_string()),
                port: Some(5432),
                database: Some("app".to_string()),
                user: Some("dev".to_string()),
            },
            sqlite: None,
            container: None,
        };

        let profile = profile_from_dbnest_instance(
            "local".to_string(),
            Driver::Postgres,
            "public".to_string(),
            100,
            &instance,
        );

        assert_eq!(profile.name, "local");
        assert_eq!(profile.driver, Driver::Postgres);
        assert_eq!(profile.source, ProfileSource::Dbnest);
        assert_eq!(profile.dbnest_instance_id.as_deref(), Some("abc"));
    }
}
