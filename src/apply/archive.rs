//! Archive files task executor
//!
//! Handles creating archives from files and directories.
//!
//! # Examples
//!
//! ## Create a tar archive
//!
//! This example creates a tar archive from a directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: archive
//!   description: "Create backup archive"
//!   path: /tmp/backup.tar
//!   state: present
//!   format: tar
//!   sources:
//!     - /home/user/documents
//!     - /home/user/pictures
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "archive",
//!   "description": "Create backup archive",
//!   "path": "/tmp/backup.tar",
//!   "state": "present",
//!   "format": "tar",
//!   "sources": ["/home/user/documents", "/home/user/pictures"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "archive"
//! description = "Create backup archive"
//! path = "/tmp/backup.tar"
//! state = "present"
//! format = "tar"
//! sources = ["/home/user/documents", "/home/user/pictures"]
//! ```
//!
//! ## Create a compressed tar archive
//!
//! This example creates a gzip-compressed tar archive.
//!
//! **YAML Format:**
//! ```yaml
//! - type: archive
//!   description: "Create compressed backup"
//!   path: /tmp/backup.tar.gz
//!   state: present
//!   format: tgz
//!   sources:
//!     - /var/log
//!   compression: 9
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "archive",
//!   "description": "Create compressed backup",
//!   "path": "/tmp/backup.tar.gz",
//!   "state": "present",
//!   "format": "tgz",
//!   "sources": ["/var/log"],
//!   "compression": 9
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "archive"
//! description = "Create compressed backup"
//! path = "/tmp/backup.tar.gz"
//! state = "present"
//! format = "tgz"
//! sources = ["/var/log"]
//! compression = 9
//! ```
//!
//! ## Create a zip archive
//!
//! This example creates a zip archive.
//!
//! **YAML Format:**
//! ```yaml
//! - type: archive
//!   description: "Create zip archive"
//!   path: /tmp/data.zip
//!   state: present
//!   format: zip
//!   sources:
//!     - /home/user/data
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "archive",
//!   "description": "Create zip archive",
//!   "path": "/tmp/data.zip",
//!   "state": "present",
//!   "format": "zip",
//!   "sources": ["/home/user/data"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "archive"
//! description = "Create zip archive"
//! path = "/tmp/data.zip"
//! state = "present"
//! format = "zip"
//! sources = ["/home/user/data"]
//! ```
//!
//! ## Remove an archive
//!
//! This example removes an archive file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: archive
//!   description: "Remove old backup"
//!   path: /tmp/old-backup.tar.gz
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "archive",
//!   "description": "Remove old backup",
//!   "path": "/tmp/old-backup.tar.gz",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "archive"
//! description = "Remove old backup"
//! path = "/tmp/old-backup.tar.gz"
//! state = "absent"
//! ```

/// Archive files task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Archive file path
    pub path: String,
    /// Archive state
    pub state: ArchiveState,
    /// Archive format
    #[serde(default)]
    pub format: ArchiveFormat,
    /// Files/directories to archive
    pub sources: Vec<String>,
    /// Destination directory (for extraction)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
    /// Compression level (1-9)
    #[serde(default = "default_compression_level")]
    pub compression: u32,
    /// Extra options for archiving
    #[serde(default)]
    pub extra_opts: Vec<String>,
}

/// Archive state enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveState {
    /// Ensure archive exists
    Present,
    /// Ensure archive does not exist
    Absent,
}

/// Archive format enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    /// Tar archive
    Tar,
    /// Gzip compressed tar
    #[default]
    Tgz,
    /// Bzip2 compressed tar
    Tbz2,
    /// XZ compressed tar
    Txz,
    /// Zip archive
    Zip,
    /// 7z archive
    SevenZ,
}

use serde::{Deserialize, Serialize};

use crate::apply::default_compression_level;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Execute an archive task
pub async fn execute_archive_task(task: &ArchiveTask, dry_run: bool) -> Result<()> {
    match task.state {
        ArchiveState::Present => ensure_archive_created(task, dry_run).await,
        ArchiveState::Absent => ensure_archive_removed(task, dry_run).await,
    }
}

