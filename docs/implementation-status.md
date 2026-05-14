# Implementation Status

## Completed In Current Pass

- Added developer documentation under `docs/`.
- Added AI-agent context under `.ai/`.
- Fixed direct viewer startup to connect before metadata loading.
- Added `DbEvt::Connected` and reducer-driven metadata loading after successful connect.
- Implemented real `LoadSqlMeta` with table and column metadata.
- Made DB worker command failures emit `DbEvt::Error` instead of crashing the worker.
- Fixed Browse Enter vs SQL Enter behavior.
- Changed SQL execution shortcut to `F5` and `Ctrl+Enter`.
- Added destructive/mutating SQL confirmation in the TUI.
- Replaced hardcoded manager profiles with user-config TOML loading.
- Added initial unit tests for SQL safety and cursor positioning.
- Added ignored Docker/testcontainers Postgres integration tests for core DB behavior.
- Added TUI key-event mapping tests and extracted the pure key mapping function.
- Fixed `j`/`k` Browse navigation while preserving `j`/`k` typing in the SQL editor.
- Added `rustlens-gui` egui/eframe MVP with profile loading, connection, Browse tab, SQL tab, result grid, and destructive SQL confirmation.
- Added GUI helper tests for profile parsing, SQL safety, and cell truncation.
- Moved shared profile model/storage into `rustlens-core`.
- Added `dbnest-core` local path integration for PostgreSQL Docker and SQLite provisioning.
- Added GUI first-start setup wizard with add-existing, create-local, and install-tools flows.
- Added capability detection and installer command generation.
- Added ignored live dbnest provisioning integration test with cleanup.
- Added a dedicated `rustlens-gui` theme module and replaced the plain default egui look with a custom dark RustLens palette.
- Added shared SQL completion in `rustlens-core` and wired GUI SQL highlighting/autocomplete.
- Added initial Schema Studio visual table/column designer with validation and SQL preview.
- Fixed empty table/query column metadata by using SQLx describe metadata.
- Added pagination validation and corrected Postgres integer/float decoding.
- Added confirmed Schema Studio apply through a transactional DB-worker batch with metadata refresh on success.
- Updated successful PostgreSQL provisioning to connect and open Schema Studio with the starter template.
- Added Schema Studio starter templates for Users, Content, and Tasks.
- Moved destructive-SQL confirmation classification into `rustlens-core` and wired both TUI and GUI to the shared classifier.
- Wired TUI SQL completion to the shared `rustlens-core` completion engine.
- Added confirmed profile deletion in the GUI sidebar.
- Added basic GUI profile editing for name, URL, schema, and page size.
- Started GUI module refactor by extracting sidebar/profile UI and confirmation dialogs out of `src/lib.rs`.
- Continued GUI module refactor by extracting Browse, SQL, and Schema Studio tab rendering into `tabs.rs`.
- Continued GUI module refactor by extracting setup wizard rendering into `wizard.rs`.
- Continued GUI module refactor by extracting DB/provisioning event handling, profile flow, and profile/provisioning helper functions.
- Added optional DB request IDs and GUI stale-result suppression for table loads, SQL execution, and schema apply batches.
- Added clickable GUI result cells and a Cell Inspector for viewing full cell values.
- Added GUI table page cache and loading status for table page, SQL, and schema apply requests.

## Verified

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Ignored Docker integration tests were not run in this environment because Docker socket access failed with permission denied.

The live dbnest provisioning test was attempted and still failed at `docker info` with Docker socket permission denied.

## Next Recommended Work

1. Run ignored Docker/testcontainers tests in an environment with Docker socket access.
2. Add reducer state-transition tests.
3. Add starter template selection during onboarding/provisioning.
4. Add dbnest instance lifecycle UI: start, stop, restart, remove.
5. Add query cancellation and query timeout UI.
6. Extract UI-neutral state/actions into `rustlens-app` after request-state semantics are clearer.
