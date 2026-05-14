# Installers

RustLens can show and launch installer commands after explicit confirmation.

## Supported Package Managers

- `apt`
- `dnf`
- `pacman`
- `brew`
- `winget`

## Safety Rules

- Show exact commands before launch.
- Require explicit confirmation.
- Prefer external terminal launch for privileged commands.
- Never capture or store sudo/admin passwords.
- Do not run arbitrary user-provided commands.

## Example Arch Commands

PostgreSQL:

```bash
sudo pacman -S postgresql
```

Docker:

```bash
sudo pacman -S docker
sudo systemctl enable --now docker
sudo usermod -aG docker "$USER"
```

SQLite:

```bash
sudo pacman -S sqlite
```
