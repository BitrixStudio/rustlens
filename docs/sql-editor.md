# SQL Editor

The GUI SQL editor now has a RustLens-specific highlighted editor and metadata-aware completion.

## Highlighting

The first implementation uses a lightweight tokenizer rather than a full SQL parser. It highlights:

- SQL keywords
- String literals
- Numeric literals
- Line comments
- Identifiers and operators

The highlighter is intentionally dependency-light and lives in `crates/rustlens-gui/src/sql_editor.rs`.

## Completion

Completion logic is shared through `rustlens-core/src/sql/completion.rs` and is used by both the GUI and TUI.

Current completion sources:

- SQL keywords
- Loaded table names
- Loaded column names for `table.` qualifiers
- Initial snippets such as `SELECT * FROM`

GUI behavior:

- Completion appears below the editor while focused.
- `ArrowUp` / `ArrowDown` changes selection.
- `Tab` accepts the selected completion.
- `Esc` closes completion.

Future improvements:

- Cursor-positioned popup.
- Alias-aware completion.
- Function completion.
- More snippets.

## Safety

Destructive or mutating SQL detection is shared through `rustlens-core/src/sql/safety.rs`. Both the TUI and GUI use the same conservative keyword scanner and require explicit confirmation before sending matching statements to the DB worker.
