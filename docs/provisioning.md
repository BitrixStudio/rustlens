# Provisioning

RustLens uses `dbnest-core` for managed local database creation.

During development, `rustlens-core` depends on the local checkout:

```toml
dbnest-core = { path = "../../../dbnest/crates/dbnest-core" }
```

Before release, switch this to a crates.io version once the required `dbnest-core` API is published.

## Supported Now

PostgreSQL Docker:

- Uses `dbnest_core::provision` with `Engine::Postgres`.
- Creates a Docker container and persistent volume through dbnest.
- Maps the returned dbnest instance into a RustLens profile.
- Saves `source = "dbnest"` and `dbnest_instance_id` in the profile.
- After creation, the GUI saves the profile, connects to it, and opens Schema Studio with the starter schema template selected.

SQLite:

- Uses `dbnest_core::provision` with `Engine::Sqlite`.
- Creates a SQLite file or dbnest-managed file.
- Saves the profile as `driver = "sqlite"`.
- Browsing is disabled until RustLens adds SQLite DB support.

## Remaining Work

- Add dbnest instance lifecycle UI: start, stop, restart, remove.
- Add query support for SQLite before enabling browsing.
- Add request IDs and cancellation for long-running provisioning and DB operations.
- Add starter template selection during the create-local flow.

## Live Test

The live dbnest provisioning test is available at:

```text
crates/rustlens-core/tests/dbnest_provisioning.rs
```

Run it with Docker access:

```bash
cargo test -p rustlens-core --test dbnest_provisioning -- --ignored --nocapture
```

It creates a real PostgreSQL Docker database through dbnest, verifies a RustLens core connection, then removes the dbnest instance, container, and volume.
