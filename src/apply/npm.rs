//! Node.js package management task executor
//!
//! Handles package installation, removal, and updates using npm.
//!
//! # Examples
//!
//! ## Install an npm package
//!
//! This example installs the express package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: npm
//!   description: "Install express package"
//!   name: express
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "npm",
//!   "description": "Install express package",
//!   "name": "express",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "npm"
//! description = "Install express package"
//! name = "express"
//! state = "present"
//! ```
//!
//! ## Install package globally
//!
//! This example installs a package globally.
//!
//! **YAML Format:**
//! ```yaml
//! - type: npm
//!   description: "Install PM2 globally"
//!   name: pm2
//!   state: present
//!   global: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "npm",
//!   "description": "Install PM2 globally",
//!   "name": "pm2",
//!   "state": "present",
//!   "global": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "npm"
//! description = "Install PM2 globally"
//! name = "pm2"
//! state = "present"
//! global = true
//! ```
//!
//! ## Install specific version
//!
//! This example installs a specific version of a package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: npm
//!   description: "Install React 18"
//!   name: react
//!   state: present
//!   version: "18.2.0"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "npm",
//!   "description": "Install React 18",
//!   "name": "react",
//!   "state": "present",
//!   "version": "18.2.0"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "npm"
//! description = "Install React 18"
//! name = "react"
//! state = "present"
//! version = "18.2.0"
//! ```
//!
//! ## Remove an npm package
//!
//! This example removes the express package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: npm
//!   description: "Remove express package"
//!   name: express
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "npm",
//!   "description": "Remove express package",
//!   "name": "express",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "npm"
//! description = "Remove express package"
//! name = "express"
//! state = "absent"
//! ```

/// Node.js package management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmTask {
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
    /// NPM executable path
    #[serde(default = "default_npm_executable")]
    pub executable: String,
    /// Global installation
    #[serde(default)]
    pub global: bool,
    /// Production only
    #[serde(default)]
    pub production: bool,
    /// Version specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Registry URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Extra arguments
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// Force installation
    #[serde(default)]
    pub force: bool,
}

use serde::{Deserialize, Serialize};

use crate::apply::PackageState;
use anyhow::{Context, Result};
use std::process::Command;

/// Execute an npm task
pub async fn execute_npm_task(task: &NpmTask, dry_run: bool) -> Result<()> {
    match task.state {
        PackageState::Present => ensure_package_present(task, dry_run).await,
        PackageState::Absent => ensure_package_absent(task, dry_run).await,
        PackageState::Latest => ensure_package_latest(task, dry_run).await,
    }
}

