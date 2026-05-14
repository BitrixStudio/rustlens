# Current Bugs

- Docker/testcontainers DB integration tests exist but are ignored by default and still need to be run in Docker-capable CI.
- Query cancellation/timeouts are not implemented for non-connect queries.
- Profile manager can load/open profiles but cannot add/edit/delete them.
- Profile TOML stores credentials in plaintext.
- TUI state still contains Ratatui widget state and needs extraction before GUI reuse.
- SQL safety classification is conservative and should eventually move to shared app/core code for GUI reuse.
