use anyhow::Result;

fn main() -> Result<()> {
    rustlens_tui::run(rustlens_tui::LaunchMode::Manager)
}