/// Ensure package is installed
async fn ensure_package_present(task: &NpmTask, dry_run: bool) -> Result<()> {
    // Check if package is already installed
    let is_installed = is_package_installed(task).unwrap_or_default();

    if is_installed {
        println!("NPM package {} is already installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would install NPM package: {}", task.name);
        if task.global {
            println!("  (globally)");
        }
        if let Some(ref registry) = task.registry {
            println!("  (from registry: {})", registry);
        }
    } else {
        // Install package
        let mut args = vec!["install".to_string()];

        if task.global {
            args.push("--global".to_string());
        }

        if task.production {
            args.push("--production".to_string());
        }

        if let Some(ref registry) = task.registry {
            args.push("--registry".to_string());
            args.push(registry.clone());
        }

        if let Some(ref version) = task.version {
            args.push(format!("{}@{}", task.name, version));
        } else {
            args.push(task.name.clone());
        }

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_npm_command(&args, &task.executable)
            .await
            .with_context(|| format!("Failed to install NPM package {}", task.name))?;

        println!("Installed NPM package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is removed
async fn ensure_package_absent(task: &NpmTask, dry_run: bool) -> Result<()> {
    // Check if package is installed
    let is_installed = match is_package_installed(task) {
        Ok(installed) => installed,
        Err(_) => {
            // If we can't check installation status, assume it's not installed for dry runs
            // or fail for real runs
            if dry_run {
                false
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot determine if NPM package {} is installed",
                    task.name
                ));
            }
        }
    };

    if !is_installed {
        println!("NPM package {} is not installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove NPM package: {}", task.name);
        if task.global {
            println!("  (globally)");
        }
    } else {
        // Uninstall package
        let mut args = vec!["uninstall".to_string()];

        if task.global {
            args.push("--global".to_string());
        }

        args.push(task.name.clone());

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_npm_command(&args, &task.executable)
            .await
            .with_context(|| format!("Failed to remove NPM package {}", task.name))?;

        println!("Removed NPM package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is at latest version
async fn ensure_package_latest(task: &NpmTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would upgrade NPM package: {}", task.name);
        if task.global {
            println!("  (globally)");
        }
    } else {
        // Update package
        let mut args = vec!["update".to_string()];

        if task.global {
            args.push("--global".to_string());
        }

        args.push(task.name.clone());

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_npm_command(&args, &task.executable)
            .await
            .with_context(|| format!("Failed to upgrade NPM package {}", task.name))?;

        println!("Upgraded NPM package: {}", task.name);
    }

    Ok(())
}

/// Check if package is installed
fn is_package_installed(task: &NpmTask) -> Result<bool> {
    let mut args = vec!["list".to_string()];

    if task.global {
        args.push("--global".to_string());
    }

    args.push(task.name.clone());

    let output = Command::new(&task.executable)
        .args(&args)
        .output()
        .with_context(|| format!("Failed to check NPM package status: {}", task.name))?;

    // npm list returns exit code 1 if package is not found
    Ok(output.status.success())
}

/// Run npm command with proper error handling
async fn run_npm_command(args: &[String], executable: &str) -> Result<()> {
    let output = Command::new(executable)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run npm command: {} {:?}", executable, args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "NPM command failed: {} {:?}\nstdout: {}\nstderr: {}",
            executable,
            args,
            stdout,
            stderr
        ));
    }

    Ok(())
}

/// Default npm executable ("npm")
pub fn default_npm_executable() -> String {
    "npm".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_npm_install_dry_run() {
        let task = NpmTask {
            description: None,
            name: "express".to_string(),
            state: PackageState::Present,
            executable: "npm".to_string(),
            global: false,
            production: false,
            version: Some("4.18.2".to_string()),
            registry: None,
            extra_args: vec![],
            force: false,
        };

        let result = execute_npm_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_npm_install_global_dry_run() {
        let task = NpmTask {
            description: None,
            name: "typescript".to_string(),
            state: PackageState::Present,
            executable: "npm".to_string(),
            global: true,
            production: false,
            version: None,
            registry: Some("https://registry.npmjs.org/".to_string()),
            extra_args: vec!["--save-dev".to_string()],
            force: false,
        };

        let result = execute_npm_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_npm_remove_dry_run() {
        let task = NpmTask {
            description: None,
            name: "express".to_string(),
            state: PackageState::Absent,
            executable: "npm".to_string(),
            global: false,
            production: false,
            version: None,
            registry: None,
            extra_args: vec![],
            force: false,
        };

        let result = execute_npm_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_npm_upgrade_dry_run() {
        let task = NpmTask {
            description: None,
            name: "express".to_string(),
            state: PackageState::Latest,
            executable: "npm".to_string(),
            global: false,
            production: true,
            version: None,
            registry: None,
            extra_args: vec![],
            force: false,
        };

        let result = execute_npm_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_package_installed() {
        let task = NpmTask {
            description: None,
            name: "npm".to_string(), // npm should know about itself
            state: PackageState::Present,
            executable: "npm".to_string(),
            global: true,
            production: false,
            version: None,
            registry: None,
            extra_args: vec![],
            force: false,
        };

        let result = is_package_installed(&task);
        // Just ensure the function doesn't panic, result may be error if npm not available
        let _ = result;
    }
}
