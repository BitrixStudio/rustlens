# AI Context

RustLens is an early-stage Rust Postgres database viewer. It currently has a terminal UI and will gain a native Rust GUI named `rustlens-gui`.

Near-term scope:

- Postgres only.
- Local TOML profiles in user config directories only.
- GUI stack: `egui`/`eframe`.
- Destructive SQL confirmation required in both TUI and GUI.
- Integration tests should use Docker/testcontainers.

Do not assume multi-driver support yet. Do not put UI framework types in shared app state.
