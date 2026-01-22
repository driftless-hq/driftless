//! Ruby gem management task executor
//!
//! Handles gem installation, removal, and updates using gem.
//!
//! # Examples
//!
//! ## Install a gem
//!
//! This example installs the bundler gem.
//!
//! **YAML Format:**
//! ```yaml
//! - type: gem
//!   description: "Install bundler gem"
//!   name: bundler
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "gem",
//!   "description": "Install bundler gem",
//!   "name": "bundler",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "gem"
//! description = "Install bundler gem"
//! name = "bundler"
//! state = "present"
//! ```
//!
//! ## Install gem with specific version
//!
//! This example installs a specific version of the rails gem.
//!
//! **YAML Format:**
//! ```yaml
//! - type: gem
//!   description: "Install Rails 7.0"
//!   name: rails
//!   state: present
//!   version: "7.0.0"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "gem",
//!   "description": "Install Rails 7.0",
//!   "name": "rails",
//!   "state": "present",
//!   "version": "7.0.0"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "gem"
//! description = "Install Rails 7.0"
//! name = "rails"
//! state = "present"
//! version = "7.0.0"
//! ```
//!
//! ## Install gem for specific user
//!
//! This example installs a gem in the user's home directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: gem
//!   description: "Install jekyll for user"
//!   name: jekyll
//!   state: present
//!   user_install: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "gem",
//!   "description": "Install jekyll for user",
//!   "name": "jekyll",
//!   "state": "present",
//!   "user_install": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "gem"
//! description = "Install jekyll for user"
//! name = "jekyll"
//! state = "present"
//! user_install = true
//! ```
//!
//! ## Remove a gem
//!
//! This example removes the bundler gem.
//!
//! **YAML Format:**
//! ```yaml
//! - type: gem
//!   description: "Remove bundler gem"
//!   name: bundler
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "gem",
//!   "description": "Remove bundler gem",
//!   "name": "bundler",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "gem"
//! description = "Remove bundler gem"
//! name = "bundler"
//! state = "absent"
//! ```

/// Ruby gem management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Gem name
    pub name: String,
    /// Gem state
    pub state: PackageState,
    /// Ruby executable path
    #[serde(default = "default_ruby_executable")]
    pub executable: String,
    /// Gem executable path
    #[serde(default = "default_gem_executable")]
    pub gem_executable: String,
    /// User installation
    #[serde(default)]
    pub user_install: bool,
    /// Version specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Install documentation
    #[serde(default)]
    pub install_doc: bool,
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

/// Execute a gem task
pub async fn execute_gem_task(task: &GemTask, dry_run: bool) -> Result<()> {
    match task.state {
        PackageState::Present => {
            ensure_gem_present(task, dry_run).await
        }
        PackageState::Absent => {
            ensure_gem_absent(task, dry_run).await
        }
        PackageState::Latest => {
            ensure_gem_latest(task, dry_run).await
        }
    }
}

/// Ensure gem is installed
async fn ensure_gem_present(task: &GemTask, dry_run: bool) -> Result<()> {
    // Check if gem is already installed
    let is_installed = is_gem_installed(task).unwrap_or_default();

    if is_installed {
        println!("Gem {} is already installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would install gem: {}", task.name);
        if task.user_install {
            println!("  (user installation)");
        }
        if !task.install_doc {
            println!("  (without documentation)");
        }
    } else {
        // Install gem
        let mut args = vec![task.gem_executable.clone(), "install".to_string()];

        if task.user_install {
            args.push("--user-install".to_string());
        }

        if !task.install_doc {
            args.push("--no-document".to_string());
        }

        if let Some(ref version) = task.version {
            args.push("--version".to_string());
            args.push(version.clone());
        }

        args.push(task.name.clone());

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_gem_command(&args).await
            .with_context(|| format!("Failed to install gem {}", task.name))?;

        println!("Installed gem: {}", task.name);
    }

    Ok(())
}

