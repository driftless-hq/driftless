//! Line in file task executor
//!
//! Handles ensuring specific lines are present or absent in files.
//!
//! # Examples
//!
//! ## Add a line to a file
//!
//! This example adds a line to /etc/hosts.
//!
//! **YAML Format:**
//! ```yaml
//! - type: lineinfile
//!   description: "Add localhost entry to hosts file"
//!   path: /etc/hosts
//!   state: present
//!   line: "127.0.0.1 localhost"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "lineinfile",
//!   "description": "Add localhost entry to hosts file",
//!   "path": "/etc/hosts",
//!   "state": "present",
//!   "line": "127.0.0.1 localhost"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "lineinfile"
//! description = "Add localhost entry to hosts file"
//! path = "/etc/hosts"
//! state = "present"
//! line = "127.0.0.1 localhost"
//! ```
//!
//! ## Replace a line using regex
//!
//! This example replaces a line matching a pattern.
//!
//! **YAML Format:**
//! ```yaml
//! - type: lineinfile
//!   description: "Update SSH port configuration"
//!   path: /etc/ssh/sshd_config
//!   state: present
//!   regexp: "^#?Port .*"
//!   line: "Port 22"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "lineinfile",
//!   "description": "Update SSH port configuration",
//!   "path": "/etc/ssh/sshd_config",
//!   "state": "present",
//!   "regexp": "^#?Port .*",
//!   "line": "Port 22"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "lineinfile"
//! description = "Update SSH port configuration"
//! path = "/etc/ssh/sshd_config"
//! state = "present"
//! regexp = "^#?Port .*"
//! line = "Port 22"
//! ```
//!
//! ## Insert line after a pattern
//!
//! This example inserts a line after a specific pattern.
//!
//! **YAML Format:**
//! ```yaml
//! - type: lineinfile
//!   description: "Add include directive after main config"
//!   path: /etc/nginx/nginx.conf
//!   state: present
//!   line: "include /etc/nginx/sites-enabled/*;"
//!   insertafter: "http \{"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "lineinfile",
//!   "description": "Add include directive after main config",
//!   "path": "/etc/nginx/nginx.conf",
//!   "state": "present",
//!   "line": "include /etc/nginx/sites-enabled/*;",
//!   "insertafter": "http \{"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "lineinfile"
//! description = "Add include directive after main config"
//! path = "/etc/nginx/nginx.conf"
//! state = "present"
//! line = "include /etc/nginx/sites-enabled/*;"
//! insertafter = "http \{"
//! ```

/// Line in file state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineInFileState {
    /// Ensure line is present
    Present,
    /// Ensure line is absent
    Absent,
}

/// Ensure line in file task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LineInFileTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to the file
    pub path: String,
    /// Line state
    pub state: LineInFileState,
    /// The line content
    pub line: String,
    /// Regular expression to match existing line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<String>,
    /// Insert after this line (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertafter: Option<String>,
    /// Insert before this line (regex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insertbefore: Option<String>,
    /// Create file if it doesn't exist
    #[serde(default)]
    pub create: bool,
    /// Backup file before modification
    #[serde(default)]
    pub backup: bool,
}

use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

/// Execute a lineinfile task
pub async fn execute_lineinfile_task(task: &LineInFileTask, dry_run: bool) -> Result<()> {
    match task.state {
        LineInFileState::Present => ensure_line_present(task, dry_run).await,
        LineInFileState::Absent => ensure_line_absent(task, dry_run).await,
    }
}

/// Ensure line is present in file
async fn ensure_line_present(task: &LineInFileTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    // Read existing file content
    let content = if path.exists() {
        fs::read_to_string(path).with_context(|| format!("Failed to read file {}", task.path))?
    } else if task.create {
        String::new()
    } else {
        return Err(anyhow::anyhow!(
            "File does not exist and create=false: {}",
            task.path
        ));
    };

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut new_lines = lines.clone();
    let mut line_found = false;
    let mut insert_index = None;

    // Check if line already exists
    for (i, line) in lines.iter().enumerate() {
        if matches_line(line, task)? {
            line_found = true;
            // If we have a regexp match, replace the line
            if task.regexp.is_some() {
                new_lines[i] = task.line.clone();
            }
            break;
        }
    }

    if !line_found {
        // Line doesn't exist, need to add it
        if let Some(regexp) = &task.regexp {
            // Find insertion point based on regexp
            let re = Regex::new(regexp).with_context(|| format!("Invalid regexp: {}", regexp))?;

            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    insert_index = Some(i + 1); // Insert after matching line
                    break;
                }
            }
        } else if let Some(insertafter) = &task.insertafter {
            // Find insertion point after specified line
            let re = Regex::new(insertafter)
                .with_context(|| format!("Invalid insertafter regexp: {}", insertafter))?;

            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    insert_index = Some(i + 1);
                    break;
                }
            }
        } else if let Some(insertbefore) = &task.insertbefore {
            // Find insertion point before specified line
            let re = Regex::new(insertbefore)
                .with_context(|| format!("Invalid insertbefore regexp: {}", insertbefore))?;

            for (i, line) in lines.iter().enumerate() {
                if re.is_match(line) {
                    insert_index = Some(i);
                    break;
                }
            }
        }

        // Insert the line at the determined position (or at end if no position found)
        let insert_pos = insert_index.unwrap_or(new_lines.len());
        new_lines.insert(insert_pos, task.line.clone());
    }

    // Check if content has changed
    let new_content = new_lines.join("\n") + if new_lines.is_empty() { "" } else { "\n" };
    if content == new_content {
        println!("Line already present in {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would ensure line in file: {}", task.path);
        if line_found && task.regexp.is_some() {
            println!("  (would replace existing line)");
        } else if !line_found {
            println!("  (would add new line)");
        }
    } else {
        // Backup file if requested
        if task.backup && path.exists() {
            let backup_path = format!("{}.backup", task.path);
            fs::copy(&task.path, &backup_path)
                .with_context(|| format!("Failed to backup {} to {}", task.path, backup_path))?;
            println!("Backed up {} to {}", task.path, backup_path);
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directories for {}", task.path)
            })?;
        }

        // Write new content
        fs::write(&task.path, new_content)
            .with_context(|| format!("Failed to write to file {}", task.path))?;

        if line_found && task.regexp.is_some() {
            println!("Replaced line in {}", task.path);
        } else if !line_found {
            println!("Added line to {}", task.path);
        } else {
            println!("Line already present in {}", task.path);
        }
    }

    Ok(())
}

