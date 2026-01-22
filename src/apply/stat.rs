//! File/directory statistics task executor
//!
//! Handles gathering and displaying file/directory statistics.
//!
//! # Examples
//!
//! ## Get file statistics
//!
//! This example displays statistics for a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: stat
//!   description: "Get file statistics"
//!   path: /etc/passwd
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "stat",
//!   "description": "Get file statistics",
//!   "path": "/etc/passwd"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "stat"
//! description = "Get file statistics"
//! path = "/etc/passwd"
//! ```
//!
//! ## Get file checksum
//!
//! This example calculates and displays the checksum of a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: stat
//!   description: "Get file checksum"
//!   path: /etc/hosts
//!   checksum: true
//!   checksum_algorithm: sha256
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "stat",
//!   "description": "Get file checksum",
//!   "path": "/etc/hosts",
//!   "checksum": true,
//!   "checksum_algorithm": "sha256"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "stat"
//! description = "Get file checksum"
//! path = "/etc/hosts"
//! checksum = true
//! checksum_algorithm = "sha256"
//! ```
//!
//! ## Follow symlinks
//!
//! This example follows symlinks when getting statistics.
//!
//! **YAML Format:**
//! ```yaml
//! - type: stat
//!   description: "Follow symlink for statistics"
//!   path: /var/log/syslog
//!   follow: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "stat",
//!   "description": "Follow symlink for statistics",
//!   "path": "/var/log/syslog",
//!   "follow": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "stat"
//! description = "Follow symlink for statistics"
//! path = "/var/log/syslog"
//! follow = true
//! ```
//!
//! ## Register file status
//!
//! This example checks if a file exists and registers its status for use in subsequent tasks.
//!
//! **YAML Format:**
//! ```yaml
//! - type: stat
//!   description: "Check if nginx config exists"
//!   path: /etc/nginx/nginx.conf
//!   register: nginx_conf
//!
//! - type: debug
//!   msg: "Nginx config exists: {{ nginx_conf.exists }}"
//!   when: "{{ nginx_conf.exists }}"
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "stat",
//!     "description": "Check if nginx config exists",
//!     "path": "/etc/nginx/nginx.conf",
//!     "register": "nginx_conf"
//!   },
//!   {
//!     "type": "debug",
//!     "msg": "Nginx config exists: {{ nginx_conf.exists }}",
//!     "when": "{{ nginx_conf.exists }}"
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "stat"
//! description = "Check if nginx config exists"
//! path = "/etc/nginx/nginx.conf"
//! register = "nginx_conf"
//!
//! [[tasks]]
//! type = "debug"
//! msg = "Nginx config exists: {{ nginx_conf.exists }}"
//! when = "{{ nginx_conf.exists }}"
//! ```
//!
//! ## Get directory statistics
//!
//! This example displays statistics for a directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: stat
//!   description: "Get directory statistics"
//!   path: /home/user
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "stat",
//!   "description": "Get directory statistics",
//!   "path": "/home/user"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "stat"
//! description = "Get directory statistics"
//! path = "/home/user"
//! ```

/// Checksum algorithm enumeration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecksumAlgorithm {
    /// MD5 hash algorithm
    Md5,
    /// SHA-1 hash algorithm
    Sha1,
    /// SHA-256 hash algorithm (default)
    #[default]
    Sha256,
    /// SHA-512 hash algorithm
    Sha512,
}

/// File/directory statistics task
///
/// # Registered Outputs
/// - `exists` (bool): Whether the file or directory exists
/// - `is_file` (bool): Whether the path is a file
/// - `is_dir` (bool): Whether the path is a directory
/// - `size` (u64): The size of the file in bytes
/// - `mode` (u32): The file mode (permissions)
/// - `uid` (u32): The user ID of the owner
/// - `gid` (u32): The group ID of the owner
/// - `modified` (u64): Last modification time (epoch seconds)
/// - `checksum` (String): The file checksum (if `checksum` is true)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to check
    pub path: String,
    /// Whether to follow symlinks
    #[serde(default)]
    pub follow: bool,
    /// Get checksum of file
    #[serde(default)]
    pub checksum: bool,
    /// Checksum algorithm
    #[serde(default)]
    pub checksum_algorithm: ChecksumAlgorithm,
}

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Execute a stat task
pub async fn execute_stat_task(task: &StatTask, dry_run: bool) -> Result<serde_yaml::Value> {
    let mut result = serde_yaml::Mapping::new();

    if dry_run {
        println!("Would get statistics for: {}", task.path);
        result.insert(
            serde_yaml::Value::String("exists".to_string()),
            serde_yaml::Value::Bool(false),
        );
        return Ok(serde_yaml::Value::Mapping(result));
    }

    let path = Path::new(&task.path);
    let metadata_res = fs::metadata(path);

    if let Err(e) = &metadata_res {
        result.insert(
            serde_yaml::Value::String("exists".to_string()),
            serde_yaml::Value::Bool(false),
        );
        println!("File does not exist: {} ({})", task.path, e);
        return Ok(serde_yaml::Value::Mapping(result));
    }

    let metadata = metadata_res.unwrap();
    result.insert(
        serde_yaml::Value::String("exists".to_string()),
        serde_yaml::Value::Bool(true),
    );

    // Display basic file information
    println!("File: {}", task.path);
    println!("Size: {} bytes", metadata.len());
    result.insert(
        serde_yaml::Value::String("size".to_string()),
        serde_yaml::Value::Number(metadata.len().into()),
    );
    println!("Mode: {:o}", metadata.mode());
    result.insert(
        serde_yaml::Value::String("mode".to_string()),
        serde_yaml::Value::Number(metadata.mode().into()),
    );
    println!("Uid: {}", metadata.uid());
    result.insert(
        serde_yaml::Value::String("uid".to_string()),
        serde_yaml::Value::Number(metadata.uid().into()),
    );
    println!("Gid: {}", metadata.gid());
    result.insert(
        serde_yaml::Value::String("gid".to_string()),
        serde_yaml::Value::Number(metadata.gid().into()),
    );

    // Display timestamps
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
            println!("Modified: {}", duration.as_secs());
            result.insert(
                serde_yaml::Value::String("modified".to_string()),
                serde_yaml::Value::Number(duration.as_secs().into()),
            );
        }
    }
    if let Ok(accessed) = metadata.accessed() {
        if let Ok(duration) = accessed.duration_since(UNIX_EPOCH) {
            println!("Accessed: {}", duration.as_secs());
        }
    }
    if let Ok(created) = metadata.created() {
        if let Ok(duration) = created.duration_since(UNIX_EPOCH) {
            println!("Created: {}", duration.as_secs());
        }
    }

    // Display file type
    result.insert(
        serde_yaml::Value::String("is_file".to_string()),
        serde_yaml::Value::Bool(metadata.is_file()),
    );
    result.insert(
        serde_yaml::Value::String("is_dir".to_string()),
        serde_yaml::Value::Bool(metadata.is_dir()),
    );

    let file_type = if metadata.is_file() {
        "regular file"
    } else if metadata.is_dir() {
        "directory"
    } else if metadata.is_symlink() {
        "symbolic link"
    } else {
        "special file"
    };
    println!("Type: {}", file_type);

    // Calculate checksum if requested
    if task.checksum {
        match calculate_checksum(path, &task.checksum_algorithm) {
            Ok(checksum) => {
                println!(
                    "Checksum ({}): {}",
                    format!("{:?}", task.checksum_algorithm).to_lowercase(),
                    checksum
                );
                result.insert(
                    serde_yaml::Value::String("checksum".to_string()),
                    serde_yaml::Value::String(checksum),
                );
            }
            Err(e) => {
                println!("Failed to calculate checksum: {}", e);
            }
        }
    }

    Ok(serde_yaml::Value::Mapping(result))
}

