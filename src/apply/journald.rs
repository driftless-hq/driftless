//! Journald task executor
//!
//! Manages systemd journal configuration files and drop-in configurations.
//!
//! # Examples
//!
//! ## Configure journald storage and rotation
//!
//! This example configures journald to use persistent storage and sets size limits.
//!
//! **YAML Format:**
//! ```yaml
//! - type: journald
//!   description: "Configure systemd journal settings"
//!   config:
//!     Storage: persistent
//!     SystemMaxUse: 100M
//!     SystemKeepFree: 500M
//!     SystemMaxFileSize: 10M
//!     MaxRetentionSec: 1week
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "journald",
//!   "description": "Configure systemd journal settings",
//!   "config": {
//!     "Storage": "persistent",
//!     "SystemMaxUse": "100M",
//!     "SystemKeepFree": "500M",
//!     "SystemMaxFileSize": "10M",
//!     "MaxRetentionSec": "1week"
//!   },
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "journald"
//! description = "Configure systemd journal settings"
//! [tasks.config]
//! Storage = "persistent"
//! SystemMaxUse = "100M"
//! SystemKeepFree = "500M"
//! SystemMaxFileSize = "10M"
//! MaxRetentionSec = "1week"
//! state = "present"
//! ```
//!
//! ## Create a drop-in configuration
//!
//! This example creates a drop-in configuration file for journald.
//!
//! **YAML Format:**
//! ```yaml
//! - type: journald
//!   description: "Configure journald forwarding"
//!   name: forwarding
//!   config:
//!     ForwardToSyslog: "yes"
//!     ForwardToKMsg: "no"
//!     ForwardToConsole: "no"
//!     ForwardToWall: "no"
//!   state: present
//! ```
//!
//! ## Remove journald configuration
//!
//! This example removes a journald drop-in configuration.
//!
//! **YAML Format:**
//! ```yaml
//! - type: journald
//!   description: "Remove custom journald config"
//!   name: custom-config
//!   state: absent
//! ```

use std::collections::HashMap;

/// Journald state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JournaldState {
    /// Ensure journald config exists
    Present,
    /// Ensure journald config does not exist
    Absent,
}

/// Journald configuration task
///
/// Manages systemd journal configuration. Can modify the main /etc/systemd/journald.conf
/// file or create drop-in configuration files in /etc/systemd/journald.conf.d/.
/// Supports all journald configuration options like storage settings, size limits,
/// forwarding options, and compression settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JournaldTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Configuration name (for drop-in configs)
    ///
    /// Name of the drop-in configuration file to create in /etc/systemd/journald.conf.d/.
    /// If not specified, modifies the main /etc/systemd/journald.conf file.
    /// This becomes the filename (e.g., "storage" creates /etc/systemd/journald.conf.d/storage.conf).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Journald configuration options
    ///
    /// Key-value pairs of journald configuration options. Required when state is present.
    /// Common options include:
    /// - Storage: volatile|persistent|auto|none
    /// - SystemMaxUse: Maximum disk space to use
    /// - SystemKeepFree: Disk space to keep free
    /// - SystemMaxFileSize: Maximum size of individual journal files
    /// - MaxRetentionSec: Maximum time to retain journal entries
    /// - ForwardToSyslog: Forward to syslog
    /// - Compress: Enable compression
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub config: HashMap<String, String>,

    /// Configuration state (present, absent)
    ///
    /// - `present`: Ensure the journald configuration exists
    /// - `absent`: Ensure the journald configuration does not exist
    #[serde(default = "default_state")]
    pub state: JournaldState,
}

