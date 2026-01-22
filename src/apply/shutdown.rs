//! Shutdown task executor
//!
//! Handles system shutdown operations.
//!
//! # Examples
//!
//! ## Shutdown system with delay
//!
//! This example shuts down the system after a 30-second delay.
//!
//! **YAML Format:**
//! ```yaml
//! - type: shutdown
//!   description: "Shutdown system for maintenance"
//!   delay: 30
//!   msg: "System will shutdown in 30 seconds for maintenance"
//!   force: false
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "shutdown",
//!   "description": "Shutdown system for maintenance",
//!   "delay": 30,
//!   "msg": "System will shutdown in 30 seconds for maintenance",
//!   "force": false
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "shutdown"
//! description = "Shutdown system for maintenance"
//! delay = 30
//! msg = "System will shutdown in 30 seconds for maintenance"
//! force = false
//! ```
//!
//! ## Immediate shutdown
//!
//! This example shuts down the system immediately without delay.
//!
//! **YAML Format:**
//! ```yaml
//! - type: shutdown
//!   description: "Immediate system shutdown"
//!   delay: 0
//!   force: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "shutdown",
//!   "description": "Immediate system shutdown",
//!   "delay": 0,
//!   "force": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "shutdown"
//! description = "Immediate system shutdown"
//! delay = 0
//! force = true
//! ```
//!
//! ## Test shutdown (dry run)
//!
//! This example tests the shutdown configuration without actually shutting down.
//!
//! **YAML Format:**
//! ```yaml
//! - type: shutdown
//!   description: "Test shutdown configuration"
//!   delay: 60
//!   msg: "This is a test shutdown - system will not actually shutdown"
//!   force: false
//!   test: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "shutdown",
//!   "description": "Test shutdown configuration",
//!   "delay": 60,
//!   "msg": "This is a test shutdown - system will not actually shutdown",
//!   "force": false,
//!   "test": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "shutdown"
//! description = "Test shutdown configuration"
//! delay = 60
//! msg = "This is a test shutdown - system will not actually shutdown"
//! force = false
//! test = true
//! ```

/// System shutdown task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShutdownTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Delay before shutdown (seconds)
    #[serde(default)]
    pub delay: u32,
    /// Message to display before shutdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// Whether to force shutdown (don't wait for clean shutdown)
    #[serde(default)]
    pub force: bool,
    /// Test mode (don't actually shutdown)
    #[serde(default)]
    pub test: bool,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Execute a shutdown task
pub async fn execute_shutdown_task(task: &ShutdownTask, dry_run: bool) -> Result<()> {
    if task.test {
        println!("Test mode: would shutdown system");
        println!("  Delay: {} seconds", task.delay);
        if let Some(msg) = &task.msg {
            println!("  Message: {}", msg);
        }
        println!("  Force: {}", task.force);
        return Ok(());
    }

    if dry_run {
        println!("Would shutdown system");
        println!("  Delay: {} seconds", task.delay);
        if let Some(msg) = &task.msg {
            println!("  Message: {}", msg);
        }
        println!("  Force: {}", task.force);
    } else {
        println!("Initiating system shutdown...");
        if let Some(msg) = &task.msg {
            println!("Message: {}", msg);
        }

        // Send message to all users if provided
        if let Some(msg) = &task.msg {
            let _ = Command::new("wall").arg(msg).status(); // Ignore errors here
        }

        // Wait for the specified delay
        if task.delay > 0 {
            println!("Waiting {} seconds before shutdown...", task.delay);
            tokio::time::sleep(std::time::Duration::from_secs(task.delay as u64)).await;
        }

        // Execute shutdown command
        shutdown_system(task.force)?;
    }

    Ok(())
}

/// Execute the actual system shutdown
fn shutdown_system(force: bool) -> Result<()> {
    let mut cmd = Command::new("shutdown");

    if force {
        // Force immediate shutdown
        cmd.arg("-h").arg("now");
    } else {
        // Graceful shutdown
        cmd.arg("-h")
            .arg("+0")
            .arg("Shutting down system via Driftless");
    }

    let status = cmd
        .status()
        .with_context(|| "Failed to execute shutdown command")?;

    if !status.success() {
        // Try alternative shutdown commands
        let commands = vec![
            vec!["poweroff".to_string()],
            vec!["halt".to_string()],
            vec!["init".to_string(), "0".to_string()],
        ];

        for alt_cmd in commands {
            let alt_status = Command::new(&alt_cmd[0]).args(&alt_cmd[1..]).status();

            if alt_status.is_ok() && alt_status.unwrap().success() {
                return Ok(());
            }
        }

        return Err(anyhow::anyhow!("All shutdown commands failed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shutdown_dry_run() {
        let task = ShutdownTask {
            description: None,
            delay: 30,
            msg: Some("System maintenance shutdown".to_string()),
            force: false,
            test: false,
        };

        let result = execute_shutdown_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_test_mode() {
        let task = ShutdownTask {
            description: None,
            delay: 10,
            msg: Some("Test shutdown".to_string()),
            force: true,
            test: true,
        };

        let result = execute_shutdown_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_zero_delay() {
        let task = ShutdownTask {
            description: None,
            delay: 0, // Immediate shutdown
            msg: None,
            force: false,
            test: true,
        };

        let result = execute_shutdown_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_very_long_delay() {
        let task = ShutdownTask {
            description: None,
            delay: 604800, // 1 week - very long delay
            msg: Some("Extended maintenance shutdown".to_string()),
            force: false,
            test: true,
        };

        let result = execute_shutdown_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_empty_message() {
        let task = ShutdownTask {
            description: None,
            delay: 5,
            msg: Some("".to_string()), // Empty message
            force: false,
            test: true,
        };

        let result = execute_shutdown_task(&task, true).await;
        assert!(result.is_ok());
    }
}
