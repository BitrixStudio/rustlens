# Development

## Common Commands

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Run the direct viewer:

```bash
cargo run -p rustlens -- postgres://app:app@localhost:5432/appdb public
```

Run the manager:

```bash
cargo run -p rustlensmanager
```

Manager mode reads profiles from the user config directory. See `docs/profiles.md`.

Run the GUI:

```bash
cargo run --bin rustlens-gui
```

During current development, `rustlens-core` depends on local `~/Projects/dbnest/crates/dbnest-core`. Ensure that checkout exists before building.

## Implementation Guidelines

- Keep fixes small and targeted.
- Keep DB access in `rustlens-core`; UI crates should use `DbCmd`/`DbEvt`.
- Keep near-term database scope Postgres-only.
- Do not add GUI-specific or TUI-specific types to shared application state.
- Treat credentials in local profile TOML as plaintext and document that clearly.
- Prefer recoverable `DbEvt::Error` over worker crashes.
- Potentially destructive SQL must be confirmed before execution in every UI.

## Planned GUI

The GUI binary will be named `rustlens-gui`. It should use `egui`/`eframe` with `egui_extras::TableBuilder` for high-density result grids.

opencode -s ses_1f840e9d7ffeoVL9VcNWG2ao0c
