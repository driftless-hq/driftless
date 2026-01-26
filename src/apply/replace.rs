//! Replace text in files task executor
//!
//! Handles replacing text in files using regex patterns or string matching.
//!
//! # Examples
//!
//! ## Replace text using regex
//!
//! This example replaces text in a configuration file using a regular expression.
//!
//! **YAML Format:**
//! ```yaml
//! - type: replace
//!   description: "Update database host"
//!   path: /etc/myapp/config.ini
//!   state: present
//!   regexp: '^db_host\s*=\s*.*$'
//!   replace: 'db_host = newdb.example.com'
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "replace",
//!   "description": "Update database host",
//!   "path": "/etc/myapp/config.ini",
//!   "state": "present",
//!   "regexp": "^db_host\\s*=\\s*.*$",
//!   "replace": "db_host = newdb.example.com"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "replace"
//! description = "Update database host"
//! path = "/etc/myapp/config.ini"
//! state = "present"
//! regexp = "^db_host\\s*=\\s*.*$"
//! replace = "db_host = newdb.example.com"
//! ```
//!
//! ## Replace string literal
//!
//! This example replaces a specific string in a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: replace
//!   description: "Update version number"
//!   path: /opt/myapp/version.txt
//!   state: present
//!   before: 'version = "1.0.0"'
//!   replace: 'version = "1.1.0"'
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "replace",
//!   "description": "Update version number",
//!   "path": "/opt/myapp/version.txt",
//!   "state": "present",
//!   "before": "version = \"1.0.0\"",
//!   "replace": "version = \"1.1.0\""
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "replace"
//! description = "Update version number"
//! path = "/opt/myapp/version.txt"
//! state = "present"
//! before = 'version = "1.0.0"'
//! replace = 'version = "1.1.0"'
//! ```
//!
//! ## Replace with backup
//!
//! This example replaces text and creates a backup of the original file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: replace
//!   description: "Update configuration with backup"
//!   path: /etc/httpd/httpd.conf
//!   state: present
//!   regexp: '^Listen 80$'
//!   replace: 'Listen 8080'
//!   backup: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "replace",
//!   "description": "Update configuration with backup",
//!   "path": "/etc/httpd/httpd.conf",
//!   "state": "present",
//!   "regexp": "^Listen 80$",
//!   "replace": "Listen 8080",
//!   "backup": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "replace"
//! description = "Update configuration with backup"
//! path = "/etc/httpd/httpd.conf"
//! state = "present"
//! regexp = "^Listen 80$"
//! replace = "Listen 8080"
//! backup = true
//! ```
//!
//! ## Replace all occurrences
//!
//! This example replaces all occurrences of a pattern in a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: replace
//!   description: "Update all IP addresses"
//!   path: /etc/hosts
//!   state: present
//!   regexp: '192\.168\.1\.\d+'
//!   replace: '10.0.0.100'
//!   replace_all: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "replace",
//!   "description": "Update all IP addresses",
//!   "path": "/etc/hosts",
//!   "state": "present",
//!   "regexp": "192\\.168\\.1\\.\\d+",
//!   "replace": "10.0.0.100",
//!   "replace_all": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "replace"
//! description = "Update all IP addresses"
//! path = "/etc/hosts"
//! state = "present"
//! regexp = "192\\.168\\.1\\.\\d+"
//! replace = "10.0.0.100"
//! replace_all = true
//! ```

/// Replace state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplaceState {
    /// Ensure replacement is made
    Present,
    /// Ensure replacement is not made (revert)
    Absent,
}

/// Replace text in files task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplaceTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to the file
    pub path: String,
    /// Replace state
    pub state: ReplaceState,
    /// Regular expression to match
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<String>,
    /// String to match (alternative to regexp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Replacement string
    pub replace: String,
    /// Replace all occurrences
    #[serde(default = "crate::apply::default_true")]
    pub replace_all: bool,
    /// Backup file before modification
    #[serde(default)]
    pub backup: bool,
}

use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Execute a replace task
pub async fn execute_replace_task(task: &ReplaceTask, dry_run: bool) -> Result<()> {
    match task.state {
        ReplaceState::Present => ensure_replacement_present(task, dry_run).await,
        ReplaceState::Absent => ensure_replacement_absent(task, dry_run).await,
    }
}

/// Ensure replacement is present (perform replacement)
async fn ensure_replacement_present(task: &ReplaceTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", task.path));
    }

    // Read file content
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file {}", task.path))?;

    // Create replacement pattern
    let (pattern, replacement) = if let Some(regexp) = &task.regexp {
        (regexp.clone(), task.replace.clone())
    } else if let Some(before) = &task.before {
        // Escape special regex characters in the literal string
        let escaped = regex::escape(before);
        (escaped, task.replace.clone())
    } else {
        return Err(anyhow::anyhow!(
            "Either 'regexp' or 'before' must be specified"
        ));
    };

    // Compile regex
    let re = Regex::new(&pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;

    // Perform replacement
    let new_content = if task.replace_all {
        re.replace_all(&content, replacement.as_str())
    } else {
        re.replace(&content, replacement.as_str())
    };

    let new_content_str = new_content.to_string();

    // Check if content changed
    if content == new_content_str {
        println!("No replacements needed in {}", task.path);
        return Ok(());
    }

    if dry_run {
        let replacement_count = if task.replace_all {
            re.find_iter(&content).count()
        } else {
            re.find(&content).is_some() as usize
        };
        println!(
            "Would replace {} occurrence(s) in file: {}",
            replacement_count, task.path
        );
    } else {
        // Backup file if requested
        if task.backup {
            let backup_path = format!("{}.backup", task.path);
            fs::copy(&task.path, &backup_path)
                .with_context(|| format!("Failed to backup {} to {}", task.path, backup_path))?;
            println!("Backed up {} to {}", task.path, backup_path);
        }

        // Write new content
        fs::write(&task.path, &new_content_str)
            .with_context(|| format!("Failed to write to file {}", task.path))?;

        let replacement_count = if task.replace_all {
            re.find_iter(&content).count()
        } else {
            re.find(&content).is_some() as usize
        };
        println!(
            "Replaced {} occurrence(s) in {}",
            replacement_count, task.path
        );
    }

    Ok(())
}

