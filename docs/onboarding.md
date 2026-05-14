# Onboarding

`rustlens-gui` opens a setup wizard when no profiles exist in the user config profile file.

The wizard provides three paths:

- Add an existing PostgreSQL database.
- Create a local database.
- Install database tools.

## Add Existing Database

The first supported connection form is PostgreSQL.

Users can enter host, port, database, user, password, schema, and page size, or switch to advanced connection URL mode. The profile can be saved or saved and connected immediately.

## Create Local Database

The GUI integrates with `dbnest-core` for local database provisioning.

Currently supported creation paths:

- PostgreSQL via Docker.
- SQLite embedded file creation.

After PostgreSQL Docker creation succeeds, RustLens saves the dbnest-backed profile, connects to it, and switches to Schema Studio so the next step is designing and applying the starter schema.

SQLite profiles can be created and saved, but RustLens browsing for SQLite is not implemented yet. The UI must keep this warning visible anywhere SQLite creation is offered.

MySQL/MariaDB is shown as planned but disabled until both dbnest provisioning and RustLens query support exist.

## Install Tools

The GUI detects local capabilities and supported package managers. Installer commands are shown before execution and require explicit user confirmation.

RustLens should not capture sudo passwords. Privileged commands should be launched in an external terminal when possible.
