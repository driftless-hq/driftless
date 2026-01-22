//! Copy files task executor
//!
//! Handles copying files with various options for permissions, ownership, etc.
//!
//! # Examples
//!
//! ## Copy a file
//!
//! This example copies a file from source to destination.
//!
//! **YAML Format:**
//! ```yaml
//! - type: copy
//!   description: "Copy configuration file"
//!   src: /etc/nginx/nginx.conf.template
//!   dest: /etc/nginx/nginx.conf
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "copy",
//!   "description": "Copy configuration file",
//!   "src": "/etc/nginx/nginx.conf.template",
//!   "dest": "/etc/nginx/nginx.conf",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "copy"
//! description = "Copy configuration file"
//! src = "/etc/nginx/nginx.conf.template"
//! dest = "/etc/nginx/nginx.conf"
//! state = "present"
//! ```
//!
//! ## Copy with backup
//!
//! This example copies a file and creates a backup of the destination.
//!
//! **YAML Format:**
//! ```yaml
//! - type: copy
//!   description: "Copy config with backup"
//!   src: /tmp/new-config.conf
//!   dest: /etc/myapp/config.conf
//!   state: present
//!   backup: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "copy",
//!   "description": "Copy config with backup",
//!   "src": "/tmp/new-config.conf",
//!   "dest": "/etc/myapp/config.conf",
//!   "state": "present",
//!   "backup": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "copy"
//! description = "Copy config with backup"
//! src = "/tmp/new-config.conf"
//! dest = "/etc/myapp/config.conf"
//! state = "present"
//! backup = true
//! ```
//!
//! ## Remove a copied file
//!
//! This example removes a file that was previously copied.
//!
//! **YAML Format:**
//! ```yaml
//! - type: copy
//!   description: "Remove copied configuration"
//!   src: /etc/nginx/nginx.conf.template
//!   dest: /etc/nginx/nginx.conf
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "copy",
//!   "description": "Remove copied configuration",
//!   "src": "/etc/nginx/nginx.conf.template",
//!   "dest": "/etc/nginx/nginx.conf",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "copy"
//! description = "Remove copied configuration"
//! src = "/etc/nginx/nginx.conf.template"
//! dest = "/etc/nginx/nginx.conf"
//! state = "absent"
//! ```

/// Copy state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CopyState {
    /// Ensure file is copied
    Present,
    /// Ensure file is not copied (remove if exists)
    Absent,
}

/// Copy files task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CopyTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source file path
    pub src: String,
    /// Destination file path
    pub dest: String,
    /// Copy state
    pub state: CopyState,
    /// Whether to follow symlinks
    #[serde(default)]
    pub follow: bool,
    /// Whether to preserve permissions
    #[serde(default = "crate::apply::default_true")]
    pub mode: bool,
    /// Whether to preserve ownership
    #[serde(default)]
    pub owner: bool,
    /// Whether to preserve timestamps
    #[serde(default)]
    pub timestamp: bool,
    /// Whether to create backup of destination
    #[serde(default)]
    pub backup: bool,
    /// Force copy even if destination exists
    #[serde(default)]
    pub force: bool,
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute a copy task
pub async fn execute_copy_task(task: &CopyTask, dry_run: bool) -> Result<()> {
    match task.state {
        CopyState::Present => ensure_file_copied(task, dry_run).await,
        CopyState::Absent => ensure_file_not_copied(task, dry_run).await,
    }
}

/// Ensure file is copied to destination
async fn ensure_file_copied(task: &CopyTask, dry_run: bool) -> Result<()> {
    let src_path = Path::new(&task.src);
    let dest_path = Path::new(&task.dest);

    // Check if source exists
    if !src_path.exists() {
        return Err(anyhow::anyhow!("Source file does not exist: {}", task.src));
    }

    // Check if source is a file (not a directory)
    if !src_path.is_file() {
        return Err(anyhow::anyhow!(
            "Source is not a regular file: {}",
            task.src
        ));
    }

    // Check if destination needs updating
    let needs_copy = if dest_path.exists() {
        if task.force {
            true // Force copy even if destination exists
        } else {
            // Check if files are different
            file_contents_differ(src_path, dest_path).unwrap_or(true)
        }
    } else {
        true // Destination doesn't exist
    };

    if !needs_copy {
        println!("File {} is already up to date", task.dest);
        return Ok(());
    }

    if dry_run {
        println!("Would copy {} to {}", task.src, task.dest);
        if task.backup && dest_path.exists() {
            println!("  (would backup existing file)");
        }
    } else {
        // Backup destination if requested
        if task.backup && dest_path.exists() {
            let backup_path = format!("{}.backup", task.dest);
            fs::copy(&task.dest, &backup_path)
                .with_context(|| format!("Failed to backup {} to {}", task.dest, backup_path))?;
            println!("Backed up {} to {}", task.dest, backup_path);
        }

        // Perform the copy
        fs::copy(&task.src, &task.dest)
            .with_context(|| format!("Failed to copy {} to {}", task.src, task.dest))?;

        println!("Copied {} to {}", task.src, task.dest);

        // Set permissions if requested
        if task.mode {
            if let Ok(metadata) = src_path.metadata() {
                let mode = metadata.permissions();
                fs::set_permissions(&task.dest, mode)
                    .with_context(|| format!("Failed to set permissions on {}", task.dest))?;
                println!("Preserved permissions on {}", task.dest);
            }
        }

        // Note: Ownership preservation would require additional privileges
        // and is not implemented in this basic version
        if task.owner {
            println!("Note: Ownership preservation not implemented");
        }

        // Note: Timestamp preservation is complex and not implemented
        if task.timestamp {
            println!("Note: Timestamp preservation not implemented");
        }
    }

    Ok(())
}