/// Ensure replacement is absent (undo replacement)
async fn ensure_replacement_absent(task: &ReplaceTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    if !path.exists() {
        println!("File does not exist: {}", task.path);
        return Ok(());
    }

    // Read file content
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file {}", task.path))?;

    // For "absent" state, we need to replace the replacement string back with the original
    // This is more complex and would require tracking what was replaced
    // For simplicity, we'll implement a basic version that looks for exact matches

    if let Some(before) = &task.before {
        // Simple string replacement: replace 'replace' with 'before'
        let new_content = content.replace(&task.replace, before);

        if content == new_content {
            println!("No replacements to undo in {}", task.path);
            return Ok(());
        }

        if dry_run {
            println!("Would undo replacement in file: {}", task.path);
        } else {
            // Backup file if requested
            if task.backup {
                let backup_path = format!("{}.backup", task.path);
                fs::copy(&task.path, &backup_path).with_context(|| {
                    format!("Failed to backup {} to {}", task.path, backup_path)
                })?;
                println!("Backed up {} to {}", task.path, backup_path);
            }

            // Write new content
            fs::write(&task.path, &new_content)
                .with_context(|| format!("Failed to write to file {}", task.path))?;

            println!("Undid replacement in {}", task.path);
        }
    } else {
        // For regex replacements, we need to undo by restoring from backup
        // or implementing a reverse replacement logic
        if task.backup {
            // Try to restore from backup
            let backup_path = format!("{}.backup", task.path);
            if Path::new(&backup_path).exists() {
                if dry_run {
                    println!("Would restore {} from backup {}", task.path, backup_path);
                } else {
                    fs::copy(&backup_path, &task.path).with_context(|| {
                        format!("Failed to restore {} from {}", task.path, backup_path)
                    })?;
                    println!("Restored {} from backup", task.path);
                }
            } else {
                println!(
                    "No backup found for {} - cannot undo regex replacement",
                    task.path
                );
            }
        } else {
            // Without backup, we cannot reliably undo regex replacements
            println!(
                "Cannot undo regex replacement for {} - no backup available. Consider using backup=true",
                task.path
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_replace_with_regexp_dry_run() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "version: 1.0.0\nversion: 2.0.0\n").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path.clone(),
            state: ReplaceState::Present,
            regexp: Some(r"version: \d+\.\d+\.\d+".to_string()),
            before: None,
            replace: "version: 3.0.0".to_string(),
            replace_all: true,
            backup: false,
        };

        let result = execute_replace_task(&task, true).await;
        assert!(result.is_ok());

        // Verify file wasn't modified in dry run
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("version: 1.0.0"));
        assert!(content.contains("version: 2.0.0"));
    }

    #[tokio::test]
    async fn test_replace_with_regexp_real() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "version: 1.0.0\nversion: 2.0.0\n").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path.clone(),
            state: ReplaceState::Present,
            regexp: Some(r"version: \d+\.\d+\.\d+".to_string()),
            before: None,
            replace: "version: 3.0.0".to_string(),
            replace_all: true,
            backup: false,
        };

        let result = execute_replace_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "version: 3.0.0\nversion: 3.0.0\n");
    }

    #[tokio::test]
    async fn test_replace_with_string() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "Hello world\nHello universe\n").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path.clone(),
            state: ReplaceState::Present,
            regexp: None,
            before: Some("Hello world".to_string()),
            replace: "Hi world".to_string(),
            replace_all: false,
            backup: false,
        };

        let result = execute_replace_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hi world\nHello universe\n");
    }

    #[tokio::test]
    async fn test_replace_no_matches() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "some content\n").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path.clone(),
            state: ReplaceState::Present,
            regexp: Some(r"notfound".to_string()),
            before: None,
            replace: "replacement".to_string(),
            replace_all: true,
            backup: false,
        };

        let result = execute_replace_task(&task, false).await;
        assert!(result.is_ok());

        // Content should be unchanged
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "some content\n");
    }

    #[tokio::test]
    async fn test_replace_undo_with_string() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "Hi world\nHello universe\n").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path.clone(),
            state: ReplaceState::Absent,
            regexp: None,
            before: Some("Hello world".to_string()),
            replace: "Hi world".to_string(),
            replace_all: false,
            backup: false,
        };

        let result = execute_replace_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello world\nHello universe\n");
    }

    #[tokio::test]
    async fn test_replace_invalid_regexp() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "content").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path,
            state: ReplaceState::Present,
            regexp: Some(r"[invalid".to_string()), // Invalid regex
            before: None,
            replace: "replacement".to_string(),
            replace_all: false,
            backup: false,
        };

        let result = execute_replace_task(&task, true).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid regex pattern"));
    }

    #[tokio::test]
    async fn test_replace_missing_pattern() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "content").unwrap();

        let task = ReplaceTask {
            description: None,
            path: file_path,
            state: ReplaceState::Present,
            regexp: None,
            before: None, // Neither regexp nor before specified
            replace: "replacement".to_string(),
            replace_all: false,
            backup: false,
        };

        let result = execute_replace_task(&task, true).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Either 'regexp' or 'before' must be specified"));
    }
}
