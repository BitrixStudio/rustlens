# Architecture Map

Key files:

- `Cargo.toml`: workspace members.
- `apps/rustlens/src/main.rs`: direct viewer CLI.
- `apps/rustlensmanager/src/main.rs`: manager CLI.
- `crates/rustlens-core/src/db/protocol.rs`: `DbCmd`/`DbEvt`.
- `crates/rustlens-core/src/db/worker.rs`: async DB actor.
- `crates/rustlens-core/src/db/postgres.rs`: Postgres queries.
- `crates/rustlens-core/src/model/connection.rs`: connection profile model.
- `crates/rustlens-tui/src/lib.rs`: launch mode public API.
- `crates/rustlens-tui/src/app/reducer.rs`: TUI state transitions.
- `crates/rustlens-tui/src/app/state.rs`: current app state.
- `crates/rustlens-tui/src/term/input.rs`: key mapping.
- `crates/rustlens-tui/src/ui/*`: rendering.

Planned crates:

- `crates/rustlens-app`: UI-neutral app state/actions/reducer.
- `crates/rustlens-gui`: egui implementation.
- `apps/rustlens-gui`: GUI binary.
