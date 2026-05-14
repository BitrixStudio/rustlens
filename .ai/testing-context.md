# Testing Context

Run before finalizing implementation changes:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Testcontainers should be used for Postgres integration tests. Unit tests should cover pure logic first so they do not require Docker.

Current pure unit coverage includes SQL safety classification, SQL cursor line/column calculation, and TUI key mapping.

Postgres integration tests live at:

```text
crates/rustlens-core/tests/postgres_integration.rs
```

They are ignored by default. Run them with Docker access using:

```bash
cargo test -p rustlens-core --test postgres_integration -- --ignored --nocapture
```

Live dbnest provisioning test:

```bash
cargo test -p rustlens-core --test dbnest_provisioning -- --ignored --nocapture
```

This test provisions a real dbnest PostgreSQL Docker instance and removes it during cleanup. It requires Docker socket access.
