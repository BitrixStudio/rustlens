use anyhow::Result;
use tokio::sync::mpsc;

use crate::{config::AppConfig, LaunchMode};
use rustlens_core::db;

pub fn run_app(cfg: AppConfig, mode: LaunchMode) -> Result<()> {
    // We need a tokio runtime because rustlens_tui::run is sync (called from bin main()).
    // Keep it explicit so the binaries stay tiny.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let mut terminal = crate::term::terminal::TerminalGuard::new()?;

        let (db_cmd_tx, db_cmd_rx) = mpsc::channel::<db::DbCmd>(64);
        let (db_evt_tx, mut db_evt_rx) = mpsc::channel::<db::DbEvt>(256);

        let cfg_for_worker = cfg.clone();
        tokio::spawn(async move {
            if let Err(e) = db::worker::run(cfg_for_worker.database_url, db_cmd_rx, db_evt_tx).await
            {
                eprintln!("db worker crashed: {e:#}");
            }
        });

        let mut root = crate::app::state::RootState::new(cfg.clone(), mode);

        // In viewer mode, start immediately by loading tables.
        // In manager mode, you’ll load profiles from storage later; for now it can also load tables.
        db_cmd_tx
            .send(db::DbCmd::LoadTables {
                schema: cfg.schema.clone(),
            })
            .await
            .ok();

        use crate::app::event::AppEvent;

        loop {
            if let Some(input) = crate::term::input::poll_next_event(root.session.tick_rate)? {
                if crate::app::reducer::reduce(&mut root, AppEvent::Input(input), &db_cmd_tx).await
                {
                    break;
                }
            }

            while let Ok(evt) = db_evt_rx.try_recv() {
                if crate::app::reducer::reduce(&mut root, AppEvent::Db(evt), &db_cmd_tx).await {
                    break;
                }
            }

            terminal.draw(|f| crate::ui::draw::draw(f, &mut root))?;
        }

        Ok(())
    })
}
