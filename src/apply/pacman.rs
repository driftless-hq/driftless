//! Arch Linux package management task executor
//!
//! Handles package installation, removal, and updates using pacman.
//!
//! # Examples
//!
//! ## Install a package
//!
//! This example installs the vim package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pacman
//!   description: "Install vim package"
//!   name: vim
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pacman",
//!   "description": "Install vim package",
//!   "name": "vim",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pacman"
//! description = "Install vim package"
//! name = "vim"
//! state = "present"
//! ```
//!
//! ## Install with cache update
//!
//! This example installs nginx and updates the package database first.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pacman
//!   description: "Install nginx with cache update"
//!   name: nginx
//!   state: present
//!   update_cache: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pacman",
//!   "description": "Install nginx with cache update",
//!   "name": "nginx",
//!   "state": "present",
//!   "update_cache": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pacman"
//! description = "Install nginx with cache update"
//! name = "nginx"
//! state = "present"
//! update_cache = true
//! ```
//!
//! ## Remove a package
//!
//! This example removes the vim package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pacman
//!   description: "Remove vim package"
//!   name: vim
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pacman",
//!   "description": "Remove vim package",
//!   "name": "vim",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pacman"
//! description = "Remove vim package"
//! name = "vim"
//! state = "absent"
//! ```
//!
//! ## Remove package with dependencies
//!
//! This example removes a package and its dependencies.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pacman
//!   description: "Remove package with dependencies"
//!   name: old-package
//!   state: absent
//!   remove_dependencies: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pacman",
//!   "description": "Remove package with dependencies",
//!   "name": "old-package",
//!   "state": "absent",
//!   "remove_dependencies": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pacman"
//! description = "Remove package with dependencies"
//! name = "old-package"
//! state = "absent"
//! remove_dependencies = true
//! ```

/// Arch Linux package management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacmanTask {
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
    /// Update package database
    #[serde(default)]
    pub update_cache: bool,
    /// Force installation/removal
    #[serde(default)]
    pub force: bool,
    /// Force reinstallation
    #[serde(default)]
    pub reinstall: bool,
    /// Remove dependencies
    #[serde(default)]
    pub remove_dependencies: bool,
    /// Remove configuration files
    #[serde(default)]
    pub remove_config: bool,
    /// Upgrade system
    #[serde(default)]
    pub upgrade: bool,
}

use serde::{Deserialize, Serialize};

use crate::apply::PackageState;
use anyhow::{Context, Result};
use std::process::Command;

/// Execute a pacman task
pub async fn execute_pacman_task(task: &PacmanTask, dry_run: bool) -> Result<()> {
    match task.state {
        PackageState::Present => ensure_package_present(task, dry_run).await,
        PackageState::Absent => ensure_package_absent(task, dry_run).await,
        PackageState::Latest => ensure_package_latest(task, dry_run).await,
    }
}

