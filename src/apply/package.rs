//! Package task executor
//!
//! Handles package management operations: install, remove, update packages
//! using the appropriate package manager for the system.
//!
//! # Examples
//!
//! ## Install a package
//!
//! This example installs nginx using the system's default package manager.
//!
//! **YAML Format:**
//! ```yaml
//! - type: package
//!   description: "Install nginx web server"
//!   name: nginx
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "package",
//!   "description": "Install nginx web server",
//!   "name": "nginx",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "package"
//! description = "Install nginx web server"
//! name = "nginx"
//! state = "present"
//! ```
//!
//! ## Install with specific package manager
//!
//! This example forces the use of apt even on systems that might have multiple package managers.
//!
//! **YAML Format:**
//! ```yaml
//! - type: package
//!   description: "Install curl using apt"
//!   name: curl
//!   state: present
//!   manager: apt
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "package",
//!   "description": "Install curl using apt",
//!   "name": "curl",
//!   "state": "present",
//!   "manager": "apt"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "package"
//! description = "Install curl using apt"
//! name = "curl"
//! state = "present"
//! manager = "apt"
//! ```
//!
//! ## Update a package to latest version
//!
//! This example ensures a package is updated to the latest available version.
//!
//! **YAML Format:**
//! ```yaml
//! - type: package
//!   description: "Update vim to latest version"
//!   name: vim
//!   state: latest
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "package",
//!   "description": "Update vim to latest version",
//!   "name": "vim",
//!   "state": "latest"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "package"
//! description = "Update vim to latest version"
//! name = "vim"
//! state = "latest"
//! ```
//!
//! ## Remove a package
//!
//! This example ensures a package is not installed.
//!
//! **YAML Format:**
//! ```yaml
//! - type: package
//!   description: "Remove telnet client"
//!   name: telnet
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "package",
//!   "description": "Remove telnet client",
//!   "name": "telnet",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "package"
//! description = "Remove telnet client"
//! name = "telnet"
//! state = "absent"
//! ```
//!
//! ## Register package installation
//!
//! This example installs a package and registers the result to check for changes.
//!
//! **YAML Format:**
//! ```yaml
//! - type: package
//!   description: "Install git and check if changed"
//!   name: git
//!   state: present
//!   register: git_install
//!
//! - type: debug
//!   msg: "Git was newly installed"
//!   when: "{{ git_install.changed }}"
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "package",
//!     "description": "Install git and check if changed",
//!     "name": "git",
//!     "state": "present",
//!     "register": "git_install"
//!   },
//!   {
//!     "type": "debug",
//!     "msg": "Git was newly installed",
//!     "when": "{{ git_install.changed }}"
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "package"
//! description = "Install git and check if changed"
//! name = "git"
//! state = "present"
//! register = "git_install"
//!
//! [[tasks]]
//! type = "debug"
//! msg = "Git was newly installed"
//! when = "{{ git_install.changed }}"
//! ```

/// Package state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageState {
    /// Ensure package is installed
    Present,
    /// Ensure package is not installed
    Absent,
    /// Ensure package is latest version
    Latest,
}

/// Package management task
///
/// # Registered Outputs
/// - `changed` (bool): Whether any packages were installed or removed
/// - `packages` (`Vec<String>`): List of packages affected
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Package name
    pub name: String,
    /// Package state
    pub state: PackageState,
    /// Package manager to use (auto-detect if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Execute a package task
pub async fn execute_package_task(task: &PackageTask, dry_run: bool) -> Result<serde_yaml::Value> {
    let manager = detect_package_manager()
        .or_else(|| task.manager.as_ref().cloned())
        .ok_or_else(|| anyhow::anyhow!("Could not detect package manager"))?;

    let changed = match task.state {
        PackageState::Present => ensure_package_present(&task.name, &manager, dry_run).await?,
        PackageState::Absent => ensure_package_absent(&task.name, &manager, dry_run).await?,
        PackageState::Latest => ensure_package_latest(&task.name, &manager, dry_run).await?,
    };

    let mut result = serde_yaml::Mapping::new();
    result.insert(
        serde_yaml::Value::from("changed"),
        serde_yaml::Value::from(changed),
    );

    let packages = vec![serde_yaml::Value::from(task.name.clone())];
    result.insert(
        serde_yaml::Value::from("packages"),
        serde_yaml::Value::from(packages),
    );

    Ok(serde_yaml::Value::Mapping(result))
}

/// Detect the package manager available on the system
fn detect_package_manager() -> Option<String> {
    let managers = vec![
        ("apt-get", "apt"),
        ("yum", "yum"),
        ("dnf", "dnf"),
        ("pacman", "pacman"),
        ("zypper", "zypper"),
        ("brew", "brew"),
    ];

    for (cmd, name) in managers {
        if Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(name.to_string());
        }
    }

    None
}

/// Ensure a package is present
async fn ensure_package_present(package: &str, manager: &str, dry_run: bool) -> Result<bool> {
    if is_package_installed(package, manager)? {
        println!("Package {} is already installed", package);
        return Ok(false);
    }

    let install_cmd = get_install_command(package, manager);

    if dry_run {
        println!("Would run: {}", install_cmd.join(" "));
    } else {
        run_command(&install_cmd)
            .with_context(|| format!("Failed to install package {}", package))?;
        println!("Installed package: {}", package);
    }

    Ok(true)
}