/// Ensure line is absent from file
async fn ensure_line_absent(task: &LineInFileTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    if !path.exists() {
        println!("File does not exist: {}", task.path);
        return Ok(());
    }

    // Read existing file content
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file {}", task.path))?;

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut new_lines = Vec::new();
    let mut line_removed = false;

    // Filter out matching lines
    for line in lines {
        if matches_line(&line, task)? {
            line_removed = true;
            continue; // Skip this line
        }
        new_lines.push(line);
    }

    if !line_removed {
        println!("Line not found in {}", task.path);
        return Ok(());
    }

    let new_content = new_lines.join("\n") + if new_lines.is_empty() { "" } else { "\n" };

    if dry_run {
        println!("Would remove line from file: {}", task.path);
    } else {
        // Backup file if requested
        if task.backup {
            let backup_path = format!("{}.backup", task.path);
            fs::copy(&task.path, &backup_path)
                .with_context(|| format!("Failed to backup {} to {}", task.path, backup_path))?;
            println!("Backed up {} to {}", task.path, backup_path);
        }

        // Write new content
        fs::write(&task.path, new_content)
            .with_context(|| format!("Failed to write to file {}", task.path))?;

        println!("Removed line from {}", task.path);
    }

    Ok(())
}

/// Check if a line matches the task criteria
fn matches_line(line: &str, task: &LineInFileTask) -> Result<bool> {
    if let Some(regexp) = &task.regexp {
        let re = Regex::new(regexp).with_context(|| format!("Invalid regexp: {}", regexp))?;
        Ok(re.is_match(line))
    } else {
        // Exact string match
        Ok(line == task.line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_lineinfile_add_line_dry_run() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "line1\nline2\n").unwrap();

        let task = LineInFileTask {
            description: None,
            path: file_path.clone(),
            state: LineInFileState::Present,
            line: "line3".to_string(),
            regexp: None,
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_lineinfile_task(&task, true).await;
        assert!(result.is_ok());

        // Verify file wasn't actually modified in dry run
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nline2\n");
    }

    #[tokio::test]
    async fn test_lineinfile_add_line_real() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "line1\nline2\n").unwrap();

        let task = LineInFileTask {
            description: None,
            path: file_path.clone(),
            state: LineInFileState::Present,
            line: "line3".to_string(),
            regexp: None,
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_lineinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nline2\nline3\n");
    }

    #[tokio::test]
    async fn test_lineinfile_replace_with_regexp() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "export PATH=/usr/bin\n").unwrap();

        let task = LineInFileTask {
            description: None,
            path: file_path.clone(),
            state: LineInFileState::Present,
            line: "export PATH=/usr/local/bin:/usr/bin".to_string(),
            regexp: Some(r"^export PATH=".to_string()),
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_lineinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "export PATH=/usr/local/bin:/usr/bin\n");
    }

    #[tokio::test]
    async fn test_lineinfile_remove_line() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "line1\nto_remove\nline3\n").unwrap();

        let task = LineInFileTask {
            description: None,
            path: file_path.clone(),
            state: LineInFileState::Absent,
            line: "to_remove".to_string(),
            regexp: None,
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_lineinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nline3\n");
    }

    #[tokio::test]
    async fn test_lineinfile_insert_after() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "line1\nmarker\nline3\n").unwrap();

        let task = LineInFileTask {
            description: None,
            path: file_path.clone(),
            state: LineInFileState::Present,
            line: "inserted_line".to_string(),
            regexp: None,
            insertafter: Some(r"^marker$".to_string()),
            insertbefore: None,
            create: false,
            backup: false,
        };

        let result = execute_lineinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "line1\nmarker\ninserted_line\nline3\n");
    }

    #[tokio::test]
    async fn test_lineinfile_create_file() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string() + ".new";
        // Don't create the file initially

        let task = LineInFileTask {
            description: None,
            path: file_path.clone(),
            state: LineInFileState::Present,
            line: "new_file_line".to_string(),
            regexp: None,
            insertafter: None,
            insertbefore: None,
            create: true, // Allow creating the file
            backup: false,
        };

        let result = execute_lineinfile_task(&task, false).await;
        assert!(result.is_ok());

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "new_file_line\n");
    }

    #[test]
    fn test_matches_line_exact() {
        let task = LineInFileTask {
            description: None,
            path: "test".to_string(),
            state: LineInFileState::Present,
            line: "exact match".to_string(),
            regexp: None,
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        assert!(matches_line("exact match", &task).unwrap());
        assert!(!matches_line("different", &task).unwrap());
    }

    #[test]
    fn test_matches_line_regexp() {
        let task = LineInFileTask {
            description: None,
            path: "test".to_string(),
            state: LineInFileState::Present,
            line: "dummy".to_string(),
            regexp: Some(r"^export \w+=".to_string()),
            insertafter: None,
            insertbefore: None,
            create: false,
            backup: false,
        };

        assert!(matches_line("export PATH=/bin", &task).unwrap());
        assert!(!matches_line("not an export", &task).unwrap());
    }
}
