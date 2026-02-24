mod config;

pub mod app;
pub mod term;
pub mod ui;

use anyhow::Result;

#[derive(Debug, Clone)]
pub enum LaunchMode {
    Viewer {
        database_url: String,
        schema: Option<String>,
    },
    Manager,
}

/// Runs the TUI application.
/// - Viewer mode connects directly to a DB URL.
/// - Manager mode shows saved connections and opens viewer sessions.
pub fn run(mode: LaunchMode) -> Result<()> {
    let cfg = match &mode {
        LaunchMode::Viewer {
            database_url,
            schema,
        } => {
            // In direct viewer mode we don't require config.toml.
            config::AppConfig {
                database_url: database_url.clone(),
                schema: schema.clone().unwrap_or_else(|| "public".to_string()),
                page_size: 200,
            }
        }

        LaunchMode::Manager => {
            // Manager mode depends on config file.
            load_default_config()?
        }
    };

    app::run::run_app(cfg, mode)
}

fn load_default_config() -> Result<config::AppConfig> {
    // Simple, predictable precedence:
    // - debug: config-dev.toml if present
    // - else: config.toml
    let dev = "config-dev.toml";
    if cfg!(debug_assertions) && std::path::Path::new(dev).exists() {
        return config::load_from_file(dev);
    }
    config::load_from_file("config.toml")
}
