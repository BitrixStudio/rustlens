use anyhow::Result;
use rustlens_core::model::connection::{ConnectionProfile, Driver, ProfileSource};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub driver: Driver,
    pub database_url: String,
    pub schema: String,
    pub page_size: i64,
    pub source: ProfileSource,
    pub dbnest_instance_id: Option<String>,
}

impl Profile {
    pub fn supports_browsing(&self) -> bool {
        self.driver == Driver::Postgres
    }
}

pub fn load_profiles() -> Result<Vec<Profile>> {
    Ok(rustlens_core::profiles::load_profiles()?
        .into_iter()
        .map(Profile::from)
        .collect())
}

pub fn add_profile(profile: ConnectionProfile) -> Result<Profile> {
    rustlens_core::profiles::add_profile(profile).map(Profile::from)
}

pub fn update_profile(profile: ConnectionProfile) -> Result<Profile> {
    rustlens_core::profiles::update_profile(profile.clone())?;
    Ok(Profile::from(profile))
}

pub fn delete_profile(id: Uuid) -> Result<()> {
    rustlens_core::profiles::delete_profile(id)
}

pub fn profiles_path_text() -> String {
    rustlens_core::profiles::profiles_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|err| format!("Could not resolve profile path: {err}"))
}

pub fn manual_postgres_profile(
    name: String,
    database_url: String,
    schema: String,
    page_size: i64,
) -> ConnectionProfile {
    ConnectionProfile {
        id: Uuid::new_v4(),
        name,
        driver: Driver::Postgres,
        database_url,
        schema: Some(schema),
        page_size: Some(page_size),
        source: ProfileSource::Manual,
        dbnest_instance_id: None,
    }
}

impl From<ConnectionProfile> for Profile {
    fn from(profile: ConnectionProfile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            driver: profile.driver,
            database_url: profile.database_url,
            schema: profile.schema.unwrap_or_else(|| "public".to_string()),
            page_size: profile.page_size.unwrap_or(200),
            source: profile.source,
            dbnest_instance_id: profile.dbnest_instance_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{manual_postgres_profile, Profile};
    use rustlens_core::model::connection::{Driver, ProfileSource};

    #[test]
    fn creates_manual_postgres_profile() {
        let profile = manual_postgres_profile(
            "local".to_string(),
            "postgres://localhost/app".to_string(),
            "app".to_string(),
            50,
        );

        assert_eq!(profile.name, "local");
        assert_eq!(profile.driver, Driver::Postgres);
        assert_eq!(profile.source, ProfileSource::Manual);
        assert_eq!(profile.schema.as_deref(), Some("app"));
        assert_eq!(profile.page_size, Some(50));
    }

    #[test]
    fn sqlite_profiles_do_not_support_browsing_yet() {
        let profile = Profile {
            id: uuid::Uuid::new_v4(),
            name: "sqlite".to_string(),
            driver: Driver::Sqlite,
            database_url: "sqlite:///tmp/app.sqlite".to_string(),
            schema: "main".to_string(),
            page_size: 200,
            source: ProfileSource::Dbnest,
            dbnest_instance_id: Some("abc".to_string()),
        };

        assert!(!profile.supports_browsing());
    }
}
