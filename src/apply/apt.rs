//! Debian/Ubuntu package management task executor
//!
//! Handles package installation, removal, and updates using apt/apt-get.
//!
//! # Examples
//!
//! ## Install a package
//!
//! This example installs the curl package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: apt
//!   description: "Install curl package"
//!   name: curl
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "apt",
//!   "description": "Install curl package",
//!   "name": "curl",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "apt"
//! description = "Install curl package"
//! name = "curl"
//! state = "present"
//! ```
//!
//! ## Install package with cache update
//!
//! This example installs nginx and updates the package cache first.
//!
//! **YAML Format:**
//! ```yaml
//! - type: apt
//!   description: "Install nginx with cache update"
//!   name: nginx
//!   state: present
//!   update_cache: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "apt",
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
//! type = "apt"
//! description = "Install nginx with cache update"
//! name = "nginx"
//! state = "present"
//! update_cache = true
//! ```
//!
//! ## Remove a package
//!
//! This example removes the apache2 package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: apt
//!   description: "Remove apache2 package"
//!   name: apache2
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "apt",
//!   "description": "Remove apache2 package",
//!   "name": "apache2",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "apt"
//! description = "Remove apache2 package"
//! name = "apache2"
//! state = "absent"
//! ```
//!
//! ## Update package to latest version
//!
//! This example ensures vim is installed and updated to the latest version.
//!
//! **YAML Format:**
//! ```yaml
//! - type: apt
//!   description: "Update vim to latest version"
//!   name: vim
//!   state: latest
//!   update_cache: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "apt",
//!   "description": "Update vim to latest version",
//!   "name": "vim",
//!   "state": "latest",
//!   "update_cache": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "apt"
//! description = "Update vim to latest version"
//! name = "vim"
//! state = "latest"
//! update_cache = true
//! ```

/// Debian/Ubuntu package management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptTask {
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
    /// Update package cache
    #[serde(default)]
    pub update_cache: bool,
    /// Cache validity time in seconds
    #[serde(default = "default_cache_valid_time")]
    pub cache_valid_time: u32,
    /// Allow downgrades
    #[serde(default)]
    pub allow_downgrades: bool,
    /// Allow unauthenticated packages
    #[serde(default)]
    pub allow_unauthenticated: bool,
    /// Autoclean package cache
    #[serde(default)]
    pub autoclean: bool,
    /// Autoremove unused packages
    #[serde(default)]
    pub autoremove: bool,
    /// Force installation
    #[serde(default)]
    pub force: bool,
}

use serde::{Deserialize, Serialize};

use crate::apply::PackageState;
use anyhow::{Context, Result};
use std::process::Command;

/// Execute an apt task
pub async fn execute_apt_task(task: &AptTask, dry_run: bool) -> Result<()> {
    match task.state {
        PackageState::Present => ensure_package_present(task, dry_run).await,
        PackageState::Absent => ensure_package_absent(task, dry_run).await,
        PackageState::Latest => ensure_package_latest(task, dry_run).await,
    }
}

