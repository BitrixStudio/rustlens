use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::model::connection::ConnectionProfile;

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProfilesFile {
    #[serde(default)]
    profiles: Vec<ConnectionProfile>,
}

pub fn load_profiles() -> Result<Vec<ConnectionProfile>> {
    load_profiles_from_path(&profiles_path()?)
}

pub fn save_profiles(profiles: &[ConnectionProfile]) -> Result<()> {
    save_profiles_to_path(&profiles_path()?, profiles)
}

pub fn add_profile(mut profile: ConnectionProfile) -> Result<ConnectionProfile> {
    let mut profiles = load_profiles()?;
    if profile.id == Uuid::nil() {
        profile.id = Uuid::new_v4();
    }
    profiles.push(profile.clone());
    save_profiles(&profiles)?;
    Ok(profile)
}

pub fn update_profile(profile: ConnectionProfile) -> Result<()> {
    let mut profiles = load_profiles()?;
    let Some(existing) = profiles.iter_mut().find(|p| p.id == profile.id) else {
        anyhow::bail!("Profile not found: {}", profile.id);
    };
    *existing = profile;
    save_profiles(&profiles)
}

pub fn delete_profile(id: Uuid) -> Result<()> {
    let mut profiles = load_profiles()?;
    let before = profiles.len();
    profiles.retain(|profile| profile.id != id);
    if profiles.len() == before {
        anyhow::bail!("Profile not found: {id}");
    }
    save_profiles(&profiles)
}

pub fn profiles_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "rustlens")
        .context("Could not resolve user config directory for rustlens")?;
    Ok(dirs.config_dir().join("profiles.toml"))
}

pub fn load_profiles_from_path(path: &Path) -> Result<Vec<ConnectionProfile>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path)
        .with_context(|| format!("Could not read profiles file: {}", path.display()))?;
    parse_profiles(&raw).with_context(|| format!("Invalid profiles file: {}", path.display()))
}

pub fn save_profiles_to_path(path: &Path, profiles: &[ConnectionProfile]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Could not create profile directory: {}", parent.display()))?;
    }

    let file = ProfilesFile {
        profiles: profiles.to_vec(),
    };
    let raw = toml::to_string_pretty(&file).context("Could not serialize profiles")?;
    fs::write(path, raw)
        .with_context(|| format!("Could not write profiles file: {}", path.display()))
}

pub fn parse_profiles(raw: &str) -> Result<Vec<ConnectionProfile>> {
    let file: ProfilesFile = toml::from_str(raw)?;
    Ok(file.profiles)
}

#[cfg(test)]
mod tests {
    use super::{parse_profiles, save_profiles_to_path};
    use crate::model::connection::{ConnectionProfile, Driver, ProfileSource};
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn parses_profiles_with_defaults() {
        let profiles = parse_profiles(
            r#"
            [[profiles]]
            name = "local"
            driver = "postgres"
            database_url = "postgres://localhost/app"
            "#,
        )
        .unwrap();

        assert_eq!(profiles.len(), 1);
        assert_ne!(profiles[0].id, Uuid::nil());
        assert_eq!(profiles[0].driver, Driver::Postgres);
        assert_eq!(profiles[0].schema_or_default(), "public");
        assert_eq!(profiles[0].page_size_or_default(), 200);
        assert_eq!(profiles[0].source, ProfileSource::Manual);
    }

    #[test]
    fn saves_profiles_as_toml() {
        let dir = std::env::temp_dir().join(format!("rustlens-profile-test-{}", Uuid::new_v4()));
        let path = dir.join("profiles.toml");
        let profile = ConnectionProfile {
            id: Uuid::new_v4(),
            name: "local".to_string(),
            driver: Driver::Postgres,
            database_url: "postgres://localhost/app".to_string(),
            schema: Some("app".to_string()),
            page_size: Some(50),
            source: ProfileSource::Manual,
            dbnest_instance_id: None,
        };

        save_profiles_to_path(&path, &[profile]).unwrap();
        let raw = fs::read_to_string(&path).unwrap();
        assert!(raw.contains("[[profiles]]"));
        assert!(raw.contains("driver = \"postgres\""));

        let _ = fs::remove_dir_all(dir);
    }
}
