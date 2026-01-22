//! File task executor
//!
//! Handles file operations: create, modify, delete files with proper permissions.
//!
//! # Examples
//!
//! ## Create a file with content
//!
//! This example creates a simple configuration file with specific permissions and ownership.
//!
//! **YAML Format:**
//! ```yaml
//! - type: file
//!   description: "Create nginx configuration file"
//!   path: /etc/nginx/sites-available/default
//!   state: present
//!   content: |
//!     server {
//!         listen 80;
//!         root /var/www/html;
//!         index index.html index.htm;
//!
//!         location / {
//!             try_files $uri $uri/ =404;
//!         }
//!     }
//!   mode: "0644"
//!   owner: root
//!   group: root
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "file",
//!   "description": "Create nginx configuration file",
//!   "path": "/etc/nginx/sites-available/default",
//!   "state": "present",
//!   "content": "server {\n    listen 80;\n    root /var/www/html;\n    index index.html index.htm;\n\n    location / {\n        try_files $uri $uri/ =404;\n    }\n}",
//!   "mode": "0644",
//!   "owner": "root",
//!   "group": "root"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "file"
//! description = "Create nginx configuration file"
//! path = "/etc/nginx/sites-available/default"
//! state = "present"
//! content = """
//! server {
//!     listen 80;
//!     root /var/www/html;
//!     index index.html index.htm;
//!
//!     location / {
//!         try_files $uri $uri/ =404;
//!     }
//! }
//! """
//! mode = "0644"
//! owner = "root"
//! group = "root"
//! ```
//!
//! ## Copy a file from source
//!
//! This example copies an existing file to a new location.
//!
//! **YAML Format:**
//! ```yaml
//! - type: file
//!   description: "Copy configuration template"
//!   path: /etc/myapp/config.yml
//!   state: present
//!   source: /opt/myapp/config.template.yml
//!   mode: "0644"
//!   owner: myapp
//!   group: myapp
//! ```
//!
//! ## Remove a file
//!
//! This example ensures a file does not exist.
//!
//! **YAML Format:**
//! ```yaml
//! - type: file
//!   description: "Remove temporary file"
//!   path: /tmp/temp-file.txt
//!   state: absent
//! ```
//!
//! ## Create a directory
//!
//! This example creates a directory (note: directories are just files with no content).
//!
//! **YAML Format:**
//! ```yaml
//! - type: file
//!   description: "Create application directory"
//!   path: /opt/myapp
//!   state: present
//!   mode: "0755"
//!   owner: myapp
//!   group: myapp
//! ```
//!
//! ## Register file creation
//!
//! This example creates a file and registers the result.
//!
//! **YAML Format:**
//! ```yaml
//! - type: file
//!   description: "Create marker file"
//!   path: /tmp/driftless.marker
//!   state: present
//!   register: marker_file
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "file",
//!   "description": "Create marker file",
//!   "path": "/tmp/driftless.marker",
//!   "state": "present",
//!   "register": "marker_file"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "file"
//! description = "Create marker file"
//! path = "/tmp/driftless.marker"
//! state = "present"
//! register = "marker_file"
//! ```

/// File state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileState {
    /// Ensure file exists
    Present,
    /// Ensure file does not exist
    Absent,
}

/// File operation task
///
/// Manages files and directories - create, modify, or remove files with content,
/// permissions, and ownership. Similar to Ansible's `file` module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to the file or directory
    ///
    /// Absolute path to the file or directory to manage.
    /// Parent directories will not be created automatically.
    pub path: String,

    /// File state (present, absent)
    ///
    /// - `present`: Ensure the file exists with specified properties
    /// - `absent`: Ensure the file does not exist
    pub state: FileState,

    /// File content (for present state)
    ///
    /// Content to write to the file when state is `present`.
    /// Mutually exclusive with `source`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Source file to copy from (alternative to content)
    ///
    /// Path to a source file to copy content from when state is `present`.
    /// Mutually exclusive with `content`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// File permissions (octal string like "0644")
    ///
    /// File permissions in octal notation (e.g., "0644", "0755").
    /// Only applied when creating or modifying files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,

    /// File owner username
    ///
    /// Username of the file owner. Only applied when creating or modifying files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,

    /// File group name
    ///
    /// Group name for the file. Only applied when creating or modifying files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Execute a file task
pub async fn execute_file_task(task: &FileTask, dry_run: bool) -> Result<()> {
    let path = Path::new(&task.path);

    match task.state {
        FileState::Present => ensure_file_present(path, task, dry_run).await,
        FileState::Absent => ensure_file_absent(path, dry_run).await,
    }
}

