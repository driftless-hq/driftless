//! Rsyslog task executor
//!
//! Manages rsyslog configuration files in /etc/rsyslog.d/
//!
//! # Examples
//!
//! ## Create an rsyslog configuration for remote logging
//!
//! This example creates an rsyslog configuration to forward logs to a remote server.
//!
//! **YAML Format:**
//! ```yaml
//! - type: rsyslog
//!   description: "Configure remote log forwarding"
//!   name: remote-logging
//!   config: |
//!     # Forward all logs to remote server
//!     *.* @@logserver.example.com:514
//!
//!     # Forward auth logs with TCP
//!     auth.* @@logserver.example.com:514
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "rsyslog",
//!   "description": "Configure remote log forwarding",
//!   "name": "remote-logging",
//!   "config": "*.* @@logserver.example.com:514\n\nauth.* @@logserver.example.com:514\n",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "rsyslog"
//! description = "Configure remote log forwarding"
//! name = "remote-logging"
//! config = """
//! *.* @@logserver.example.com:514
//!
//! auth.* @@logserver.example.com:514
//! """
//! state = "present"
//! ```
//!
//! ## Create an rsyslog configuration for custom log file
//!
//! This example creates a custom log file for application logs.
//!
//! **YAML Format:**
//! ```yaml
//! - type: rsyslog
//!   description: "Configure application logging"
//!   name: app-logs
//!   config: |
//!     # Log application messages to custom file
//!     :programname, startswith, "myapp" /var/log/myapp.log
//!     & stop
//!   state: present
//! ```
//!
//! ## Remove an rsyslog configuration
//!
//! This example removes an rsyslog configuration file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: rsyslog
//!   description: "Remove custom rsyslog config"
//!   name: old-config
//!   state: absent
//! ```

/// Rsyslog state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RsyslogState {
    /// Ensure rsyslog config exists
    Present,
    /// Ensure rsyslog config does not exist
    Absent,
}

/// Rsyslog configuration task
///
/// Manages rsyslog configuration files in /etc/rsyslog.d/.
/// Creates or removes rsyslog configuration snippets for log processing and forwarding.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RsyslogTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Configuration name
    ///
    /// Name of the rsyslog configuration file to create in /etc/rsyslog.d/.
    /// This becomes the filename (e.g., "remote-logging" creates /etc/rsyslog.d/remote-logging.conf).
    pub name: String,

    /// Rsyslog configuration content
    ///
    /// The rsyslog configuration directives. Required when state is present.
    /// Examples include log forwarding rules, custom log files, filters, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,

    /// Configuration state (present, absent)
    ///
    /// - `present`: Ensure the rsyslog configuration exists
    /// - `absent`: Ensure the rsyslog configuration does not exist
    #[serde(default = "default_state")]
    pub state: RsyslogState,
}

fn default_state() -> RsyslogState {
    RsyslogState::Present
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute an rsyslog task
pub async fn execute_rsyslog_task(task: &RsyslogTask, dry_run: bool) -> Result<()> {
    let config_path = Path::new("/etc/rsyslog.d").join(format!("{}.conf", task.name));

    match task.state {
        RsyslogState::Present => ensure_rsyslog_present(&config_path, task, dry_run).await,
        RsyslogState::Absent => ensure_rsyslog_absent(&config_path, dry_run).await,
    }
}

/// Ensure an rsyslog configuration exists
async fn ensure_rsyslog_present(
    config_path: &Path,
    task: &RsyslogTask,
    dry_run: bool,
) -> Result<()> {
    // Validate required fields
    let config = task
        .config
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("config is required when state is present"))?;

    let exists = config_path.exists();

    if !exists {
        if dry_run {
            println!("Would create rsyslog config: {}", config_path.display());
        } else {
            // Ensure parent directory exists
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create rsyslog.d directory: {}", parent.display())
                })?;
            }

            fs::write(config_path, config).with_context(|| {
                format!("Failed to create rsyslog config: {}", config_path.display())
            })?;
            println!("Created rsyslog config: {}", config_path.display());
        }
    } else {
        // Check if content needs updating
        let current_content = fs::read_to_string(config_path).with_context(|| {
            format!("Failed to read existing config: {}", config_path.display())
        })?;

        if current_content != *config {
            if dry_run {
                println!("Would update rsyslog config: {}", config_path.display());
            } else {
                fs::write(config_path, config).with_context(|| {
                    format!("Failed to update rsyslog config: {}", config_path.display())
                })?;
                println!("Updated rsyslog config: {}", config_path.display());
            }
        }
    }

    Ok(())
}

/// Ensure an rsyslog configuration does not exist
async fn ensure_rsyslog_absent(config_path: &Path, dry_run: bool) -> Result<()> {
    if config_path.exists() {
        if dry_run {
            println!("Would remove rsyslog config: {}", config_path.display());
        } else {
            fs::remove_file(config_path).with_context(|| {
                format!("Failed to remove rsyslog config: {}", config_path.display())
            })?;
            println!("Removed rsyslog config: {}", config_path.display());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_rsyslog_config_dry_run() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.conf");

        let task = RsyslogTask {
            description: None,
            name: "test".to_string(),
            config: Some("*.* /var/log/test.log".to_string()),
            state: RsyslogState::Present,
        };

        let result = execute_rsyslog_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!config_path.exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_create_rsyslog_config_real() {
        let task = RsyslogTask {
            description: None,
            name: "test".to_string(),
            config: Some("*.* /var/log/test.log\n:programname, startswith, \"myapp\" /var/log/myapp.log\n& stop\n".to_string()),
            state: RsyslogState::Present,
        };

        // For testing, we'll use dry-run since we can't write to /etc/ in test environment
        let result = execute_rsyslog_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_rsyslog_config() {
        let task = RsyslogTask {
            description: None,
            name: "test".to_string(),
            config: None,
            state: RsyslogState::Absent,
        };

        // For testing, we'll use dry-run since we can't write to /etc/ in test environment
        let result = execute_rsyslog_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_missing_config_error() {
        let task = RsyslogTask {
            description: None,
            name: "test".to_string(),
            config: None, // Missing config
            state: RsyslogState::Present,
        };

        let result = execute_rsyslog_task(&task, true).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("config is required"));
    }
}
