//! Python package management task executor
//!
//! Handles package installation, removal, and updates using pip.
//!
//! # Examples
//!
//! ## Install a Python package
//!
//! This example installs the requests package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pip
//!   description: "Install requests package"
//!   name: requests
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pip",
//!   "description": "Install requests package",
//!   "name": "requests",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pip"
//! description = "Install requests package"
//! name = "requests"
//! state = "present"
//! ```
//!
//! ## Install package in virtual environment
//!
//! This example installs a package in a specific virtual environment.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pip
//!   description: "Install Django in virtualenv"
//!   name: django
//!   state: present
//!   virtualenv: /opt/myapp/venv
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pip",
//!   "description": "Install Django in virtualenv",
//!   "name": "django",
//!   "state": "present",
//!   "virtualenv": "/opt/myapp/venv"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pip"
//! description = "Install Django in virtualenv"
//! name = "django"
//! state = "present"
//! virtualenv = "/opt/myapp/venv"
//! ```
//!
//! ## Install specific version
//!
//! This example installs a specific version of a package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pip
//!   description: "Install Flask 2.0"
//!   name: flask
//!   state: present
//!   version: "2.0.0"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pip",
//!   "description": "Install Flask 2.0",
//!   "name": "flask",
//!   "state": "present",
//!   "version": "2.0.0"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pip"
//! description = "Install Flask 2.0"
//! name = "flask"
//! state = "present"
//! version = "2.0.0"
//! ```
//!
//! ## Remove a Python package
//!
//! This example removes the requests package.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pip
//!   description: "Remove requests package"
//!   name: requests
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "pip",
//!   "description": "Remove requests package",
//!   "name": "requests",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pip"
//! description = "Remove requests package"
//! name = "requests"
//! state = "absent"
//! ```

/// Python package management task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipTask {
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
    /// Python executable path
    #[serde(default = "default_python_executable")]
    pub executable: String,
    /// Virtual environment path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtualenv: Option<String>,
    /// Requirements file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<String>,
    /// Version specification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
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
use std::env;
use std::process::Command;