/// Ensure package is installed
async fn ensure_package_present(task: &PacmanTask, dry_run: bool) -> Result<()> {
    // Check if package is already installed
    let is_installed = is_package_installed(&task.name).unwrap_or_default();

    if is_installed {
        println!("Package {} is already installed", task.name);
        return Ok(());
    }

    // Update package database if requested
    if task.update_cache {
        update_cache(task, dry_run).await?;
    }

    if dry_run {
        println!("Would install package: {}", task.name);
        if task.force {
            println!("  (with force)");
        }
    } else {
        // Install package
        let mut args = vec!["-S".to_string(), "--noconfirm".to_string()];

        if task.force {
            args.push("--force".to_string());
        }

        if task.reinstall {
            args.push("--reinstall".to_string());
        }

        args.push(task.name.clone());

        run_pacman_command(&args)
            .await
            .with_context(|| format!("Failed to install package {}", task.name))?;

        println!("Installed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is removed
async fn ensure_package_absent(task: &PacmanTask, dry_run: bool) -> Result<()> {
    // Check if package is installed
    let is_installed = match is_package_installed(&task.name) {
        Ok(installed) => installed,
        Err(_) => {
            // If we can't check installation status, assume it's not installed for dry runs
            // or fail for real runs
            if dry_run {
                false
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot determine if package {} is installed",
                    task.name
                ));
            }
        }
    };

    if !is_installed {
        println!("Package {} is not installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove package: {}", task.name);
        if task.remove_dependencies {
            println!("  (removing dependencies)");
        }
        if task.remove_config {
            println!("  (removing config files)");
        }
    } else {
        // Remove package
        let mut args = vec!["-R".to_string(), "--noconfirm".to_string()];

        if task.remove_dependencies {
            args.push("--cascade".to_string());
        }

        if task.remove_config {
            args.push("--nosave".to_string());
        }

        if task.force {
            args.push("--force".to_string());
        }

        args.push(task.name.clone());

        run_pacman_command(&args)
            .await
            .with_context(|| format!("Failed to remove package {}", task.name))?;

        println!("Removed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is at latest version
async fn ensure_package_latest(task: &PacmanTask, dry_run: bool) -> Result<()> {
    // Update package database first
    update_cache(task, dry_run).await?;

    if dry_run {
        println!("Would upgrade package: {}", task.name);
        if task.upgrade {
            println!("  (system upgrade)");
        }
    } else if task.upgrade {
        // Full system upgrade
        run_pacman_command(&["-Syu".to_string(), "--noconfirm".to_string()])
            .await
            .with_context(|| "Failed to upgrade system")?;
        println!("Upgraded system");
    } else {
        // Upgrade specific package
        run_pacman_command(&[
            "-S".to_string(),
            "--noconfirm".to_string(),
            task.name.clone(),
        ])
        .await
        .with_context(|| format!("Failed to upgrade package {}", task.name))?;
        println!("Upgraded package: {}", task.name);
    }

    Ok(())
}

/// Update package database
async fn update_cache(_task: &PacmanTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would update package database");
    } else {
        run_pacman_command(&["-Sy".to_string()])
            .await
            .with_context(|| "Failed to update package database")?;
        println!("Updated package database");
    }

    Ok(())
}

/// Check if package is installed
fn is_package_installed(package_name: &str) -> Result<bool> {
    let output = Command::new("pacman")
        .args(["-Q", package_name])
        .output()
        .with_context(|| format!("Failed to check package status: {}", package_name))?;

    Ok(output.status.success())
}

/// Run pacman command with proper error handling
async fn run_pacman_command(args: &[String]) -> Result<()> {
    let output = Command::new("pacman")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run pacman command: {:?}", args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Pacman command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pacman_install_dry_run() {
        let task = PacmanTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Present,
            update_cache: false,
            force: false,
            reinstall: false,
            remove_dependencies: false,
            remove_config: false,
            upgrade: false,
        };

        let result = execute_pacman_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pacman_remove_dry_run() {
        let task = PacmanTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Absent,
            update_cache: false,
            force: true,
            reinstall: false,
            remove_dependencies: true,
            remove_config: true,
            upgrade: false,
        };

        let result = execute_pacman_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pacman_upgrade_dry_run() {
        let task = PacmanTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Latest,
            update_cache: true,
            force: false,
            reinstall: false,
            remove_dependencies: false,
            remove_config: false,
            upgrade: false,
        };

        let result = execute_pacman_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pacman_system_upgrade_dry_run() {
        let task = PacmanTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Latest,
            update_cache: true,
            force: false,
            reinstall: false,
            remove_dependencies: false,
            remove_config: false,
            upgrade: true,
        };

        let result = execute_pacman_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_package_installed() {
        // Test package installation check - may fail in test environments
        let result = is_package_installed("pacman");
        // Just ensure the function doesn't panic, result may be error if pacman not available
        let _ = result;
    }
}
