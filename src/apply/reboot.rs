//! Reboot task executor
//!
//! Handles system reboot operations.
//!
//! # Examples
//!
//! ## Reboot system with delay
//!
//! This example reboots the system after a 60-second delay.
//!
//! **YAML Format:**
//! ```yaml
//! - type: reboot
//!   description: "Reboot system after kernel update"
//!   delay: 60
//!   msg: "System will reboot in 60 seconds for kernel update"
//!   force: false
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "reboot",
//!   "description": "Reboot system after kernel update",
//!   "delay": 60,
//!   "msg": "System will reboot in 60 seconds for kernel update",
//!   "force": false
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "reboot"
//! description = "Reboot system after kernel update"
//! delay = 60
//! msg = "System will reboot in 60 seconds for kernel update"
//! force = false
//! ```
//!
//! ## Immediate reboot
//!
//! This example reboots the system immediately.
//!
//! **YAML Format:**
//! ```yaml
//! - type: reboot
//!   description: "Immediate system reboot"
//!   delay: 0
//!   force: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "reboot",
//!   "description": "Immediate system reboot",
//!   "delay": 0,
//!   "force": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "reboot"
//! description = "Immediate system reboot"
//! delay = 0
//! force = true
//! ```
//!
//! ## Test reboot (dry run)
//!
//! This example tests the reboot configuration without actually rebooting.
//!
//! **YAML Format:**
//! ```yaml
//! - type: reboot
//!   description: "Test reboot configuration"
//!   delay: 30
//!   msg: "This is a test reboot - system will not actually reboot"
//!   force: false
//!   test: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "reboot",
//!   "description": "Test reboot configuration",
//!   "delay": 30,
//!   "msg": "This is a test reboot - system will not actually reboot",
//!   "force": false,
//!   "test": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "reboot"
//! description = "Test reboot configuration"
//! delay = 30
//! msg = "This is a test reboot - system will not actually reboot"
//! force = false
//! test = true
//! ```

/// System reboot task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RebootTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Delay before reboot (seconds)
    #[serde(default)]
    pub delay: u32,
    /// Message to display before reboot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// Whether to force reboot (don't wait for clean shutdown)
    #[serde(default)]
    pub force: bool,
    /// Test mode (don't actually reboot)
    #[serde(default)]
    pub test: bool,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Execute a reboot task
pub async fn execute_reboot_task(task: &RebootTask, dry_run: bool) -> Result<()> {
    if task.test {
        println!("Test mode: would reboot system");
        println!("  Delay: {} seconds", task.delay);
        if let Some(msg) = &task.msg {
            println!("  Message: {}", msg);
        }
        println!("  Force: {}", task.force);
        return Ok(());
    }

    if dry_run {
        println!("Would reboot system");
        println!("  Delay: {} seconds", task.delay);
        if let Some(msg) = &task.msg {
            println!("  Message: {}", msg);
        }
        println!("  Force: {}", task.force);
    } else {
        println!("Initiating system reboot...");
        if let Some(msg) = &task.msg {
            println!("Message: {}", msg);
        }

        // Send message to all users if provided
        if let Some(msg) = &task.msg {
            let _ = Command::new("wall").arg(msg).status(); // Ignore errors here
        }

        // Wait for the specified delay
        if task.delay > 0 {
            println!("Waiting {} seconds before reboot...", task.delay);
            tokio::time::sleep(std::time::Duration::from_secs(task.delay as u64)).await;
        }

        // Execute reboot command
        reboot_system(task.force)?;
    }

    Ok(())
}

/// Execute the actual system reboot
fn reboot_system(force: bool) -> Result<()> {
    let mut cmd = Command::new("shutdown");

    if force {
        // Force immediate reboot
        cmd.arg("-r").arg("now");
    } else {
        // Graceful reboot
        cmd.arg("-r")
            .arg("+0")
            .arg("Rebooting system via Driftless");
    }

    let status = cmd
        .status()
        .with_context(|| "Failed to execute reboot command")?;

    if !status.success() {
        // Try alternative reboot command
        let alt_status = Command::new("reboot")
            .status()
            .with_context(|| "Failed to execute alternative reboot command")?;

        if !alt_status.success() {
            return Err(anyhow::anyhow!("All reboot commands failed"));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reboot_dry_run() {
        let task = RebootTask {
            description: None,
            delay: 10,
            msg: Some("System maintenance reboot".to_string()),
            force: false,
            test: false,
        };

        let result = execute_reboot_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reboot_test_mode() {
        let task = RebootTask {
            description: None,
            delay: 5,
            msg: Some("Test reboot".to_string()),
            force: true,
            test: true,
        };

        let result = execute_reboot_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reboot_zero_delay() {
        let task = RebootTask {
            description: None,
            delay: 0, // Immediate reboot
            msg: None,
            force: false,
            test: true,
        };

        let result = execute_reboot_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reboot_very_long_delay() {
        let task = RebootTask {
            description: None,
            delay: 86400, // 24 hours - very long delay
            msg: Some("Scheduled maintenance reboot".to_string()),
            force: false,
            test: true,
        };

        let result = execute_reboot_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reboot_empty_message() {
        let task = RebootTask {
            description: None,
            delay: 10,
            msg: Some("".to_string()), // Empty message
            force: false,
            test: true,
        };

        let result = execute_reboot_task(&task, true).await;
        assert!(result.is_ok());
    }
}
