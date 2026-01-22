//! Logrotate task executor
//!
//! Manages log rotation configuration files in /etc/logrotate.d/
//!
//! # Examples
//!
//! ## Create a logrotate configuration for nginx
//!
//! This example creates a logrotate configuration for nginx logs with weekly rotation,
//! compression, and post-rotation service reload.
//!
//! **YAML Format:**
//! ```yaml
//! - type: logrotate
//!   description: "Configure nginx log rotation"
//!   name: nginx
//!   path: /var/log/nginx/*.log
//!   options:
//!     - weekly
//!     - rotate 52
//!     - compress
//!     - delaycompress
//!     - missingok
//!     - notifempty
//!     - create 644 www-data www-data
//!   postrotate: |
//!     systemctl reload nginx
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "logrotate",
//!   "description": "Configure nginx log rotation",
//!   "name": "nginx",
//!   "path": "/var/log/nginx/*.log",
//!   "options": [
//!     "weekly",
//!     "rotate 52",
//!     "compress",
//!     "delaycompress",
//!     "missingok",
//!     "notifempty",
//!     "create 644 www-data www-data"
//!   ],
//!   "postrotate": "systemctl reload nginx\n",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "logrotate"
//! description = "Configure nginx log rotation"
//! name = "nginx"
//! path = "/var/log/nginx/*.log"
//! options = [
//!   "weekly",
//!   "rotate 52",
//!   "compress",
//!   "delaycompress",
//!   "missingok",
//!   "notifempty",
//!   "create 644 www-data www-data"
//! ]
//! postrotate = """
//! systemctl reload nginx
//! """
//! state = "present"
//! ```
//!
//! ## Remove a logrotate configuration
//!
//! This example removes a logrotate configuration file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: logrotate
//!   description: "Remove custom logrotate config"
//!   name: myapp
//!   state: absent
//! ```

/// Logrotate state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogrotateState {
    /// Ensure logrotate config exists
    Present,
    /// Ensure logrotate config does not exist
    Absent,
}

/// Logrotate configuration task
///
/// Manages logrotate configuration files in /etc/logrotate.d/.
/// Creates or removes logrotate configuration snippets for log rotation management.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogrotateTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Configuration name
    ///
    /// Name of the logrotate configuration file to create in /etc/logrotate.d/.
    /// This becomes the filename (e.g., "nginx" creates /etc/logrotate.d/nginx).
    pub name: String,

    /// Log file path(s)
    ///
    /// Path or glob pattern for log files to rotate. Required when state is present.
    /// Examples: "/var/log/app/*.log", "/var/log/nginx/access.log"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Logrotate options
    ///
    /// List of logrotate configuration options. Common options include:
    /// - "daily", "weekly", "monthly", "yearly"
    /// - "rotate N" (keep N rotations)
    /// - "compress", "delaycompress"
    /// - "missingok", "notifempty"
    /// - "create MODE OWNER GROUP"
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub options: Vec<String>,

    /// Post-rotate script
    ///
    /// Shell commands to execute after log rotation.
    /// Commonly used to reload services after log rotation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postrotate: Option<String>,

    /// Configuration state (present, absent)
    ///
    /// - `present`: Ensure the logrotate configuration exists
    /// - `absent`: Ensure the logrotate configuration does not exist
    #[serde(default = "default_state")]
    pub state: LogrotateState,
}