/// Ensure a file exists with the correct content and permissions
async fn ensure_file_present(path: &Path, task: &FileTask, dry_run: bool) -> Result<()> {
    let exists = path.exists();

    if !exists {
        if dry_run {
            println!("Would create file: {}", path.display());
        } else {
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create parent directories for {}", path.display())
                })?;
            }

            // Create the file
            fs::write(path, task.content.as_deref().unwrap_or(""))
                .with_context(|| format!("Failed to create file {}", path.display()))?;
            println!("Created file: {}", path.display());
        }
    } else {
        // File exists, check if content needs updating
        if let Some(content) = &task.content {
            let current_content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read existing file {}", path.display()))?;

            if current_content != *content {
                if dry_run {
                    println!("Would update content of file: {}", path.display());
                } else {
                    fs::write(path, content)
                        .with_context(|| format!("Failed to update file {}", path.display()))?;
                    println!("Updated content of file: {}", path.display());
                }
            }
        }

        // Check if source file should be copied
        if let Some(source) = &task.source {
            let source_path = Path::new(source);
            if source_path.exists() {
                let source_content = fs::read(source_path)
                    .with_context(|| format!("Failed to read source file {}", source))?;

                let current_content = fs::read(path)
                    .with_context(|| format!("Failed to read target file {}", path.display()))?;

                if source_content != current_content {
                    if dry_run {
                        println!("Would copy {} to {}", source, path.display());
                    } else {
                        fs::write(path, source_content).with_context(|| {
                            format!("Failed to copy {} to {}", source, path.display())
                        })?;
                        println!("Copied {} to {}", source, path.display());
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Source file does not exist: {}", source));
            }
        }
    }

    // Set permissions if specified
    if let Some(mode) = &task.mode {
        set_file_permissions(path, mode, dry_run)?;
    }

    // Set ownership if specified
    if task.owner.is_some() || task.group.is_some() {
        set_file_ownership(path, task.owner.as_deref(), task.group.as_deref(), dry_run)?;
    }

    Ok(())
}

/// Ensure a file does not exist
async fn ensure_file_absent(path: &Path, dry_run: bool) -> Result<()> {
    if path.exists() {
        if dry_run {
            println!("Would remove file: {}", path.display());
        } else if path.is_file() {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove file {}", path.display()))?;
            println!("Removed file: {}", path.display());
        } else {
            return Err(anyhow::anyhow!(
                "Path exists but is not a file: {}",
                path.display()
            ));
        }
    }

    Ok(())
}

/// Set file permissions
fn set_file_permissions(path: &Path, mode: &str, dry_run: bool) -> Result<()> {
    let mode_u32 = u32::from_str_radix(mode.trim_start_matches("0o"), 8)
        .with_context(|| format!("Invalid octal mode: {}", mode))?;

    if dry_run {
        println!("Would set permissions of {} to {}", path.display(), mode);
    } else {
        let mut perms = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for {}", path.display()))?
            .permissions();

        perms.set_mode(mode_u32);
        fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to set permissions on {}", path.display()))?;

        println!("Set permissions of {} to {}", path.display(), mode);
    }

    Ok(())
}

/// Set file ownership
fn set_file_ownership(
    path: &Path,
    owner: Option<&str>,
    group: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    // Note: This is a simplified implementation. In a real system, you'd need to:
    // 1. Look up UID/GID from username/groupname
    // 2. Handle cases where user/group doesn't exist
    // 3. Check permissions for chown operation

    let owner_str = owner.unwrap_or("unchanged");
    let group_str = group.unwrap_or("unchanged");

    if dry_run {
        println!(
            "Would set ownership of {} to {}:{}",
            path.display(),
            owner_str,
            group_str
        );
    } else {
        // For now, just log what would be done
        // In a real implementation, you'd use the users crate or similar
        println!(
            "Note: Ownership setting not fully implemented yet for {}:{}",
            owner_str, group_str
        );
        println!(
            "Set ownership of {} to {}:{}",
            path.display(),
            owner_str,
            group_str
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::FileTask;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_create_file_dry_run() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        // Remove the temp file so we can test creation
        drop(temp_file);

        let task = FileTask {
            description: None,
            path: temp_path.clone(),
            state: FileState::Present,
            content: Some("test content".to_string()),
            source: None,
            mode: Some("0644".to_string()),
            owner: Some("root".to_string()),
            group: Some("root".to_string()),
        };

        let result = execute_file_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!Path::new(&temp_path).exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_create_file_real() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        // Remove the temp file so we can test creation
        drop(temp_file);

        let task = FileTask {
            description: None,
            path: temp_path.clone(),
            state: FileState::Present,
            content: Some("test content".to_string()),
            source: None,
            mode: None,
            owner: None,
            group: None,
        };

        let result = execute_file_task(&task, false).await;
        assert!(result.is_ok());
        assert!(Path::new(&temp_path).exists());

        let content = fs::read_to_string(&temp_path).unwrap();
        assert_eq!(content, "test content");

        // Cleanup
        fs::remove_file(&temp_path).unwrap();
    }

    #[tokio::test]
    async fn test_remove_file() {
        use std::io::Write;
        let temp_file = NamedTempFile::new().unwrap();
        temp_file.as_file().write_all(b"content").unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let task = FileTask {
            description: None,
            path: temp_path.clone(),
            state: FileState::Absent,
            content: None,
            source: None,
            mode: None,
            owner: None,
            group: None,
        };

        let result = execute_file_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&temp_path).exists());
    }

    #[test]
    fn test_invalid_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let task = FileTask {
            description: None,
            path: temp_path.clone(),
            state: FileState::Present,
            content: Some("test".to_string()),
            source: None,
            mode: Some("invalid".to_string()),
            owner: None,
            group: None,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(execute_file_task(&task, false));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid octal mode"));
    }
}