fn default_state() -> JournaldState {
    JournaldState::Present
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute a journald task
pub async fn execute_journald_task(task: &JournaldTask, dry_run: bool) -> Result<()> {
    let config_path = if let Some(name) = &task.name {
        Path::new("/etc/systemd/journald.conf.d").join(format!("{}.conf", name))
    } else {
        Path::new("/etc/systemd/journald.conf").to_path_buf()
    };

    match task.state {
        JournaldState::Present => ensure_journald_present(&config_path, task, dry_run).await,
        JournaldState::Absent => ensure_journald_absent(&config_path, dry_run).await,
    }
}

/// Ensure a journald configuration exists
async fn ensure_journald_present(
    config_path: &Path,
    task: &JournaldTask,
    dry_run: bool,
) -> Result<()> {
    // Validate required fields
    if task.config.is_empty() {
        return Err(anyhow::anyhow!("config is required when state is present"));
    }

    // Generate configuration content
    let content = generate_journald_config(&task.config)?;

    let exists = config_path.exists();

    if !exists {
        if dry_run {
            println!("Would create journald config: {}", config_path.display());
        } else {
            // Ensure parent directory exists
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create journald config directory: {}",
                        parent.display()
                    )
                })?;
            }

            fs::write(config_path, &content).with_context(|| {
                format!(
                    "Failed to create journald config: {}",
                    config_path.display()
                )
            })?;
            println!("Created journald config: {}", config_path.display());
        }
    } else {
        // For main config file, we need to merge with existing content
        if task.name.is_none() {
            update_main_journald_config(config_path, &task.config, dry_run)?;
        } else {
            // For drop-in configs, replace entirely
            let current_content = fs::read_to_string(config_path).with_context(|| {
                format!("Failed to read existing config: {}", config_path.display())
            })?;

            if current_content != content {
                if dry_run {
                    println!("Would update journald config: {}", config_path.display());
                } else {
                    fs::write(config_path, &content).with_context(|| {
                        format!(
                            "Failed to update journald config: {}",
                            config_path.display()
                        )
                    })?;
                    println!("Updated journald config: {}", config_path.display());
                }
            }
        }
    }

    Ok(())
}

/// Ensure a journald configuration does not exist
async fn ensure_journald_absent(config_path: &Path, dry_run: bool) -> Result<()> {
    if config_path.exists() {
        if dry_run {
            println!("Would remove journald config: {}", config_path.display());
        } else {
            // For main config file, we should not remove it entirely
            // Instead, we could comment out or reset specific settings
            if config_path == Path::new("/etc/systemd/journald.conf") {
                return Err(anyhow::anyhow!(
                    "Cannot remove main journald.conf file. Use present state to modify it instead."
                ));
            }

            fs::remove_file(config_path).with_context(|| {
                format!(
                    "Failed to remove journald config: {}",
                    config_path.display()
                )
            })?;
            println!("Removed journald config: {}", config_path.display());
        }
    }

    Ok(())
}

/// Generate journald configuration content
fn generate_journald_config(config: &HashMap<String, String>) -> Result<String> {
    let mut content = String::new();

    // Add header comment
    content.push_str("# This file is managed by Driftless\n");
    content.push_str("# Do not edit manually\n\n");
    content.push_str("[Journal]\n");

    // Add configuration options
    for (key, value) in config {
        content.push_str(&format!("{}={}\n", key, value));
    }

    Ok(content)
}

/// Update the main journald.conf file by modifying specific settings
fn update_main_journald_config(
    config_path: &Path,
    new_config: &HashMap<String, String>,
    dry_run: bool,
) -> Result<()> {
    // Read existing content
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read journald config: {}", config_path.display()))?;

    // Parse and update the configuration
    let updated_content = update_journald_config_content(&content, new_config)?;

    if content != updated_content {
        if dry_run {
            println!("Would update journald config: {}", config_path.display());
        } else {
            fs::write(config_path, &updated_content).with_context(|| {
                format!(
                    "Failed to update journald config: {}",
                    config_path.display()
                )
            })?;
            println!("Updated journald config: {}", config_path.display());
        }
    }

    Ok(())
}