/// Ensure archive is created
async fn ensure_archive_created(task: &ArchiveTask, dry_run: bool) -> Result<()> {
    let archive_path = Path::new(&task.path);

    // Check if archive needs updating
    let needs_creation = if archive_path.exists() {
        // For simplicity, we'll recreate the archive
        // A full implementation would check if source files changed
        true
    } else {
        true
    };

    if !needs_creation {
        println!("Archive already exists: {}", task.path);
        return Ok(());
    }

    // Verify source files exist
    for source in &task.sources {
        if !Path::new(source).exists() {
            return Err(anyhow::anyhow!("Source file does not exist: {}", source));
        }
    }

    if dry_run {
        println!(
            "Would create archive {} with sources: {:?}",
            task.path, task.sources
        );
    } else {
        // Ensure destination directory exists
        if let Some(parent) = archive_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directories for {}", task.path)
            })?;
        }

        // Create the archive
        create_archive(task).await?;

        println!(
            "Created archive {} with {} sources",
            task.path,
            task.sources.len()
        );
    }

    Ok(())
}

/// Ensure archive is removed
async fn ensure_archive_removed(task: &ArchiveTask, dry_run: bool) -> Result<()> {
    let archive_path = Path::new(&task.path);

    if !archive_path.exists() {
        println!("Archive does not exist: {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would remove archive: {}", task.path);
    } else {
        fs::remove_file(archive_path)
            .with_context(|| format!("Failed to remove archive {}", task.path))?;
        println!("Removed archive: {}", task.path);
    }

    Ok(())
}

/// Create archive using appropriate tool
async fn create_archive(task: &ArchiveTask) -> Result<()> {
    match task.format {
        ArchiveFormat::Tar => create_tar_archive(task).await,
        ArchiveFormat::Tgz => create_tar_gz_archive(task).await,
        ArchiveFormat::Tbz2 => create_tar_bz2_archive(task).await,
        ArchiveFormat::Txz => create_tar_xz_archive(task).await,
        ArchiveFormat::Zip => create_zip_archive(task).await,
        ArchiveFormat::SevenZ => create_7z_archive(task).await,
    }
}

/// Create uncompressed tar archive
async fn create_tar_archive(task: &ArchiveTask) -> Result<()> {
    let mut args = vec!["-cf", &task.path];
    args.extend(task.sources.iter().map(|s| s.as_str()));
    args.extend(task.extra_opts.iter().map(|s| s.as_str()));

    run_command("tar", &args).await
}

/// Create gzip-compressed tar archive
async fn create_tar_gz_archive(task: &ArchiveTask) -> Result<()> {
    let mut args = vec!["-czf", &task.path];
    args.extend(task.sources.iter().map(|s| s.as_str()));
    args.extend(task.extra_opts.iter().map(|s| s.as_str()));

    run_command("tar", &args).await
}

/// Create bzip2-compressed tar archive
async fn create_tar_bz2_archive(task: &ArchiveTask) -> Result<()> {
    let mut args = vec!["-cjf", &task.path];
    args.extend(task.sources.iter().map(|s| s.as_str()));
    args.extend(task.extra_opts.iter().map(|s| s.as_str()));

    run_command("tar", &args).await
}

/// Create xz-compressed tar archive
async fn create_tar_xz_archive(task: &ArchiveTask) -> Result<()> {
    let mut args = vec!["-cJf", &task.path];
    args.extend(task.sources.iter().map(|s| s.as_str()));
    args.extend(task.extra_opts.iter().map(|s| s.as_str()));

    run_command("tar", &args).await
}

/// Create zip archive
async fn create_zip_archive(task: &ArchiveTask) -> Result<()> {
    let mut args = vec!["-r", "-q", &task.path];
    args.extend(task.sources.iter().map(|s| s.as_str()));
    args.extend(task.extra_opts.iter().map(|s| s.as_str()));

    run_command("zip", &args).await
}

