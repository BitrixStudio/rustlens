# Schema Studio

Schema Studio is the GUI-first visual schema designer for RustLens.

## Current Scope

The first implementation is create-oriented and mirrors dbnest's current schema capabilities:

- Tables
- Columns
- Logical types: `string`, `int64`, `bool`, `uuid`, `timestamp`
- Nullable, primary key, and unique flags
- Simple defaults such as `now`, literals, and raw SQL-like function calls
- Live PostgreSQL SQL preview
- Confirmed apply to the active PostgreSQL connection/schema
- Starter templates: Users, Content, and Tasks

Apply currently sends the generated `CREATE TABLE IF NOT EXISTS` statements to the GUI database worker as one batch. PostgreSQL runs the batch in a transaction, so a failing statement rolls back the batch and reports the failing statement number. On success, RustLens reloads schema metadata afterward.

## UX

- Left column: table list and add/delete table actions.
- Middle column: selected table and column editor.
- Right column: validation status and generated SQL preview.
- Top actions: choose a starter template, replace the current draft, and `Apply schema` when connected and valid.

The default template creates a practical `users` table so newly provisioned databases do not start from a completely blank design surface. The Content and Tasks templates provide quick alternatives for common app shapes.

## Planned Next

- Apply generated schema to dbnest-managed PostgreSQL instances.
- Add starter template selection during the create-local flow.
- Add index editor.
- Add foreign keys when dbnest supports them.
- Add migration/diff planning later.
