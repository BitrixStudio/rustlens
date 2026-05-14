# Agent Guidelines

- Prefer minimal correct changes.
- Preserve user changes in a dirty worktree.
- Keep near-term DB support Postgres-only.
- Keep credentials handling explicit; profile TOML is plaintext until keyring support exists.
- Do not add backward compatibility unless there is shipped/persisted behavior that requires it.
- Update docs when implementation milestones change behavior.
- If adding shared app logic, avoid Ratatui, egui, or crossterm types in that shared crate.
