# rustlens

A lightweight, terminal-based database viewer written in Rust.

`rustlens` aims to provide a fast, minimal, and stable alternative to heavyweight GUI database tools. It focuses on browsing tables and executing SQL from the terminal with a simple and predictable interface.

> ⚠️ This project is early-stage and evolving.

---

## Binaries

This workspace provides two commands:

### `rustlens`

Direct database viewer. Connects to a database via connection string.
<img width="799" height="368" alt="image" src="https://github.com/user-attachments/assets/57fc6b37-5658-495c-be81-88d7ad5f5f08" />

### `rustlensmanager`

Connection manager. Intended to manage saved database profiles and launch viewer sessions.

*(Manager functionality is currently minimal and will evolve.)*

---

## Requirements

- Rust (stable toolchain)
- PostgreSQL (currently only Postgres is supported)

---

## Running

All commands should be executed from the workspace root.

---

### Run with a connection string (no config file required)

```bash
cargo run -p rustlens -- postgres://app:app@localhost:5432/appdb
```

This will:

- Connect to the provided database
- Load tables from the default schema (`public`)
- Use a default page size of `200`

---

### Run using a config file

Place a `config.toml` file in the workspace root:

```toml
database_url = "postgres://app:app@localhost:5432/appdb"
schema = "public"
page_size = 200
```

Then run:

```bash
cargo run -p rustlensmanager
```

Manager mode uses the config file.

You may also create a `config-dev.toml` for local development.  
In debug builds, if `config-dev.toml` exists, it will be preferred over `config.toml`.

---

## User Controls (current)

### General

| Key        | Action        |
|------------|--------------|
| `F2`       | Browse tab    |
| `F3`       | SQL tab       |
| `Tab`      | Switch focus  |
| `q`        | Quit          |

---

### Browse Tab

| Key                          | Action                |
|------------------------------|-----------------------|
| `j` / `k` or arrow keys     | Navigate              |
| `Enter`                      | Open table            |
| `r`                          | Refresh tables        |
| `[` / `]`                    | Page backward/forward |

---

### SQL Tab

| Key             | Action          |
|----------------|-----------------|
| Type            | Edit SQL        |
| `Ctrl+F5` / `Ctrl+Enter` | Execute SQL     |
| `Enter`         | Insert newline  |

---

## Project Structure

This is a Cargo workspace:

```
crates/
  rustlens-core/   # DB logic and shared models
  rustlens-tui/    # Terminal UI and app logic

apps/
  rustlens/        # Viewer binary
  rustlensmanager/ # Manager binary
```

- `rustlens-core` contains database worker and Postgres logic.
- `rustlens-tui` contains state management, reducer, and UI.
- The binaries are thin entrypoints.

---

## License

MIT