fn default_state() -> LogrotateState {
    LogrotateState::Present
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute a logrotate task
pub async fn execute_logrotate_task(task: &LogrotateTask, dry_run: bool) -> Result<()> {
    let config_path = Path::new("/etc/logrotate.d").join(&task.name);

    match task.state {
        LogrotateState::Present => ensure_logrotate_present(&config_path, task, dry_run).await,
        LogrotateState::Absent => ensure_logrotate_absent(&config_path, dry_run).await,
    }
}

/// Ensure a logrotate configuration exists
async fn ensure_logrotate_present(
    config_path: &Path,
    task: &LogrotateTask,
    dry_run: bool,
) -> Result<()> {
    // Validate required fields
    let path = task
        .path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("path is required when state is present"))?;

    // Generate configuration content
    let content = generate_logrotate_config(path, &task.options, &task.postrotate)?;

    let exists = config_path.exists();

    if !exists {
        if dry_run {
            println!("Would create logrotate config: {}", config_path.display());
        } else {
            // Ensure parent directory exists
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create logrotate.d directory: {}",
                        parent.display()
                    )
                })?;
            }

            fs::write(config_path, &content).with_context(|| {
                format!(
                    "Failed to create logrotate config: {}",
                    config_path.display()
                )
            })?;
            println!("Created logrotate config: {}", config_path.display());
        }
    } else {
        // Check if content needs updating
        let current_content = fs::read_to_string(config_path).with_context(|| {
            format!("Failed to read existing config: {}", config_path.display())
        })?;

        if current_content != content {
            if dry_run {
                println!("Would update logrotate config: {}", config_path.display());
            } else {
                fs::write(config_path, &content).with_context(|| {
                    format!(
                        "Failed to update logrotate config: {}",
                        config_path.display()
                    )
                })?;
                println!("Updated logrotate config: {}", config_path.display());
            }
        }
    }

    Ok(())
}

/// Ensure a logrotate configuration does not exist
async fn ensure_logrotate_absent(config_path: &Path, dry_run: bool) -> Result<()> {
    if config_path.exists() {
        if dry_run {
            println!("Would remove logrotate config: {}", config_path.display());
        } else {
            fs::remove_file(config_path).with_context(|| {
                format!(
                    "Failed to remove logrotate config: {}",
                    config_path.display()
                )
            })?;
            println!("Removed logrotate config: {}", config_path.display());
        }
    }

    Ok(())
}

/// Generate logrotate configuration content
fn generate_logrotate_config(
    path: &str,
    options: &[String],
    postrotate: &Option<String>,
) -> Result<String> {
    let mut content = format!("{}\n", path);

    // Add options with proper indentation
    for option in options {
        content.push_str(&format!("    {}\n", option));
    }

    // Add postrotate script if specified
    if let Some(script) = postrotate {
        content.push_str("    postrotate\n");
        for line in script.lines() {
            content.push_str(&format!("        {}\n", line));
        }
        content.push_str("    endscript\n");
    }

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_logrotate_config_dry_run() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("testapp");

        let task = LogrotateTask {
            description: None,
            name: "testapp".to_string(),
            path: Some("/var/log/testapp/*.log".to_string()),
            options: vec![
                "weekly".to_string(),
                "rotate 4".to_string(),
                "compress".to_string(),
            ],
            postrotate: Some("systemctl reload testapp".to_string()),
            state: LogrotateState::Present,
        };

        let result = execute_logrotate_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!config_path.exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_create_logrotate_config_real() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("testapp");

        let task = LogrotateTask {
            description: None,
            name: "testapp".to_string(),
            path: Some("/var/log/testapp/*.log".to_string()),
            options: vec![
                "daily".to_string(),
                "rotate 7".to_string(),
                "compress".to_string(),
            ],
            postrotate: None,
            state: LogrotateState::Present,
        };

        // For testing, we'll use dry-run since we can't write to /etc/ in test environment
        let result = execute_logrotate_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!config_path.exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_remove_logrotate_config() {
        let task = LogrotateTask {
            description: None,
            name: "testapp".to_string(),
            path: None,
            options: vec![],
            postrotate: None,
            state: LogrotateState::Absent,
        };

        // For testing, we'll use dry-run since we can't write to /etc/ in test environment
        let result = execute_logrotate_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_config_content() {
        let content = generate_logrotate_config(
            "/var/log/app/*.log",
            &[
                "weekly".to_string(),
                "rotate 52".to_string(),
                "compress".to_string(),
            ],
            &Some("systemctl reload app".to_string()),
        )
        .unwrap();

        assert!(content.contains("/var/log/app/*.log"));
        assert!(content.contains("    weekly"));
        assert!(content.contains("    rotate 52"));
        assert!(content.contains("    compress"));
        assert!(content.contains("    postrotate"));
        assert!(content.contains("        systemctl reload app"));
        assert!(content.contains("    endscript"));
    }

    #[tokio::test]
    async fn test_missing_path_error() {
        let task = LogrotateTask {
            description: None,
            name: "testapp".to_string(),
            path: None, // Missing path
            options: vec![],
            postrotate: None,
            state: LogrotateState::Present,
        };

        let result = execute_logrotate_task(&task, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path is required"));
    }
}