/// Calculate file checksum
fn calculate_checksum(path: &Path, algorithm: &ChecksumAlgorithm) -> Result<String> {
    use std::io::Read;

    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open file for checksum: {}", path.display()))?;

    match algorithm {
        ChecksumAlgorithm::Md5 => {
            let mut hasher = md5::Context::new();
            let mut buffer = [0; 8192];
            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .with_context(|| "Failed to read file for MD5")?;
                if bytes_read == 0 {
                    break;
                }
                hasher.consume(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.compute()))
        }
        ChecksumAlgorithm::Sha1 => {
            use sha1::{Digest, Sha1};
            let mut hasher = Sha1::new();
            let mut buffer = [0; 8192];
            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .with_context(|| "Failed to read file for SHA1")?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        ChecksumAlgorithm::Sha256 => {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            let mut buffer = [0; 8192];
            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .with_context(|| "Failed to read file for SHA256")?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
        ChecksumAlgorithm::Sha512 => {
            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            let mut buffer = [0; 8192];
            loop {
                let bytes_read = file
                    .read(&mut buffer)
                    .with_context(|| "Failed to read file for SHA512")?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
            }
            Ok(format!("{:x}", hasher.finalize()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_stat_file() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        let test_content = "test content for stat";
        fs::write(&file_path, test_content).unwrap();

        let task = StatTask {
            description: None,
            path: file_path.clone(),
            follow: false,
            checksum: false,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
        };

        let result = execute_stat_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stat_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().to_str().unwrap().to_string();

        let task = StatTask {
            description: None,
            path: dir_path.clone(),
            follow: false,
            checksum: false,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
        };

        let result = execute_stat_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stat_nonexistent() {
        let task = StatTask {
            description: None,
            path: "/nonexistent/file/that/does/not/exist".to_string(),
            follow: false,
            checksum: false,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
        };

        let result = execute_stat_task(&task, false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to get metadata"));
    }

    #[tokio::test]
    async fn test_stat_with_checksum() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        let test_content = "test content for checksum";
        fs::write(&file_path, test_content).unwrap();

        let task = StatTask {
            description: None,
            path: file_path.clone(),
            follow: false,
            checksum: true,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
        };

        let result = execute_stat_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_checksum_sha256() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        let test_content = "test content for checksum";
        fs::write(&file_path, test_content).unwrap();

        let checksum = calculate_checksum(Path::new(&file_path), &ChecksumAlgorithm::Sha256);
        assert!(checksum.is_ok());
        let checksum_str = checksum.unwrap();
        assert!(!checksum_str.is_empty());
        assert!(checksum_str.len() == 64); // SHA256 produces 64 character hex string
    }

    #[test]
    fn test_calculate_checksum_md5() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        let test_content = "test content for checksum";
        fs::write(&file_path, test_content).unwrap();

        let checksum = calculate_checksum(Path::new(&file_path), &ChecksumAlgorithm::Md5);
        assert!(checksum.is_ok());
        let checksum_str = checksum.unwrap();
        assert!(!checksum_str.is_empty());
        assert!(checksum_str.len() == 32); // MD5 produces 32 character hex string
    }

    #[tokio::test]
    async fn test_stat_dry_run() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path().to_str().unwrap().to_string();
        fs::write(&file_path, "test").unwrap();

        let task = StatTask {
            description: None,
            path: file_path,
            follow: false,
            checksum: false,
            checksum_algorithm: ChecksumAlgorithm::Sha256,
        };

        let result = execute_stat_task(&task, true).await;
        assert!(result.is_ok());
    }
}
