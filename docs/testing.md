# Testing

## Current Status

The project has initial unit tests for SQL safety classification, cursor line/column behavior, and TUI key-event mapping. `rustlens-core` also has ignored Docker/testcontainers Postgres integration tests that compile during normal test runs but require explicit execution.

Profile storage, installer command generation, dbnest profile mapping, and GUI helper tests are also covered by unit tests.

## Unit Test Targets

- DB worker command failure handling should emit errors and continue.
- SQL completion prefix/context behavior.
- SQL editor cursor movement, newline insertion, and completion acceptance.
- Input mapping for Enter, F5, Ctrl+Enter, navigation, and refresh.
- Profile TOML parsing and validation.
- SQL safety classifier for destructive statements.

## Integration Test Targets

Integration tests use Docker/testcontainers to start Postgres and create disposable schemas/tables.

Coverage should include:

- `load_tables` with existing schema.
- `load_columns` ordered by table and ordinal position.
- `load_table_page` for populated and empty tables.
- Empty table pages preserve column metadata.
- Empty `SELECT` returns column metadata.
- `INSERT ... RETURNING` returns rows.
- `UPDATE` reports affected rows.
- Bad SQL returns a recoverable worker error and does not kill the worker.

Run normal tests without Docker:

```bash
cargo test --workspace
```

Run ignored Postgres integration tests with Docker access:

```bash
cargo test -p rustlens-core --test postgres_integration -- --ignored --nocapture
```

Run live dbnest provisioning test with Docker access:

```bash
cargo test -p rustlens-core --test dbnest_provisioning -- --ignored --nocapture
```

This provisions a real PostgreSQL Docker container through `dbnest-core`, connects with RustLens core, and removes the dbnest instance/container/volume during cleanup.

If this fails with Docker socket permissions, add the current user to the Docker group or run in a CI job/container with Docker access.

## Verification

Before merging larger changes, run:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