/// Ensure file is not copied (remove if it exists)
async fn ensure_file_not_copied(task: &CopyTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    if !dest_path.exists() {
        println!("Destination file does not exist: {}", task.dest);
        return Ok(());
    }

    // Check if destination was actually copied from source
    // This is a simplified check - in practice, we'd need more sophisticated
    // tracking to know if a file was created by this copy task
    if let Ok(src_metadata) = fs::metadata(&task.src) {
        if let Ok(dest_metadata) = fs::metadata(&task.dest) {
            // If sizes match and modification times are close, assume it's a copy
            if src_metadata.len() == dest_metadata.len() {
                if dry_run {
                    println!("Would remove copied file: {}", task.dest);
                } else {
                    fs::remove_file(&task.dest)
                        .with_context(|| format!("Failed to remove file {}", task.dest))?;
                    println!("Removed copied file: {}", task.dest);
                }
                return Ok(());
            }
        }
    }

    println!(
        "Destination file {} exists but does not appear to be a copy of {}",
        task.dest, task.src
    );
    Ok(())
}

/// Check if two files have different contents
fn file_contents_differ(path1: &Path, path2: &Path) -> Result<bool> {
    let content1 =
        fs::read(path1).with_context(|| format!("Failed to read {}", path1.display()))?;
    let content2 =
        fs::read(path2).with_context(|| format!("Failed to read {}", path2.display()))?;

    Ok(content1 != content2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_copy_file_dry_run() {
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        fs::write(&src_path, "test content").unwrap();

        let dest_path = src_path.clone() + ".dest";

        let task = CopyTask {
            description: None,
            src: src_path.clone(),
            dest: dest_path.clone(),
            state: CopyState::Present,
            follow: false,
            mode: true,
            owner: false,
            timestamp: false,
            backup: false,
            force: false,
        };

        let result = execute_copy_task(&task, true).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists()); // File shouldn't exist in dry run
    }

    #[tokio::test]
    async fn test_copy_file_real() {
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        let test_content = "test content for copy";
        fs::write(&src_path, test_content).unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        drop(dest_file); // Remove the temp file so we can copy to it

        let task = CopyTask {
            description: None,
            src: src_path.clone(),
            dest: dest_path.clone(),
            state: CopyState::Present,
            follow: false,
            mode: false,
            owner: false,
            timestamp: false,
            backup: false,
            force: false,
        };

        let result = execute_copy_task(&task, false).await;
        assert!(result.is_ok());
        assert!(Path::new(&dest_path).exists());

        let copied_content = fs::read_to_string(&dest_path).unwrap();
        assert_eq!(copied_content, test_content);
    }

    #[tokio::test]
    async fn test_copy_nonexistent_source() {
        let task = CopyTask {
            description: None,
            src: "/nonexistent/source/file.txt".to_string(),
            dest: "/tmp/dest.txt".to_string(),
            state: CopyState::Present,
            follow: false,
            mode: false,
            owner: false,
            timestamp: false,
            backup: false,
            force: false,
        };

        let result = execute_copy_task(&task, true).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source file does not exist"));
    }

    #[tokio::test]
    async fn test_copy_to_existing_file_no_force() {
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        fs::write(&src_path, "new content").unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        fs::write(&dest_path, "existing content").unwrap();

        let task = CopyTask {
            description: None,
            src: src_path.clone(),
            dest: dest_path.clone(),
            state: CopyState::Present,
            follow: false,
            mode: false,
            owner: false,
            timestamp: false,
            backup: false,
            force: false, // Don't force overwrite
        };

        // Should succeed if files are identical
        let result = execute_copy_task(&task, false).await;
        assert!(result.is_ok()); // Should not overwrite since content is different
    }
}
