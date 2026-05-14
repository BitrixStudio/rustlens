# Known Issues

## Critical

- No DB integration tests exist yet.
- Query cancellation/timeouts are not implemented for non-connect queries.
- SQL safety classification is conservative but not a full SQL parser.

## Product Gaps

- No add/edit/delete profile UI.
- No query cancellation.
- No persistent page/result cache.
- No GUI app yet.
- No OS keyring-backed credential storage.

## Architecture Gaps

- TUI state contains Ratatui-specific widget state and should not be reused by the GUI.
- Reducer mixes app transitions, DB orchestration, editor behavior, manager behavior, and fallback policy.
- DB protocol is string/result-vector oriented and should evolve toward typed result metadata.
