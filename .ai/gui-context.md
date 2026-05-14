# GUI Context

The GUI binary must be named `rustlens-gui`.

Use `egui`/`eframe`, not Tauri. Keep DB access behind `rustlens-core` worker channels. Use `egui_extras::TableBuilder` for grids.

Current GUI MVP supports:

- Profile selection from user config TOML.
- First-start setup wizard when no profiles exist.
- Add existing PostgreSQL profile.
- Create PostgreSQL Docker and SQLite profiles through local `dbnest-core`.
- Capability detection and installer command confirmation UI.
- Browse tab with paged result grid.
- SQL tab with editor and result grid.
- Destructive SQL confirmation modal.
- Status bar.
- Custom RustLens dark theme in `crates/rustlens-gui/src/theme.rs`.
- GUI SQL editor uses `crates/rustlens-gui/src/sql_editor.rs` for highlighting and `rustlens-core/src/sql/completion.rs` for completion.
- Schema Studio lives in `crates/rustlens-gui/src/schema_studio.rs`; current phase designs tables/columns and previews SQL, but does not apply schema yet.

Remaining GUI priorities:

- Add profile edit/delete UI.
- Add page cache and request IDs to ignore stale result events.
- Add query cancellation/timeout controls.
- Add selected cell/row inspector.
- Move duplicated GUI/TUI SQL safety helpers into shared/core modules.
