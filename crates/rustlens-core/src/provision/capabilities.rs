use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    Available { path: PathBuf },
    Missing,
    PresentButUnavailable { path: PathBuf, reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Brew,
    Winget,
}

#[derive(Debug, Clone)]
pub struct SystemCapabilities {
    pub package_manager: Option<PackageManager>,
    pub docker: ToolStatus,
    pub postgres_client: ToolStatus,
    pub postgres_createdb: ToolStatus,
    pub postgres_server: ToolStatus,
    pub sqlite_cli: ToolStatus,
    pub mysql_client: ToolStatus,
}

pub fn detect_system_capabilities() -> SystemCapabilities {
    let docker = detect_docker();
    SystemCapabilities {
        package_manager: detect_package_manager(),
        docker,
        postgres_client: detect_tool("psql"),
        postgres_createdb: detect_tool("createdb"),
        postgres_server: detect_tool("postgres"),
        sqlite_cli: detect_tool("sqlite3"),
        mysql_client: detect_tool("mysql"),
    }
}

pub fn detect_package_manager() -> Option<PackageManager> {
    if command_path("apt").is_some() {
        Some(PackageManager::Apt)
    } else if command_path("dnf").is_some() {
        Some(PackageManager::Dnf)
    } else if command_path("pacman").is_some() {
        Some(PackageManager::Pacman)
    } else if command_path("brew").is_some() {
        Some(PackageManager::Brew)
    } else if command_path("winget").is_some() {
        Some(PackageManager::Winget)
    } else {
        None
    }
}

pub fn detect_tool(name: &str) -> ToolStatus {
    command_path(name)
        .map(|path| ToolStatus::Available { path })
        .unwrap_or(ToolStatus::Missing)
}

fn detect_docker() -> ToolStatus {
    let Some(path) = command_path("docker") else {
        return ToolStatus::Missing;
    };

    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => ToolStatus::Available { path },
        Ok(output) => ToolStatus::PresentButUnavailable {
            path,
            reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        },
        Err(err) => ToolStatus::PresentButUnavailable {
            path,
            reason: err.to_string(),
        },
    }
}

fn command_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }

        #[cfg(windows)]
        {
            let candidate = dir.join(format!("{name}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::PackageManager;

    #[test]
    fn package_manager_is_debuggable() {
        assert_eq!(format!("{:?}", PackageManager::Pacman), "Pacman");
    }
}