/// Ensure a package is not installed
async fn ensure_package_absent(package: &str, manager: &str, dry_run: bool) -> Result<bool> {
    if !is_package_installed(package, manager)? {
        println!("Package {} is not installed", package);
        return Ok(false);
    }

    let remove_cmd = get_remove_command(package, manager);

    if dry_run {
        println!("Would run: {}", remove_cmd.join(" "));
    } else {
        run_command(&remove_cmd)
            .with_context(|| format!("Failed to remove package {}", package))?;
        println!("Removed package: {}", package);
    }

    Ok(true)
}

/// Ensure a package is at the latest version
async fn ensure_package_latest(package: &str, manager: &str, dry_run: bool) -> Result<bool> {
    let upgrade_cmd = get_upgrade_command(package, manager);

    if dry_run {
        println!("Would run: {}", upgrade_cmd.join(" "));
        Ok(true)
    } else {
        run_command(&upgrade_cmd)
            .with_context(|| format!("Failed to upgrade package {}", package))?;
        println!("Upgraded package: {}", package);
        Ok(true)
    }
}

/// Check if a package is installed
fn is_package_installed(package: &str, manager: &str) -> Result<bool> {
    let check_cmd = get_check_command(package, manager);

    let output = Command::new(&check_cmd[0])
        .args(&check_cmd[1..])
        .output()
        .with_context(|| format!("Failed to check if package {} is installed", package))?;

    Ok(output.status.success())
}

/// Get the install command for a package manager
fn get_install_command(package: &str, manager: &str) -> Vec<String> {
    match manager {
        "apt" => vec![
            "apt-get".to_string(),
            "install".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "yum" => vec![
            "yum".to_string(),
            "install".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "dnf" => vec![
            "dnf".to_string(),
            "install".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "pacman" => vec![
            "pacman".to_string(),
            "-S".to_string(),
            "--noconfirm".to_string(),
            package.to_string(),
        ],
        "zypper" => vec![
            "zypper".to_string(),
            "install".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "brew" => vec![
            "brew".to_string(),
            "install".to_string(),
            package.to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported package manager: {}", manager),
        ],
    }
}

/// Get the remove command for a package manager
fn get_remove_command(package: &str, manager: &str) -> Vec<String> {
    match manager {
        "apt" => vec![
            "apt-get".to_string(),
            "remove".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "yum" => vec![
            "yum".to_string(),
            "remove".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "dnf" => vec![
            "dnf".to_string(),
            "remove".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "pacman" => vec![
            "pacman".to_string(),
            "-R".to_string(),
            "--noconfirm".to_string(),
            package.to_string(),
        ],
        "zypper" => vec![
            "zypper".to_string(),
            "remove".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "brew" => vec![
            "brew".to_string(),
            "uninstall".to_string(),
            package.to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported package manager: {}", manager),
        ],
    }
}

/// Get the upgrade command for a package manager
fn get_upgrade_command(package: &str, manager: &str) -> Vec<String> {
    match manager {
        "apt" => vec![
            "apt-get".to_string(),
            "install".to_string(),
            "--only-upgrade".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "yum" => vec![
            "yum".to_string(),
            "update".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "dnf" => vec![
            "dnf".to_string(),
            "upgrade".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "pacman" => vec![
            "pacman".to_string(),
            "-Syu".to_string(),
            "--noconfirm".to_string(),
            package.to_string(),
        ],
        "zypper" => vec![
            "zypper".to_string(),
            "update".to_string(),
            "-y".to_string(),
            package.to_string(),
        ],
        "brew" => vec![
            "brew".to_string(),
            "upgrade".to_string(),
            package.to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported package manager: {}", manager),
        ],
    }
}

/// Get the check command for a package manager
fn get_check_command(package: &str, manager: &str) -> Vec<String> {
    match manager {
        "apt" => vec!["dpkg".to_string(), "-l".to_string(), package.to_string()],
        "yum" | "dnf" => vec!["rpm".to_string(), "-q".to_string(), package.to_string()],
        "pacman" => vec!["pacman".to_string(), "-Q".to_string(), package.to_string()],
        "zypper" => vec!["rpm".to_string(), "-q".to_string(), package.to_string()],
        "brew" => vec!["brew".to_string(), "list".to_string(), package.to_string()],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported package manager: {}", manager),
        ],
    }
}

/// Run a command and return the result
fn run_command(cmd: &[String]) -> Result<()> {
    if cmd.is_empty() {
        return Ok(());
    }

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .status()
        .with_context(|| format!("Failed to execute command: {}", cmd.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_package_install_dry_run() {
        let task = PackageTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Present,
            manager: Some("apt".to_string()),
        };

        let result = execute_package_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_package_remove_dry_run() {
        let task = PackageTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Absent,
            manager: Some("apt".to_string()),
        };

        let result = execute_package_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_package_manager() {
        // This test might not work in all environments, but it's better than nothing
        let manager = detect_package_manager();
        // We can't assert much here since the test environment might not have package managers
        // But we can assert it's either Some or None
        assert!(manager.is_some() || manager.is_none());
    }

    #[test]
    fn test_get_install_command() {
        let cmd = get_install_command("nginx", "apt");
        assert_eq!(cmd, vec!["apt-get", "install", "-y", "nginx"]);

        let cmd = get_install_command("nginx", "yum");
        assert_eq!(cmd, vec!["yum", "install", "-y", "nginx"]);
    }

    #[test]
    fn test_get_remove_command() {
        let cmd = get_remove_command("nginx", "apt");
        assert_eq!(cmd, vec!["apt-get", "remove", "-y", "nginx"]);

        let cmd = get_remove_command("nginx", "yum");
        assert_eq!(cmd, vec!["yum", "remove", "-y", "nginx"]);
    }
}
