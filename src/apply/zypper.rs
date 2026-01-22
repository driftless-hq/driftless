//! SUSE package management task executor
//!
//! Handles package installation, removal, and updates using zypper.
//!
//! # Examples
//!
//! ## Install a package
//!
//! This example installs apache2 using zypper.
//!
//! **YAML Format:**
//! ```yaml
//! - type: zypper
//!   description: "Install apache web server"
//!   name: apache2
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "zypper",
//!   "description": "Install apache web server",
//!   "name": "apache2",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "zypper"
//! description = "Install apache web server"
//! name = "apache2"
//! state = "present"
//! ```
//!
//! ## Install with cache update
//!
//! This example installs a package and refreshes the repository metadata first.
//!
//! **YAML Format:**
//! ```yaml
//! - type: zypper
//!   description: "Install vim with repository refresh"
//!   name: vim
//!   state: present
//!   update_cache: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "zypper",
//!   "description": "Install vim with repository refresh",
//!   "name": "vim",
//!   "state": "present",
//!   "update_cache": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "zypper"
//! description = "Install vim with repository refresh"
//! name = "vim"
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
//! - type: zypper
//!   description: "Remove telnet package"
//!   name: telnet
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "zypper",
//!   "description": "Remove telnet package",
//!   "name": "telnet",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "zypper"
//! description = "Remove telnet package"
//! name = "telnet"
//! state = "absent"
//! ```

/// SUSE package management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZypperTask {
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
    /// Allow vendor changes
    #[serde(default)]
    pub allow_vendor_change: bool,
    /// Allow downgrades
    #[serde(default)]
    pub allow_downgrades: bool,
    /// Disable GPG check
    #[serde(default)]
    pub disable_gpg_check: bool,
    /// Force installation
    #[serde(default)]
    pub force: bool,
}

use serde::{Deserialize, Serialize};

use crate::apply::PackageState;
use anyhow::{Context, Result};
use std::process::Command;

/// Execute a zypper task
pub async fn execute_zypper_task(task: &ZypperTask, dry_run: bool) -> Result<()> {
    match task.state {
        PackageState::Present => {
            ensure_package_present(task, dry_run).await
        }
        PackageState::Absent => {
            ensure_package_absent(task, dry_run).await
        }
        PackageState::Latest => {
            ensure_package_latest(task, dry_run).await
        }
    }
}

/// Ensure package is installed
async fn ensure_package_present(task: &ZypperTask, dry_run: bool) -> Result<()> {
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
        if task.allow_vendor_change {
            println!("  (allowing vendor changes)");
        }
        if task.disable_gpg_check {
            println!("  (disabling GPG check)");
        }
    } else {
        // Install package
        let mut args = vec!["install".to_string(), "-y".to_string()];

        if task.allow_vendor_change {
            args.push("--allow-vendor-change".to_string());
        }

        if task.allow_downgrades {
            args.push("--allow-downgrades".to_string());
        }

        if task.disable_gpg_check {
            args.push("--no-gpg-checks".to_string());
        }

        if task.force {
            args.push("--force".to_string());
        }

        args.push(task.name.clone());

        run_zypper_command(&args).await
            .with_context(|| format!("Failed to install package {}", task.name))?;

        println!("Installed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is removed
async fn ensure_package_absent(task: &ZypperTask, dry_run: bool) -> Result<()> {
    // Check if package is installed
    let is_installed = match is_package_installed(&task.name) {
        Ok(installed) => installed,
        Err(_) => {
            // If we can't check installation status, assume it's not installed for dry runs
            // or fail for real runs
            if dry_run {
                false
            } else {
                return Err(anyhow::anyhow!("Cannot determine if package {} is installed", task.name));
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
        // Remove package
        let mut args = vec!["remove".to_string(), "-y".to_string()];

        if task.force {
            args.push("--force".to_string());
        }

        args.push(task.name.clone());

        run_zypper_command(&args).await
            .with_context(|| format!("Failed to remove package {}", task.name))?;

        println!("Removed package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is at latest version
async fn ensure_package_latest(task: &ZypperTask, dry_run: bool) -> Result<()> {
    // Update cache first
    update_cache(task, dry_run).await?;

    if dry_run {
        println!("Would upgrade package: {}", task.name);
    } else {
        // Upgrade specific package
        let mut args = vec!["update".to_string(), "-y".to_string()];

        if task.allow_vendor_change {
            args.push("--allow-vendor-change".to_string());
        }

        args.push(task.name.clone());

        run_zypper_command(&args).await
            .with_context(|| format!("Failed to upgrade package {}", task.name))?;

        println!("Upgraded package: {}", task.name);
    }

    Ok(())
}

/// Update package cache
async fn update_cache(_task: &ZypperTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would refresh package cache");
    } else {
        run_zypper_command(&["refresh".to_string()]).await
            .with_context(|| "Failed to refresh package cache")?;
        println!("Refreshed package cache");
    }

    Ok(())
}

/// Check if package is installed
fn is_package_installed(package_name: &str) -> Result<bool> {
    let output = Command::new("rpm")
        .args(["-q", package_name])
        .output()
        .with_context(|| format!("Failed to check package status: {}", package_name))?;

    Ok(output.status.success())
}

/// Run zypper command with proper error handling
async fn run_zypper_command(args: &[String]) -> Result<()> {
    let output = Command::new("zypper")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run zypper command: {:?}", args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Zypper command failed: {:?}\nstdout: {}\nstderr: {}",
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
    async fn test_zypper_install_dry_run() {
        let task = ZypperTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Present,
            update_cache: false,
            allow_vendor_change: false,
            allow_downgrades: false,
            disable_gpg_check: false,
            force: false,
        };

        let result = execute_zypper_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_zypper_remove_dry_run() {
        let task = ZypperTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Absent,
            update_cache: false,
            allow_vendor_change: false,
            allow_downgrades: false,
            disable_gpg_check: false,
            force: false,
        };

        let result = execute_zypper_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_zypper_upgrade_dry_run() {
        let task = ZypperTask {
            description: None,
            name: "curl".to_string(),
            state: PackageState::Latest,
            update_cache: true,
            allow_vendor_change: true,
            allow_downgrades: false,
            disable_gpg_check: false,
            force: false,
        };

        let result = execute_zypper_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_package_installed() {
        // Test package installation check - may fail in test environments
        let result = is_package_installed("filesystem");
        // Just ensure the function doesn't panic, result may be error if rpm not available
        let _ = result;
    }
}