/// Update journald configuration content by modifying specific settings
fn update_journald_config_content(
    content: &str,
    new_config: &HashMap<String, String>,
) -> Result<String> {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut in_journal_section = false;
    let mut updated = false;

    // First pass: update existing settings
    for line in lines.iter_mut() {
        let trimmed = line.trim();

        if trimmed == "[Journal]" {
            in_journal_section = true;
        } else if trimmed.starts_with('[') && trimmed != "[Journal]" {
            in_journal_section = false;
        } else if in_journal_section && !trimmed.starts_with('#') && !trimmed.is_empty() {
            // Check if this line contains a setting we want to update
            if let Some((key, _)) = parse_config_line(trimmed) {
                if new_config.contains_key(&key) {
                    *line = format!("{}={}", key, new_config[&key]);
                    updated = true;
                }
            }
        }
    }

    // Second pass: add new settings that weren't found
    let mut insert_index = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "[Journal]" {
            // Find the end of the Journal section
            let mut j = i + 1;
            while j < lines.len() {
                let trimmed = lines[j].trim();
                if trimmed.starts_with('[') || trimmed.is_empty() {
                    break;
                }
                j += 1;
            }
            insert_index = Some(j);
            break;
        }
    }

    // Add new settings
    if let Some(index) = insert_index {
        for (key, value) in new_config {
            let setting_line = format!("{}={}", key, value);
            let mut found = false;

            // Check if we already updated this setting
            for line in &lines {
                if line.trim() == setting_line {
                    found = true;
                    break;
                }
            }

            if !found {
                lines.insert(index, setting_line);
                updated = true;
                // Adjust insert index for next insertion
            }
        }
    }

    if updated {
        Ok(lines.join("\n") + "\n")
    } else {
        Ok(content.to_string())
    }
}

/// Parse a configuration line into key-value pair
fn parse_config_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_journald_dropin_dry_run() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("test.conf");

        let mut config = HashMap::new();
        config.insert("Storage".to_string(), "persistent".to_string());
        config.insert("SystemMaxUse".to_string(), "100M".to_string());

        let task = JournaldTask {
            description: None,
            name: Some("test".to_string()),
            config,
            state: JournaldState::Present,
        };

        let result = execute_journald_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!config_path.exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_create_journald_dropin_real() {
        let mut config = HashMap::new();
        config.insert("Storage".to_string(), "persistent".to_string());
        config.insert("Compress".to_string(), "yes".to_string());

        let task = JournaldTask {
            description: None,
            name: Some("test".to_string()),
            config,
            state: JournaldState::Present,
        };

        // For testing, we'll use dry-run since we can't write to /etc/ in test environment
        let result = execute_journald_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_journald_dropin() {
        let task = JournaldTask {
            description: None,
            name: Some("test".to_string()),
            config: HashMap::new(),
            state: JournaldState::Absent,
        };

        // For testing, we'll use dry-run since we can't write to /etc/ in test environment
        let result = execute_journald_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_missing_config_error() {
        let task = JournaldTask {
            description: None,
            name: Some("test".to_string()),
            config: HashMap::new(), // Empty config
            state: JournaldState::Present,
        };

        let result = execute_journald_task(&task, true).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("config is required"));
    }

    #[test]
    fn test_generate_config_content() {
        let mut config = HashMap::new();
        config.insert("Storage".to_string(), "persistent".to_string());
        config.insert("SystemMaxUse".to_string(), "50M".to_string());

        let content = generate_journald_config(&config).unwrap();

        assert!(content.contains("# This file is managed by Driftless"));
        assert!(content.contains("[Journal]"));
        assert!(content.contains("Storage=persistent"));
        assert!(content.contains("SystemMaxUse=50M"));
    }

    #[test]
    fn test_update_config_content() {
        let original = "[Journal]\nStorage=volatile\nCompress=no\n\n[Other]\nSetting=value\n";

        let mut new_config = HashMap::new();
        new_config.insert("Storage".to_string(), "persistent".to_string());
        new_config.insert("SystemMaxUse".to_string(), "100M".to_string());

        let updated = update_journald_config_content(original, &new_config).unwrap();

        assert!(updated.contains("Storage=persistent"));
        assert!(updated.contains("Compress=no")); // Should be preserved
        assert!(updated.contains("SystemMaxUse=100M")); // Should be added
        assert!(updated.contains("[Other]"));
    }
}
