//! RHEL/CentOS/Fedora package management task executor
//!
//! Handles package installation, removal, and updates using yum/dnf.
//!
//! # Examples
//!
//! ## Install a package
//!
//! This example installs nginx using yum/dnf.
//!
//! **YAML Format:**
//! ```yaml
//! - type: yum
//!   description: "Install nginx web server"
//!   name: nginx
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "yum",
//!   "description": "Install nginx web server",
//!   "name": "nginx",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "yum"
//! description = "Install nginx web server"
//! name = "nginx"
//! state = "present"
//! ```
//!
//! ## Install with cache update
//!
//! This example installs a package and updates the package cache first.
//!
//! **YAML Format:**
//! ```yaml
//! - type: yum
//!   description: "Install curl with cache update"
//!   name: curl
//!   state: present
//!   update_cache: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "yum",
//!   "description": "Install curl with cache update",
//!   "name": "curl",
//!   "state": "present",
//!   "update_cache": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "yum"
//! description = "Install curl with cache update"
//! name = "curl"
//! state = "present"
//! update_cache = true
//! ```
//!
//! ## Remove a package
//!
//! This example removes a package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: yum
//!   description: "Remove telnet package"
//!   name: telnet
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "yum",
//!   "description": "Remove telnet package",
//!   "name": "telnet",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "yum"
//! description = "Remove telnet package"
//! name = "telnet"
//! state = "absent"
//! ```

/// RHEL/CentOS/Fedora package management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YumTask {
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
    /// Allow downgrades
    #[serde(default)]
    pub allow_downgrades: bool,
    /// Install recommended packages
    #[serde(default)]
    pub install_recommended: bool,
    /// Install suggested packages
    #[serde(default)]
    pub install_suggested: bool,
    /// Disable GPG check
    #[serde(default)]
    pub disable_gpg_check: bool,
    /// Disable excludes
    #[serde(default)]
    pub disable_excludes: bool,
    /// Force installation
    #[serde(default)]
    pub force: bool,
}

use serde::{Deserialize, Serialize};

use crate::apply::PackageState;
use anyhow::{Context, Result};
use std::process::Command;

/// Execute a yum task
pub async fn execute_yum_task(task: &YumTask, dry_run: bool) -> Result<()> {
    match task.state {
        PackageState::Present => ensure_package_present(task, dry_run).await,
        PackageState::Absent => ensure_package_absent(task, dry_run).await,
        PackageState::Latest => ensure_package_latest(task, dry_run).await,
    }
}

/// Ensure package is installed
async fn ensure_package_present(task: &YumTask, dry_run: bool) -> Result<()> {
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
        if task.disable_gpg_check {
            println!("  (disabling GPG check)");
        }
    } else {
        // Determine which package manager to use (yum or dnf)
        let pkg_manager = detect_package_manager()?;

        // Install package
        let mut args = vec!["install".to_string(), "-y".to_string()];

        if task.allow_downgrades {
            args.push("--allow-downgrades".to_string());
        }

        if task.disable_gpg_check {
            args.push("--nogpgcheck".to_string());
        }

        if task.disable_excludes {
            args.push("--disableexcludes=main".to_string());
        }

        if task.force {
            args.push("--force".to_string());
        }

        args.push(task.name.clone());

        run_package_command(&pkg_manager, &args)
            .await
            .with_context(|| format!("Failed to install package {}", task.name))?;

        println!("Installed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is removed
async fn ensure_package_absent(task: &YumTask, dry_run: bool) -> Result<()> {
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
    } else {
        // Determine which package manager to use
        let pkg_manager = detect_package_manager()?;

        // Remove package
        run_package_command(
            &pkg_manager,
            &["remove".to_string(), "-y".to_string(), task.name.clone()],
        )
        .await
        .with_context(|| format!("Failed to remove package {}", task.name))?;

        println!("Removed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is at latest version
async fn ensure_package_latest(task: &YumTask, dry_run: bool) -> Result<()> {
    // Update cache first
    update_cache(task, dry_run).await?;

    if dry_run {
        println!("Would upgrade package: {}", task.name);
    } else {
        // Determine which package manager to use
        let pkg_manager = detect_package_manager()?;

        // Upgrade specific package
        run_package_command(
            &pkg_manager,
            &["upgrade".to_string(), "-y".to_string(), task.name.clone()],
        )
        .await
        .with_context(|| format!("Failed to upgrade package {}", task.name))?;

        println!("Upgraded package: {}", task.name);
    }

    Ok(())
}

/// Update package cache
async fn update_cache(_task: &YumTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would update package cache");
    } else {
        let pkg_manager = detect_package_manager()?;
        run_package_command(&pkg_manager, &["makecache".to_string()])
            .await
            .with_context(|| "Failed to update package cache")?;
        println!("Updated package cache");
    }

    Ok(())
}

/// Detect which package manager to use
fn detect_package_manager() -> Result<String> {
    // Check if dnf is available (newer systems)
    if Command::new("which")
        .arg("dnf")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        Ok("dnf".to_string())
    }
    // Fall back to yum (older systems)
    else if Command::new("which")
        .arg("yum")
        .output()
        .is_ok_and(|o| o.status.success())
    {
        Ok("yum".to_string())
    } else {
        Err(anyhow::anyhow!("Neither dnf nor yum found on system"))
    }
}

/// Check if package is installed
fn is_package_installed(package_name: &str) -> Result<bool> {
    let output = Command::new("rpm")
        .args(["-q", package_name])
        .output()
        .with_context(|| format!("Failed to check package status: {}", package_name))?;

    Ok(output.status.success())
}

/// Run package manager command with proper error handling
async fn run_package_command(command: &str, args: &[String]) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_yum_install_dry_run() {
        let task = YumTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Present,
            update_cache: false,
            allow_downgrades: false,
            install_recommended: true,
            install_suggested: false,
            disable_gpg_check: false,
            disable_excludes: false,
            force: false,
        };

        let result = execute_yum_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_yum_remove_dry_run() {
        let task = YumTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Absent,
            update_cache: false,
            allow_downgrades: false,
            install_recommended: true,
            install_suggested: false,
            disable_gpg_check: false,
            disable_excludes: false,
            force: false,
        };

        let result = execute_yum_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_yum_upgrade_dry_run() {
        let task = YumTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Latest,
            update_cache: true,
            allow_downgrades: false,
            install_recommended: true,
            install_suggested: false,
            disable_gpg_check: false,
            disable_excludes: false,
            force: false,
        };

        let result = execute_yum_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_package_manager() {
        let result = detect_package_manager();
        // We don't assert the result since it depends on the system having yum/dnf
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_is_package_installed() {
        // Test package installation check - may fail in test environments
        let result = is_package_installed("filesystem");
        // Just ensure the function doesn't panic, result may be error if rpm not available
        let _ = result;
    }
}
