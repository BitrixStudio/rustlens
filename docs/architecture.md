# Architecture

RustLens is a Rust workspace for lightweight Postgres database browsing.

## Workspace Layout

```text
apps/
  rustlens/          # Direct TUI viewer binary
  rustlensmanager/   # TUI profile manager binary

crates/
  rustlens-core/     # DB protocol, Postgres access, shared models, value formatting
  rustlens-tui/      # Ratatui/crossterm UI, input, state, reducer
  rustlens-gui/      # egui/eframe native GUI viewer
```

Planned additions:

```text
apps/
  rustlens-gui/      # Native Rust GUI viewer binary

crates/
  rustlens-app/      # UI-neutral state/actions/reducer
```

## Current Data Flow

```text
keyboard input
  -> rustlens-tui term::input::UiEvent
  -> app::reducer
  -> rustlens-core DbCmd
  -> db::worker
  -> Postgres/sqlx
  -> DbEvt
  -> app::reducer
  -> ratatui draw
```

The DB worker owns the SQLx pool. UI code communicates with it through bounded Tokio channels.

Result-producing commands can carry optional request IDs. The GUI assigns request IDs to table loads, SQL execution, and schema apply batches, then ignores stale `QueryResult` / `SqlExecuted` events whose IDs no longer match the latest user-visible result request. The GUI also attaches request context so table-page results can populate a page cache and in-flight operations can show loading status. The TUI currently sends `None` and keeps its existing behavior.

Direct viewer startup now sends `DbCmd::Connect` first. Once the worker emits `DbEvt::Connected`, the reducer requests `LoadSqlMeta` for the active schema.

## Core Responsibilities

- `rustlens-core/src/db/protocol.rs`: command/event protocol between UI and DB worker.
- `rustlens-core/src/db/worker.rs`: async worker loop and connection ownership.
- `rustlens-core/src/db/postgres.rs`: Postgres metadata, table page loading, and SQL execution.
- `rustlens-core/src/sql/completion.rs`: shared SQL completion logic.
- `rustlens-core/src/sql/safety.rs`: shared destructive-SQL confirmation classifier.
- `rustlens-core/src/util/value_fmt.rs`: conversion of Postgres cells to display strings.
- `rustlens-core/src/model/connection.rs`: connection profile model.

## TUI Responsibilities

- `rustlens-tui/src/lib.rs`: public launch API and mode-specific config bootstrap.
- `rustlens-tui/src/app/state.rs`: current app state, including some terminal widget state.
- `rustlens-tui/src/app/reducer.rs`: input and DB event handling.
- `rustlens-tui/src/term/input.rs`: crossterm key mapping.
- `rustlens-tui/src/ui/*`: ratatui layout and rendering.

## Refactor Direction

The current TUI state contains Ratatui-specific state (`ListState`, `TableState`), which should not be shared with a GUI. Before building the GUI deeply, introduce `rustlens-app` with UI-neutral state and actions, then make TUI and GUI thin render/input layers.

The GUI MVP currently has its own UI state and uses the same `rustlens-core` worker protocol. A future refactor should extract remaining duplicated profile/app-state logic into shared modules.

`crates/rustlens-gui/src/lib.rs` still owns the main app state and top-level panel orchestration, but sidebar/profile UI, confirmation dialogs, tab rendering, setup wizard rendering, event handling, profile flow, and profile/provisioning helpers have been split into focused modules. This keeps feature work localized without introducing a broader app-state crate prematurely.

## SQL Safety

The TUI and GUI use `rustlens-core/src/sql/safety.rs` as a conservative destructive-SQL confirmation gate before sending SQL execution commands. They also share metadata-aware completion through `rustlens-core/src/sql/completion.rs`.