/// Execute a pip task
pub async fn execute_pip_task(task: &PipTask, dry_run: bool) -> Result<()> {
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
async fn ensure_package_present(task: &PipTask, dry_run: bool) -> Result<()> {
    // Check if package is already installed
    let is_installed = is_package_installed(task).unwrap_or_default();

    if is_installed {
        println!("Python package {} is already installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would install Python package: {}", task.name);
        if let Some(ref venv) = task.virtualenv {
            println!("  (in virtual environment: {})", venv);
        }
        if task.force {
            println!("  (with force)");
        }
    } else {
        // Install package
        let mut args = vec![task.executable.clone(), "install".to_string()];

        if let Some(ref venv) = task.virtualenv {
            // Activate virtual environment
            env::set_var("VIRTUAL_ENV", venv);
            env::set_var("PATH", format!("{}/bin:{}", venv, env::var("PATH").unwrap_or_default()));
        }

        if let Some(ref requirements) = task.requirements {
            args.push("-r".to_string());
            args.push(requirements.clone());
        } else {
            // Install specific package
            if let Some(ref version) = task.version {
                args.push(format!("{}=={}", task.name, version));
            } else {
                args.push(task.name.clone());
            }
        }

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_pip_command(&args).await
            .with_context(|| format!("Failed to install Python package {}", task.name))?;

        println!("Installed Python package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is removed
async fn ensure_package_absent(task: &PipTask, dry_run: bool) -> Result<()> {
    // Check if package is installed
    let is_installed = match is_package_installed(task) {
        Ok(installed) => installed,
        Err(_) => {
            // If we can't check installation status, assume it's not installed for dry runs
            // or fail for real runs
            if dry_run {
                false
            } else {
                return Err(anyhow::anyhow!("Cannot determine if Python package {} is installed", task.name));
            }
        }
    };

    if !is_installed {
        println!("Python package {} is not installed", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove Python package: {}", task.name);
        if let Some(ref venv) = task.virtualenv {
            println!("  (from virtual environment: {})", venv);
        }
    } else {
        // Uninstall package
        let mut args = vec![task.executable.clone(), "uninstall".to_string(), "-y".to_string()];

        if let Some(ref venv) = task.virtualenv {
            // Activate virtual environment
            env::set_var("VIRTUAL_ENV", venv);
            env::set_var("PATH", format!("{}/bin:{}", venv, env::var("PATH").unwrap_or_default()));
        }

        args.push(task.name.clone());

        run_pip_command(&args).await
            .with_context(|| format!("Failed to remove Python package {}", task.name))?;

        println!("Removed Python package: {}", task.name);
    }

    Ok(())
}

/// Ensure package is at latest version
async fn ensure_package_latest(task: &PipTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would upgrade Python package: {}", task.name);
        if let Some(ref venv) = task.virtualenv {
            println!("  (in virtual environment: {})", venv);
        }
    } else {
        // Upgrade package
        let mut args = vec![task.executable.clone(), "install".to_string(), "--upgrade".to_string()];

        if let Some(ref venv) = task.virtualenv {
            // Activate virtual environment
            env::set_var("VIRTUAL_ENV", venv);
            env::set_var("PATH", format!("{}/bin:{}", venv, env::var("PATH").unwrap_or_default()));
        }

        args.push(task.name.clone());

        // Add extra arguments
        args.extend(task.extra_args.clone());

        run_pip_command(&args).await
            .with_context(|| format!("Failed to upgrade Python package {}", task.name))?;

        println!("Upgraded Python package: {}", task.name);
    }

    Ok(())
}

/// Check if package is installed
fn is_package_installed(task: &PipTask) -> Result<bool> {
    let mut args = vec![task.executable.clone(), "show".to_string()];

    if let Some(ref venv) = task.virtualenv {
        // Activate virtual environment
        env::set_var("VIRTUAL_ENV", venv);
        env::set_var("PATH", format!("{}/bin:{}", venv, env::var("PATH").unwrap_or_default()));
    }

    args.push(task.name.clone());

    let output = Command::new(&task.executable)
        .args(&args[1..]) // Skip the executable name since we already specified it
        .output()
        .with_context(|| format!("Failed to check Python package status: {}", task.name))?;

    Ok(output.status.success())
}

/// Run pip command with proper error handling
async fn run_pip_command(args: &[String]) -> Result<()> {
    let output = Command::new(&args[0])
        .args(&args[1..])
        .output()
        .with_context(|| format!("Failed to run pip command: {:?}", args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Pip command failed: {:?}\nstdout: {}\nstderr: {}",
            args,
            stdout,
            stderr
        ));
    }

    Ok(())
}

/// Default Python executable ("python3")
pub fn default_python_executable() -> String { "python3".to_string() }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pip_install_dry_run() {
        let task = PipTask {
            description: None,
            name: "requests".to_string(),
            state: PackageState::Present,
            executable: "python3".to_string(),
            virtualenv: None,
            requirements: None,
            version: Some("2.25.1".to_string()),
            extra_args: vec![],
            force: false,
        };

        let result = execute_pip_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pip_install_with_venv_dry_run() {
        let task = PipTask {
            description: None,
            name: "requests".to_string(),
            state: PackageState::Present,
            executable: "python3".to_string(),
            virtualenv: Some("/path/to/venv".to_string()),
            requirements: None,
            version: None,
            extra_args: vec!["--quiet".to_string()],
            force: false,
        };

        let result = execute_pip_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pip_remove_dry_run() {
        let task = PipTask {
            description: None,
            name: "requests".to_string(),
            state: PackageState::Absent,
            executable: "python3".to_string(),
            virtualenv: None,
            requirements: None,
            version: None,
            extra_args: vec![],
            force: false,
        };

        let result = execute_pip_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pip_upgrade_dry_run() {
        let task = PipTask {
            description: None,
            name: "requests".to_string(),
            state: PackageState::Latest,
            executable: "python3".to_string(),
            virtualenv: Some("/tmp/test_venv".to_string()),
            requirements: None,
            version: None,
            extra_args: vec![],
            force: false,
        };

        let result = execute_pip_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_package_installed() {
        let task = PipTask {
            description: None,
            name: "pip".to_string(), // pip should be installed in most Python environments
            state: PackageState::Present,
            executable: "python3".to_string(),
            virtualenv: None,
            requirements: None,
            version: None,
            extra_args: vec![],
            force: false,
        };

        let result = is_package_installed(&task);
        // We don't assert the result since it depends on the system
        assert!(result.is_ok());
    }
}