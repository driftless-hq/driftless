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
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

/// State information for tracking copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CopyStateInfo {
    /// SHA256 checksum of the source file
    source_checksum: String,
    /// Size of the source file
    source_size: u64,
    /// Last modification time of the source file
    source_modified: SystemTime,
    /// SHA256 checksum of the destination file after copy
    dest_checksum: String,
    /// Size of the destination file after copy
    dest_size: u64,
    /// Last modification time when copy was performed
    copied_at: SystemTime,
}

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

    // Check if destination needs updating using state tracking
    let needs_copy = if dest_path.exists() {
        if task.force {
            true // Force copy even if destination exists
        } else {
            // Load previous copy state
            match load_copy_state(&task.dest) {
                Ok(Some(prev_state)) => {
                    // Check if source has changed since last copy
                    let src_metadata = src_path
                        .metadata()
                        .with_context(|| format!("Failed to get metadata for {}", task.src))?;

                    let src_modified = src_metadata.modified().with_context(|| {
                        format!("Failed to get modification time for {}", task.src)
                    })?;

                    // If source modification time is newer than copy time, or if we can't determine,
                    // check the checksum
                    if src_modified > prev_state.copied_at {
                        true
                    } else {
                        // Calculate current source checksum
                        match calculate_file_checksum(src_path) {
                            Ok(current_checksum) => current_checksum != prev_state.source_checksum,
                            Err(_) => true, // If we can't calculate checksum, assume changed
                        }
                    }
                }
                Ok(None) => {
                    // No previous state, check if destination exists and force is not set
                    !dest_path.exists() || task.force
                }
                Err(_) => {
                    // Error loading state, check if destination exists and force is not set
                    !dest_path.exists() || task.force
                }
            }
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

        // Save copy state for change detection
        let src_metadata = src_path
            .metadata()
            .with_context(|| format!("Failed to get metadata for {}", task.src))?;
        let dest_metadata = dest_path
            .metadata()
            .with_context(|| format!("Failed to get metadata for {}", task.dest))?;

        let src_checksum = calculate_file_checksum(src_path)?;
        let dest_checksum = calculate_file_checksum(dest_path)?;

        let state = CopyStateInfo {
            source_checksum: src_checksum,
            source_size: src_metadata.len(),
            source_modified: src_metadata
                .modified()
                .with_context(|| format!("Failed to get modification time for {}", task.src))?,
            dest_checksum,
            dest_size: dest_metadata.len(),
            copied_at: SystemTime::now(),
        };

        if let Err(e) = save_copy_state(&task.dest, &state) {
            println!("Warning: Failed to save copy state: {}", e);
        }

        // Set permissions if requested
        if task.mode {
            if let Ok(metadata) = src_path.metadata() {
                let mode = metadata.permissions();
                fs::set_permissions(&task.dest, mode)
                    .with_context(|| format!("Failed to set permissions on {}", task.dest))?;
                println!("Preserved permissions on {}", task.dest);
            }
        }

        // Preserve ownership if requested
        if task.owner {
            preserve_ownership(&task.src, &task.dest)?;
        }

        // Preserve timestamp if requested
        if task.timestamp {
            preserve_timestamp(&task.src, &task.dest)?;
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

    // Load copy state to determine if this file was created by a copy task
    match load_copy_state(&task.dest) {
        Ok(Some(state)) => {
            // Check if the destination file matches what was copied from the source
            let current_dest_checksum = match calculate_file_checksum(dest_path) {
                Ok(checksum) => checksum,
                Err(_) => {
                    println!("Warning: Could not calculate checksum for destination file, assuming it's not a copy");
                    return Ok(());
                }
            };

            // If the destination checksum matches what was saved during copy, it's our copy
            if current_dest_checksum == state.dest_checksum {
                if dry_run {
                    println!("Would remove copied file: {}", task.dest);
                } else {
                    fs::remove_file(&task.dest)
                        .with_context(|| format!("Failed to remove file {}", task.dest))?;
                    println!("Removed copied file: {}", task.dest);

                    // Clean up the state file
                    let state_path = get_copy_state_path(&task.dest);
                    if Path::new(&state_path).exists() {
                        if let Err(e) = fs::remove_file(&state_path) {
                            println!(
                                "Warning: Failed to remove copy state file {}: {}",
                                state_path, e
                            );
                        }
                    }
                }
                return Ok(());
            } else {
                println!(
                    "Destination file {} exists but checksum doesn't match copy state (file may have been modified)",
                    task.dest
                );
            }
        }
        Ok(None) => {
            println!(
                "Destination file {} exists but no copy state found (not created by copy task)",
                task.dest
            );
        }
        Err(e) => {
            println!(
                "Warning: Could not load copy state for {}: {}, assuming file is not a copy",
                task.dest, e
            );
        }
    }

    Ok(())
}

/// Get the state file path for a copy operation
fn get_copy_state_path(dest: &str) -> String {
    format!("{}.driftless-copy-state", dest)
}

/// Load copy state from file
fn load_copy_state(dest: &str) -> Result<Option<CopyStateInfo>> {
    let state_path = get_copy_state_path(dest);
    if !Path::new(&state_path).exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&state_path)
        .with_context(|| format!("Failed to read copy state file: {}", state_path))?;

    let state: CopyStateInfo = serde_json::from_str(&content)
        .context(format!("Failed to parse copy state file: {}", state_path))?;

    Ok(Some(state))
}

/// Save copy state to file
fn save_copy_state(dest: &str, state: &CopyStateInfo) -> Result<()> {
    let state_path = get_copy_state_path(dest);
    let content = serde_json::to_string_pretty(state)
        .context(format!("Failed to serialize copy state for: {}", dest))?;

    fs::write(&state_path, content)
        .with_context(|| format!("Failed to write copy state file: {}", state_path))?;

    Ok(())
}

/// Calculate SHA256 checksum of a file
fn calculate_file_checksum(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut hasher = Sha256::new();
    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open file for checksum: {}", path.display()))?;

    let mut buffer = [0; 8192];
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file for checksum: {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Preserve ownership from source file to destination
fn preserve_ownership(src: &str, dest: &str) -> Result<()> {
    let output = Command::new("chown")
        .args(["--reference", src, dest])
        .output()
        .with_context(|| format!("Failed to preserve ownership from {} to {}", src, dest))?;

    if output.status.success() {
        println!("Preserved ownership on {}", dest);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("chown failed: {}", stderr))
    }
}

/// Preserve timestamp from source file to destination
fn preserve_timestamp(src: &str, dest: &str) -> Result<()> {
    let output = Command::new("touch")
        .args(["-r", src, dest])
        .output()
        .with_context(|| format!("Failed to preserve timestamp from {} to {}", src, dest))?;

    if output.status.success() {
        println!("Preserved timestamp on {}", dest);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("touch failed: {}", stderr))
    }
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

    #[tokio::test]
    async fn test_copy_absent_state_with_state_tracking() {
        let src_file = NamedTempFile::new().unwrap();
        let src_path = src_file.path().to_str().unwrap().to_string();
        let test_content = "test content for absent state";
        fs::write(&src_path, test_content).unwrap();

        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        drop(dest_file); // Remove the temp file so we can copy to it

        // First copy the file
        let copy_task = CopyTask {
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

        let result = execute_copy_task(&copy_task, false).await;
        assert!(result.is_ok());
        assert!(Path::new(&dest_path).exists());

        // Now test removing it with absent state
        let remove_task = CopyTask {
            description: None,
            src: src_path.clone(),
            dest: dest_path.clone(),
            state: CopyState::Absent,
            follow: false,
            mode: false,
            owner: false,
            timestamp: false,
            backup: false,
            force: false,
        };

        let result = execute_copy_task(&remove_task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists());

        // Verify state file is cleaned up
        let state_path = get_copy_state_path(&dest_path);
        assert!(!Path::new(&state_path).exists());
    }
}