/// Ensure gem is removed
async fn ensure_gem_absent(task: &GemTask, dry_run: bool) -> Result<()> {
    // Check if gem is installed
    let is_installed = match is_gem_installed(task) {
        Ok(installed) => installed,
        Err(_) => {
            // If we can't check installation status, assume it's not installed for dry runs
            // or fail for real runs
            if dry_run {
                false
            } else {
                return Err(anyhow::anyhow!("Cannot determine if gem {} is installed", task.name));
            }
        }
    };

    if !is_installed {
        println!("Gem {} is not installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove gem: {}", task.name);
    } else {
        // Uninstall gem
        let mut args = vec![task.gem_executable.clone(), "uninstall".to_string(), "-x".to_string()];

        if task.force {
            args.push("-f".to_string());
        }

        args.push(task.name.clone());

        run_gem_command(&args).await
            .with_context(|| format!("Failed to remove gem {}", task.name))?;

        println!("Removed gem: {}", task.name);
    }

    Ok(())
}

/// Ensure gem is at latest version
async fn ensure_gem_latest(task: &GemTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would upgrade gem: {}", task.name);
        if task.user_install {
            println!("  (user installation)");
        }
    } else {
        // Update gem
        let mut args = vec![task.gem_executable.clone(), "update".to_string()];

        if task.user_install {
            args.push("--user-install".to_string());
        }

        args.push(task.name.clone());

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_gem_command(&args).await
            .with_context(|| format!("Failed to upgrade gem {}", task.name))?;

        println!("Upgraded gem: {}", task.name);
    }

    Ok(())
}

/// Check if gem is installed
fn is_gem_installed(task: &GemTask) -> Result<bool> {
    let args = vec![task.gem_executable.clone(), "list".to_string(), "--local".to_string(), task.name.clone()];

    let output = Command::new(&task.executable)
        .args(&args)
        .output()
        .with_context(|| format!("Failed to check gem status: {}", task.name))?;

    // Check if the gem name appears in the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(&task.name) && output.status.success())
}

/// Run gem command with proper error handling
async fn run_gem_command(args: &[String]) -> Result<()> {
    let output = Command::new(&args[0])
        .args(&args[1..])
        .output()
        .with_context(|| format!("Failed to run gem command: {:?}", args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Gem command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        ));
    }

    Ok(())
}

/// Default Ruby executable ("ruby")
pub fn default_ruby_executable() -> String { "ruby".to_string() }
/// Default gem executable ("gem")
pub fn default_gem_executable() -> String { "gem".to_string() }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gem_install_dry_run() {
        let task = GemTask {
            description: None,
            name: "rails".to_string(),
            state: PackageState::Present,
            executable: "ruby".to_string(),
            gem_executable: "gem".to_string(),
            user_install: false,
            version: Some("7.0.0".to_string()),
            install_doc: false,
            extra_args: vec![],
            force: false,
        };

        let result = execute_gem_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gem_install_user_dry_run() {
        let task = GemTask {
            description: None,
            name: "bundler".to_string(),
            state: PackageState::Present,
            executable: "ruby".to_string(),
            gem_executable: "gem".to_string(),
            user_install: true,
            version: None,
            install_doc: true,
            extra_args: vec!["--verbose".to_string()],
            force: false,
        };

        let result = execute_gem_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gem_remove_dry_run() {
        let task = GemTask {
            description: None,
            name: "rails".to_string(),
            state: PackageState::Absent,
            executable: "ruby".to_string(),
            gem_executable: "gem".to_string(),
            user_install: false,
            version: None,
            install_doc: false,
            extra_args: vec![],
            force: true,
        };

        let result = execute_gem_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_gem_upgrade_dry_run() {
        let task = GemTask {
            description: None,
            name: "rails".to_string(),
            state: PackageState::Latest,
            executable: "ruby".to_string(),
            gem_executable: "gem".to_string(),
            user_install: false,
            version: None,
            install_doc: false,
            extra_args: vec![],
            force: false,
        };

        let result = execute_gem_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_gem_installed() {
        let task = GemTask {
            description: None,
            name: "rubygems".to_string(), // Should be available in most Ruby environments
            state: PackageState::Present,
            executable: "ruby".to_string(),
            gem_executable: "gem".to_string(),
            user_install: false,
            version: None,
            install_doc: false,
            extra_args: vec![],
            force: false,
        };

        let result = is_gem_installed(&task);
        // Just ensure the function doesn't panic, result may be error if ruby/gem not available
        let _ = result;
    }
}