/// Create 7z archive
async fn create_7z_archive(task: &ArchiveTask) -> Result<()> {
    let mut args = vec!["a", &task.path];
    args.extend(task.sources.iter().map(|s| s.as_str()));
    args.extend(task.extra_opts.iter().map(|s| s.as_str()));

    run_command("7z", &args).await
}

/// Run external command for archive creation
async fn run_command(command: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run command: {} {:?}", command, args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Command failed: {} {:?}\nstderr: {}",
            command,
            args,
            stderr
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_archive_creation_dry_run() {
        let source_file = NamedTempFile::new().unwrap();
        let source_path = source_file.path().to_str().unwrap().to_string();
        fs::write(&source_path, "test content").unwrap();

        let archive_file = NamedTempFile::new().unwrap();
        let archive_path = archive_file.path().to_str().unwrap().to_string() + ".tar.gz";

        let task = ArchiveTask {
            description: None,
            path: archive_path.clone(),
            state: ArchiveState::Present,
            format: ArchiveFormat::Tgz,
            sources: vec![source_path],
            dest: None,
            compression: 6,
            extra_opts: vec![],
        };

        let result = execute_archive_task(&task, true).await;
        assert!(result.is_ok());
        // Archive shouldn't exist in dry run
        assert!(!Path::new(&archive_path).exists());
    }

    #[tokio::test]
    async fn test_archive_remove() {
        let archive_file = NamedTempFile::new().unwrap();
        let archive_path = archive_file.path().to_str().unwrap().to_string();
        fs::write(&archive_path, "dummy archive").unwrap();

        let task = ArchiveTask {
            description: None,
            path: archive_path.clone(),
            state: ArchiveState::Absent,
            format: ArchiveFormat::Tar,
            sources: vec![],
            dest: None,
            compression: 6,
            extra_opts: vec![],
        };

        let result = execute_archive_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&archive_path).exists());
    }

    #[tokio::test]
    async fn test_archive_nonexistent_source() {
        let archive_path = "/tmp/test_archive.tar.gz".to_string();

        let task = ArchiveTask {
            description: None,
            path: archive_path,
            state: ArchiveState::Present,
            format: ArchiveFormat::Tgz,
            sources: vec!["/nonexistent/source/file.txt".to_string()],
            dest: None,
            compression: 6,
            extra_opts: vec![],
        };

        let result = execute_archive_task(&task, true).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Source file does not exist"));
    }

    #[tokio::test]
    async fn test_archive_empty_sources() {
        use crate::apply::{ApplyConfig, Task};

        let archive_path = "/tmp/empty_archive.tar.gz".to_string();

        let config = ApplyConfig {
            vars: std::collections::HashMap::new(),
            tasks: vec![Task::Archive(ArchiveTask {
                description: None,
                path: archive_path,
                state: ArchiveState::Present,
                format: ArchiveFormat::Tgz,
                sources: vec![], // Empty sources list
                dest: None,
                compression: 6,
                extra_opts: vec![],
            })],
        };

        let executor = crate::apply::executor::TaskExecutor::new(true);
        let result = executor.validate(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("sources cannot be empty"));
    }

    #[tokio::test]
    async fn test_archive_different_formats() {
        let source_file = NamedTempFile::new().unwrap();
        let source_path = source_file.path().to_str().unwrap().to_string();
        fs::write(&source_path, "test content").unwrap();

        let formats = vec![
            (ArchiveFormat::Tar, ".tar"),
            (ArchiveFormat::Tgz, ".tar.gz"),
            (ArchiveFormat::Zip, ".zip"),
        ];

        for (format, extension) in formats {
            let archive_path = format!("/tmp/test_archive{}", extension);

            let task = ArchiveTask {
                description: None,
                path: archive_path.clone(),
                state: ArchiveState::Present,
                format: format.clone(),
                sources: vec![source_path.clone()],
                dest: None,
                compression: 6,
                extra_opts: vec![],
            };

            let result = execute_archive_task(&task, true).await;
            assert!(result.is_ok(), "Failed for format {:?}", format);
        }
    }
}
