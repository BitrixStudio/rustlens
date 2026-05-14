# GUI Plan

The native GUI app is named `rustlens-gui` and uses `egui`/`eframe`.

## Why egui/eframe

- Rust-only desktop stack.
- Good fit for custom tooling UIs with panes, tabs, popups, and dense grids.
- `egui_extras::TableBuilder` supports efficient visible-row rendering.
- Easy async DB-worker integration by draining events in `eframe::App::update`.

## Planned Layout

- Left sidebar: profiles, schemas, and table list.
- Main tabs: Browse and SQL.
- Browse tab: paged table result grid.
- SQL tab: multiline SQL editor, execute controls, result grid.
- Bottom status bar: connection, schema, query status.
- Optional inspector: selected row/cell details.

## Current Implementation

Implemented MVP:

- `crates/rustlens-gui`: egui/eframe application library.
- `apps/rustlens-gui`: thin binary with binary name `rustlens-gui`.
- User-config TOML profile loading.
- Profile connection through the existing `rustlens-core` DB worker.
- Browse tab with paged table loading.
- SQL tab with multiline editor and `F5`/`Ctrl+Enter` execution.
- Clickable result cells with a full-value Cell Inspector.
- Table page cache for previously loaded Browse pages.
- Loading status for table page, SQL, and schema apply requests.
- SQL editor syntax highlighting and shared metadata-aware completion.
- Schema Studio tab with visual table/column designer and live SQL preview.
- Destructive/mutating SQL confirmation modal.
- Sidebar profile edit/delete actions.
- Resizable result table using `egui_extras::TableBuilder`.
- Custom RustLens dark theme with graphite panels, teal primary actions, amber warnings, and red destructive actions.

Run with:

```bash
cargo run --bin rustlens-gui
```

## Performance Requirements

- Keep server-side pagination.
- Do not fetch full tables by default.
- Render only visible grid rows.
- Use DB request IDs to ignore stale table/query results.
- Cache loaded table pages by schema/table/page/page size and clear cache on invalidation.
- Truncate long cell values in grids and show full text in the Cell Inspector.
- Do not block in `eframe::App::update`.

## Crate Layout

```text
apps/rustlens-gui/
  Cargo.toml
  src/main.rs

crates/rustlens-gui/
  Cargo.toml
  src/lib.rs
  src/dialogs.rs
  src/events.rs
  src/helpers.rs
  src/profile_flow.rs
  src/profiles.rs
  src/schema_studio.rs
  src/sidebar.rs
  src/sql_editor.rs
  src/tabs.rs
  src/theme.rs
  src/wizard.rs
```

## Dependency Direction

```toml
eframe = "0.28"
egui = "0.28"
egui_extras = "0.28"
```

Keep SQLx in `rustlens-core`; the GUI uses the DB worker protocol.

## Remaining GUI Work

- Continue splitting `src/lib.rs` only when a new seam becomes clear. Current remaining responsibilities are app construction, top/bottom panel orchestration, small shared UI helpers, and result-table rendering.
- Continue moving ad-hoc widgets into reusable themed components.
- Add query cancellation and query timeout UI.
- Extract UI-neutral app state into `rustlens-app` so TUI and GUI share reducers.
