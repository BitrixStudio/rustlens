# Profiles

Saved connection profiles should be stored in the user's config directory only.

Expected locations:

- Linux: `~/.config/rustlens/profiles.toml`
- macOS: `~/Library/Application Support/rustlens/profiles.toml`
- Windows: `%APPDATA%\rustlens\profiles.toml`

## Initial Format

```toml
[[profiles]]
id = "00000000-0000-0000-0000-000000000000"
name = "local"
driver = "postgres"
database_url = "postgres://app:app@localhost:5432/appdb"
schema = "public"
page_size = 200
source = "manual"
dbnest_instance_id = ""
```

`id`, `schema`, `page_size`, `source`, and `dbnest_instance_id` are optional when hand-writing profiles. Missing IDs are generated when profiles are parsed.

Supported driver values in the profile model:

- `postgres`: browsable now.
- `sqlite`: profile creation supported through dbnest; browsing planned.
- `mysql`: planned.
- `mariadb`: planned.

Supported source values:

- `manual`
- `dbnest`
- `imported`

## Security Note

The initial TOML storage is plaintext. If `database_url` includes a password, that password is stored plainly on disk. A later milestone should add OS keyring-backed credential storage.

## Current UI Support

`rustlensmanager` loads PostgreSQL profiles from this file and can open selected profiles. `rustlens-gui` can create profiles through the setup wizard, edit saved profile name/URL/schema/page size from the sidebar, and delete profiles after confirmation.

Deleting a dbnest-backed profile only removes the RustLens profile entry. It does not remove the dbnest instance, Docker container, volume, or SQLite file.

Editing a dbnest-backed profile only changes RustLens metadata. It does not modify the dbnest instance. Reconnect after changing a connection URL or credentials.
