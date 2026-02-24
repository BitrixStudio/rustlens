use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Database URL, e.g. postgres://user:pass@localhost:5432/db
    database_url: String,
    schema: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    rustlens_tui::run(rustlens_tui::LaunchMode::Viewer {
        database_url: args.database_url,
        schema: args.schema,
    })
}
