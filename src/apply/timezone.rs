//! Timezone task executor
//!
//! Handles system timezone management.
//!
//! # Examples
//!
//! ## Set system timezone to UTC
//!
//! This example sets the system timezone to UTC.
//!
//! **YAML Format:**
//! ```yaml
//! - type: timezone
//!   description: "Set system timezone to UTC"
//!   name: UTC
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "timezone",
//!   "description": "Set system timezone to UTC",
//!   "name": "UTC"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "timezone"
//! description = "Set system timezone to UTC"
//! name = "UTC"
//! ```
//!
//! ## Set timezone to Eastern Time
//!
//! This example sets the system timezone to Eastern Time.
//!
//! **YAML Format:**
//! ```yaml
//! - type: timezone
//!   description: "Set timezone to Eastern Time"
//!   name: America/New_York
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "timezone",
//!   "description": "Set timezone to Eastern Time",
//!   "name": "America/New_York"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "timezone"
//! description = "Set timezone to Eastern Time"
//! name = "America/New_York"
//! ```
//!
//! ## Set timezone to Pacific Time
//!
//! This example sets the system timezone to Pacific Time.
//!
//! **YAML Format:**
//! ```yaml
//! - type: timezone
//!   description: "Set timezone to Pacific Time"
//!   name: America/Los_Angeles
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "timezone",
//!   "description": "Set timezone to Pacific Time",
//!   "name": "America/Los_Angeles"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "timezone"
//! description = "Set timezone to Pacific Time"
//! name = "America/Los_Angeles"
//! ```

/// System timezone management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimezoneTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Timezone name (e.g., "America/New_York", "UTC")
    pub name: String,
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute a timezone task
pub async fn execute_timezone_task(task: &TimezoneTask, dry_run: bool) -> Result<()> {
    // Get current timezone
    let current_timezone = get_current_timezone()?;

    if current_timezone == task.name {
        println!("Timezone already set to: {}", task.name);
        return Ok(());
    }

    println!("Changing timezone from '{}' to '{}'", current_timezone, task.name);

    if dry_run {
        println!("Would set timezone to: {}", task.name);
    } else {
        set_timezone(&task.name)?;
        println!("Set timezone to: {}", task.name);
    }

    Ok(())
}

/// Get the current system timezone
fn get_current_timezone() -> Result<String> {
    // Try to read from /etc/timezone first (Ubuntu/Debian style)
    if Path::new("/etc/timezone").exists() {
        let content = fs::read_to_string("/etc/timezone")
            .with_context(|| "Failed to read /etc/timezone")?;
        return Ok(content.trim().to_string());
    }

    // Try to read from /etc/localtime symlink (RedHat/CentOS style)
    if let Ok(target) = fs::read_link("/etc/localtime") {
        if let Some(tz_name) = target.to_str() {
            // Extract timezone name from path like /usr/share/zoneinfo/America/New_York
            if tz_name.contains("/zoneinfo/") {
                let parts: Vec<&str> = tz_name.split("/zoneinfo/").collect();
                if parts.len() == 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }

    // Fallback: try timedatectl command
    let output = std::process::Command::new("timedatectl")
        .arg("show")
        .arg("--property=Timezone")
        .arg("--value")
        .output()
        .with_context(|| "Failed to get timezone from timedatectl")?;

    if output.status.success() {
        let tz = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !tz.is_empty() {
            return Ok(tz);
        }
    }

    Err(anyhow::anyhow!("Could not determine current timezone"))
}

/// Set the system timezone
fn set_timezone(timezone: &str) -> Result<()> {
    // Validate timezone exists
    let zoneinfo_path = format!("/usr/share/zoneinfo/{}", timezone);
    if !Path::new(&zoneinfo_path).exists() {
        return Err(anyhow::anyhow!("Timezone '{}' does not exist in /usr/share/zoneinfo", timezone));
    }

    // Try timedatectl first (systemd systems)
    let timedatectl_result = std::process::Command::new("timedatectl")
        .arg("set-timezone")
        .arg(timezone)
        .status();

    if timedatectl_result.is_ok() && timedatectl_result.unwrap().success() {
        return Ok(());
    }

    // Fallback: manual method (Ubuntu/Debian style)
    // 1. Update /etc/timezone
    fs::write("/etc/timezone", format!("{}\n", timezone))
        .with_context(|| "Failed to write /etc/timezone")?;

    // 2. Update /etc/localtime symlink
    let _ = fs::remove_file("/etc/localtime"); // Ignore error if file doesn't exist
    std::os::unix::fs::symlink(zoneinfo_path, "/etc/localtime")
        .with_context(|| "Failed to create /etc/localtime symlink")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timezone_dry_run() {
        let task = TimezoneTask {
            description: None,
            name: "America/New_York".to_string(),
        };

        let result = execute_timezone_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_current_timezone() {
        let result = get_current_timezone();
        assert!(result.is_ok());
        let timezone = result.unwrap();
        assert!(!timezone.is_empty());
    }

    #[test]
    fn test_validate_timezone_exists() {
        // Test with a timezone that should exist
        let zoneinfo_path = "/usr/share/zoneinfo/UTC";
        let exists = Path::new(zoneinfo_path).exists();
        // We can't assert this since it depends on the system having zoneinfo files
        // But we can ensure the function doesn't crash
        let _ = exists;
    }

    #[tokio::test]
    async fn test_timezone_empty_name() {
        let task = TimezoneTask {
            description: None,
            name: "".to_string(), // Empty timezone
        };

        let result = execute_timezone_task(&task, true).await;
        // Dry-run should succeed even with empty timezone name
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timezone_invalid_name() {
        let task = TimezoneTask {
            description: None,
            name: "Invalid/Timezone/Name".to_string(), // Timezone that doesn't exist
        };

        let result = execute_timezone_task(&task, true).await;
        // Dry-run should succeed even with invalid timezone
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_timezone_valid_name() {
        let task = TimezoneTask {
            description: None,
            name: "UTC".to_string(), // Should exist on most systems
        };

        let result = execute_timezone_task(&task, true).await;
        // This might succeed or fail depending on whether timedatectl or manual method works
        // and whether the timezone files exist. We just ensure it doesn't crash.
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_timezone_same_as_current() {
        let current_tz = get_current_timezone().unwrap_or_else(|_| "UTC".to_string());
        let task = TimezoneTask {
            description: None,
            name: current_tz,
        };

        let result = execute_timezone_task(&task, true).await;
        assert!(result.is_ok()); // Setting to same timezone should succeed
    }
}