use crate::provision::capabilities::PackageManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    Postgres,
    Docker,
    Sqlite,
    Mysql,
}

pub fn install_commands(manager: PackageManager, target: InstallTarget) -> Vec<String> {
    match (manager, target) {
        (PackageManager::Pacman, InstallTarget::Postgres) => {
            vec!["sudo pacman -S postgresql".into()]
        }
        (PackageManager::Pacman, InstallTarget::Docker) => vec![
            "sudo pacman -S docker".into(),
            "sudo systemctl enable --now docker".into(),
            "sudo usermod -aG docker \"$USER\"".into(),
        ],
        (PackageManager::Pacman, InstallTarget::Sqlite) => vec!["sudo pacman -S sqlite".into()],
        (PackageManager::Pacman, InstallTarget::Mysql) => vec!["sudo pacman -S mariadb".into()],

        (PackageManager::Apt, InstallTarget::Postgres) => {
            vec!["sudo apt install postgresql postgresql-client".into()]
        }
        (PackageManager::Apt, InstallTarget::Docker) => vec!["sudo apt install docker.io".into()],
        (PackageManager::Apt, InstallTarget::Sqlite) => vec!["sudo apt install sqlite3".into()],
        (PackageManager::Apt, InstallTarget::Mysql) => {
            vec!["sudo apt install mariadb-server mariadb-client".into()]
        }

        (PackageManager::Dnf, InstallTarget::Postgres) => {
            vec!["sudo dnf install postgresql postgresql-server".into()]
        }
        (PackageManager::Dnf, InstallTarget::Docker) => vec!["sudo dnf install docker".into()],
        (PackageManager::Dnf, InstallTarget::Sqlite) => vec!["sudo dnf install sqlite".into()],
        (PackageManager::Dnf, InstallTarget::Mysql) => {
            vec!["sudo dnf install mariadb-server mariadb".into()]
        }

        (PackageManager::Brew, InstallTarget::Postgres) => {
            vec!["brew install postgresql@16".into()]
        }
        (PackageManager::Brew, InstallTarget::Docker) => vec!["brew install --cask docker".into()],
        (PackageManager::Brew, InstallTarget::Sqlite) => vec!["brew install sqlite".into()],
        (PackageManager::Brew, InstallTarget::Mysql) => vec!["brew install mariadb".into()],

        (PackageManager::Winget, InstallTarget::Postgres) => {
            vec!["winget install PostgreSQL.PostgreSQL".into()]
        }
        (PackageManager::Winget, InstallTarget::Docker) => {
            vec!["winget install Docker.DockerDesktop".into()]
        }
        (PackageManager::Winget, InstallTarget::Sqlite) => {
            vec!["winget install SQLite.SQLite".into()]
        }
        (PackageManager::Winget, InstallTarget::Mysql) => {
            vec!["winget install MariaDB.Server".into()]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{install_commands, InstallTarget};
    use crate::provision::capabilities::PackageManager;

    #[test]
    fn returns_arch_docker_install_sequence() {
        let commands = install_commands(PackageManager::Pacman, InstallTarget::Docker);
        assert!(commands.iter().any(|cmd| cmd.contains("pacman -S docker")));
        assert!(commands.iter().any(|cmd| cmd.contains("systemctl enable")));
    }
}