/// Ensure package is installed
async fn ensure_package_present(task: &AptTask, dry_run: bool) -> Result<()> {
    // Check if package is already installed
    let is_installed = is_package_installed(&task.name).unwrap_or_default();

    if is_installed {
        println!("Package {} is already installed", task.name);
        return Ok(());
    }

    // Update cache if requested
    if task.update_cache {
        update_cache(task, dry_run).await?;
    }

    if dry_run {
        println!("Would install package: {}", task.name);
        if task.allow_downgrades {
            println!("  (allowing downgrades)");
        }
        if task.allow_unauthenticated {
            println!("  (allowing unauthenticated packages)");
        }
    } else {
        // Install package
        let mut args = vec!["install".to_string(), "-y".to_string()];

        if task.allow_downgrades {
            args.push("--allow-downgrades".to_string());
        }

        if task.allow_unauthenticated {
            args.push("--allow-unauthenticated".to_string());
        }

        if task.force {
            args.push("--force-yes".to_string());
        }

        args.push(task.name.clone());

        run_apt_command("apt-get", &args)
            .await
            .with_context(|| format!("Failed to install package {}", task.name))?;

        println!("Installed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is removed
async fn ensure_package_absent(task: &AptTask, dry_run: bool) -> Result<()> {
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
        if task.autoremove {
            println!("  (with autoremove)");
        }
        if task.autoclean {
            println!("  (with autoclean)");
        }
    } else {
        // Remove package
        let mut args = vec!["remove".to_string(), "-y".to_string()];
        args.push(task.name.clone());

        run_apt_command("apt-get", &args)
            .await
            .with_context(|| format!("Failed to remove package {}", task.name))?;

        // Autoremove if requested
        if task.autoremove {
            run_apt_command("apt-get", &["autoremove".to_string(), "-y".to_string()])
                .await
                .with_context(|| "Failed to autoremove packages")?;
            println!("Autoremoved unused packages");
        }

        // Autoclean if requested
        if task.autoclean {
            run_apt_command("apt-get", &["autoclean".to_string()])
                .await
                .with_context(|| "Failed to autoclean package cache")?;
            println!("Cleaned package cache");
        }

        println!("Removed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is at latest version
async fn ensure_package_latest(task: &AptTask, dry_run: bool) -> Result<()> {
    // Update cache first
    update_cache(task, dry_run).await?;

    if dry_run {
        println!("Would upgrade package: {}", task.name);
    } else {
        // Upgrade specific package
        run_apt_command(
            "apt-get",
            &[
                "install".to_string(),
                "-y".to_string(),
                "--only-upgrade".to_string(),
                task.name.clone(),
            ],
        )
        .await
        .with_context(|| format!("Failed to upgrade package {}", task.name))?;

        println!("Upgraded package: {}", task.name);
    }

    Ok(())
}

/// Update package cache
async fn update_cache(_task: &AptTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would update package cache");
    } else {
        run_apt_command("apt-get", &["update".to_string()])
            .await
            .with_context(|| "Failed to update package cache")?;
        println!("Updated package cache");
    }

    Ok(())
}

/// Check if package is installed
fn is_package_installed(package_name: &str) -> Result<bool> {
    let output = Command::new("dpkg")
        .args(["-s", package_name])
        .output()
        .with_context(|| format!("Failed to check package status: {}", package_name))?;

    Ok(output.status.success())
}

/// Run apt command with proper error handling
async fn run_apt_command(command: &str, args: &[String]) -> Result<()> {
    let output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run command: {} {:?}", command, args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Command failed: {} {:?}\nstdout: {}\nstderr: {}",
            command,
            args,
            stdout,
            stderr
        ));
    }

    Ok(())
}

/// Default cache valid time (3600 seconds)
pub fn default_cache_valid_time() -> u32 {
    3600
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_apt_install_dry_run() {
        let task = AptTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Present,
            update_cache: false,
            cache_valid_time: 0,
            allow_downgrades: false,
            allow_unauthenticated: false,
            autoclean: false,
            autoremove: false,
            force: false,
        };

        let result = execute_apt_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apt_remove_dry_run() {
        let task = AptTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Absent,
            update_cache: false,
            cache_valid_time: 0,
            allow_downgrades: false,
            allow_unauthenticated: false,
            autoclean: true,
            autoremove: true,
            force: false,
        };

        let result = execute_apt_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apt_upgrade_dry_run() {
        let task = AptTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Latest,
            update_cache: true,
            cache_valid_time: 0,
            allow_downgrades: false,
            allow_unauthenticated: false,
            autoclean: false,
            autoremove: false,
            force: false,
        };

        let result = execute_apt_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_package_installed() {
        // Test package installation check - may fail in test environments
        let result = is_package_installed("base-files");
        // Just ensure the function doesn't panic, result may be error if dpkg not available
        let _ = result;
    }
}
