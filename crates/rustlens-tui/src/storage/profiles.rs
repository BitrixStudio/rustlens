use anyhow::Result;
use rustlens_core::model::connection::Driver;

use crate::app::state::DbProfile;

pub fn load_profiles() -> Result<Vec<DbProfile>> {
    Ok(rustlens_core::profiles::load_profiles()?
        .into_iter()
        .filter(|profile| profile.driver == Driver::Postgres)
        .map(|profile| {
            let schema = profile.schema_or_default();
            let page_size = profile.page_size_or_default();
            DbProfile {
                name: profile.name,
                database_url: profile.database_url,
                schema,
                page_size,
            }
        })
        .collect())
}

#[allow(dead_code)]
pub fn profiles_path() -> Result<std::path::PathBuf> {
    rustlens_core::profiles::profiles_path()
}
