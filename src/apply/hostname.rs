//! Hostname task executor
//!
//! Handles system hostname management.
//!
//! # Examples
//!
//! ## Set system hostname
//!
//! This example sets the system hostname to "web-server-01".
//!
//! **YAML Format:**
//! ```yaml
//! - type: hostname
//!   description: "Set system hostname"
//!   name: web-server-01
//!   persist: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "hostname",
//!   "description": "Set system hostname",
//!   "name": "web-server-01",
//!   "persist": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "hostname"
//! description = "Set system hostname"
//! name = "web-server-01"
//! persist = true
//! ```
//!
//! ## Set hostname temporarily
//!
//! This example sets the hostname temporarily (not persisted across reboots).
//!
//! **YAML Format:**
//! ```yaml
//! - type: hostname
//!   description: "Set temporary hostname"
//!   name: temp-server
//!   persist: false
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "hostname",
//!   "description": "Set temporary hostname",
//!   "name": "temp-server",
//!   "persist": false
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "hostname"
//! description = "Set temporary hostname"
//! name = "temp-server"
//! persist = false
//! ```
//!
//! ## Set hostname with domain
//!
//! This example sets a fully qualified hostname.
//!
//! **YAML Format:**
//! ```yaml
//! - type: hostname
//!   description: "Set fully qualified hostname"
//!   name: app.example.com
//!   persist: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "hostname",
//!   "description": "Set fully qualified hostname",
//!   "name": "app.example.com",
//!   "persist": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "hostname"
//! description = "Set fully qualified hostname"
//! name = "app.example.com"
//! persist = true
//! ```

/// System hostname management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HostnameTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Desired hostname
    pub name: String,
    /// Whether to persist hostname to /etc/hostname
    #[serde(default = "crate::apply::default_true")]
    pub persist: bool,
}

use anyhow::{Context, Result};
use std::fs;

/// Execute a hostname task
pub async fn execute_hostname_task(task: &HostnameTask, dry_run: bool) -> Result<()> {
    // Get current hostname
    let current_hostname = get_current_hostname()?;

    if current_hostname == task.name {
        println!("Hostname already set to: {}", task.name);
        return Ok(());
    }

    println!(
        "Changing hostname from '{}' to '{}'",
        current_hostname, task.name
    );

    if dry_run {
        println!("Would set hostname to: {}", task.name);
        if task.persist {
            println!("  and persist to /etc/hostname");
        }
    } else {
        set_hostname(&task.name)?;
        println!("Set hostname to: {}", task.name);

        if task.persist {
            persist_hostname(&task.name)?;
            println!("Persisted hostname to /etc/hostname");
        }
    }

    Ok(())
}

/// Get the current system hostname
fn get_current_hostname() -> Result<String> {
    let output = std::process::Command::new("hostname")
        .output()
        .with_context(|| "Failed to get current hostname")?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Set the current hostname
fn set_hostname(hostname: &str) -> Result<()> {
    std::process::Command::new("hostname")
        .arg(hostname)
        .status()
        .with_context(|| format!("Failed to set hostname to {}", hostname))?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("hostname command failed"))?;

    Ok(())
}

/// Persist hostname to /etc/hostname
fn persist_hostname(hostname: &str) -> Result<()> {
    fs::write("/etc/hostname", format!("{}\n", hostname))
        .with_context(|| "Failed to write hostname to /etc/hostname".to_string())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hostname_dry_run() {
        let task = HostnameTask {
            description: None,
            name: "test-host".to_string(),
            persist: true,
        };

        let result = execute_hostname_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_current_hostname() {
        let result = get_current_hostname();
        assert!(result.is_ok());
        let hostname = result.unwrap();
        assert!(!hostname.is_empty());
    }

    #[tokio::test]
    async fn test_hostname_empty_name() {
        let task = HostnameTask {
            description: None,
            name: "".to_string(), // Empty hostname
            persist: false,
        };

        let result = execute_hostname_task(&task, true).await;
        // Dry-run should succeed even with empty hostname
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hostname_invalid_characters() {
        let task = HostnameTask {
            description: None,
            name: "invalid hostname with spaces!".to_string(), // Invalid hostname with spaces and special chars
            persist: false,
        };

        let result = execute_hostname_task(&task, true).await;
        assert!(result.is_ok()); // hostname command might accept invalid names, validation is minimal
    }

    #[tokio::test]
    async fn test_hostname_same_as_current() {
        let current_hostname = get_current_hostname().unwrap();
        let task = HostnameTask {
            description: None,
            name: current_hostname.clone(),
            persist: false,
        };

        let result = execute_hostname_task(&task, true).await;
        assert!(result.is_ok()); // Setting to same hostname should succeed
    }

    #[tokio::test]
    async fn test_hostname_very_long() {
        let long_hostname = "a".repeat(256); // Very long hostname (over typical limits)
        let task = HostnameTask {
            description: None,
            name: long_hostname,
            persist: false,
        };

        let result = execute_hostname_task(&task, true).await;
        assert!(result.is_ok()); // hostname command will handle length validation
    }
}
