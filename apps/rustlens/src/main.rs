use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    database_url: String,

    #[arg(default_value = "public")]
    schema: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    validate_db_url(&args.database_url)?;

    // Fail fast before launching the UI.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        rustlens_core::db::connect::connect(&args.database_url).await?;
        Ok::<(), anyhow::Error>(())
    })?;

    rustlens_tui::run(rustlens_tui::LaunchMode::Viewer {
        database_url: args.database_url,
        schema: Some(args.schema),
    })
}

fn validate_db_url(database_url: &str) -> anyhow::Result<()> {
    let scheme = database_url
        .split("://")
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    match scheme.as_str() {
        "postgres" | "postgresql" => Ok(()),
        "" => anyhow::bail!("Invalid database URL (missing scheme): {database_url}"),
        other => anyhow::bail!("Unsupported database scheme: {other}"),
    }
}
