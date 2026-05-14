use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            schema: default_schema(),
            page_size: default_page_size(),
        }
    }
}

fn default_schema() -> String {
    "public".to_string()
}
fn default_page_size() -> i64 {
    200
}

#[allow(dead_code)]
pub fn load_from_file(path: &str) -> Result<AppConfig> {
    let s = fs::read_to_string(path).with_context(|| format!("Could not read {}", path))?;
    let cfg: AppConfig = toml::from_str(&s).context("Invalid config.toml")?;
    Ok(cfg)
}
