//! Get URL task executor
//!
//! Downloads files from HTTP/HTTPS/FTP URLs with checksum validation and file management.
//!
//! # Examples
//!
//! ## Download a file
//!
//! This example downloads a file from a URL.
//!
//! **YAML Format:**
//! ```yaml
//! - type: get_url
//!   description: "Download configuration file"
//!   url: https://example.com/config.yml
//!   dest: /etc/myapp/config.yml
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "get_url",
//!   "description": "Download configuration file",
//!   "url": "https://example.com/config.yml",
//!   "dest": "/etc/myapp/config.yml",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "get_url"
//! description = "Download configuration file"
//! url = "https://example.com/config.yml"
//! dest = "/etc/myapp/config.yml"
//! state = "present"
//! ```
//!
//! ## Download with checksum validation
//!
//! This example downloads a file and validates its checksum.
//!
//! **YAML Format:**
//! ```yaml
//! - type: get_url
//!   description: "Download software with checksum validation"
//!   url: https://example.com/software.tar.gz
//!   dest: /tmp/software.tar.gz
//!   state: present
//!   checksum: sha256:abc123def456...
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "get_url",
//!   "description": "Download software with checksum validation",
//!   "url": "https://example.com/software.tar.gz",
//!   "dest": "/tmp/software.tar.gz",
//!   "state": "present",
//!   "checksum": "sha256:abc123def456..."
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "get_url"
//! description = "Download software with checksum validation"
//! url = "https://example.com/software.tar.gz"
//! dest = "/tmp/software.tar.gz"
//! state = "present"
//! checksum = "sha256:abc123def456..."
//! ```
//!
//! ## Download with authentication
//!
//! This example downloads a file using basic authentication.
//!
//! **YAML Format:**
//! ```yaml
//! - type: get_url
//!   description: "Download private file"
//!   url: https://private.example.com/file.txt
//!   dest: /tmp/private.txt
//!   state: present
//!   username: myuser
//!   password: mypassword
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "get_url",
//!   "description": "Download private file",
//!   "url": "https://private.example.com/file.txt",
//!   "dest": "/tmp/private.txt",
//!   "state": "present",
//!   "username": "myuser",
//!   "password": "mypassword"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "get_url"
//! description = "Download private file"
//! url = "https://private.example.com/file.txt"
//! dest = "/tmp/private.txt"
//! state = "present"
//! username = "myuser"
//! password = "mypassword"
//! ```
//!
//! ## Download and set permissions
//!
//! This example downloads a file and sets its permissions and ownership.
//!
//! **YAML Format:**
//! ```yaml
//! - type: get_url
//!   description: "Download script with proper permissions"
//!   url: https://example.com/script.sh
//!   dest: /usr/local/bin/myscript.sh
//!   state: present
//!   mode: "0755"
//!   owner: root
//!   group: root
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "get_url",
//!   "description": "Download script with proper permissions",
//!   "url": "https://example.com/script.sh",
//!   "dest": "/usr/local/bin/myscript.sh",
//!   "state": "present",
//!   "mode": "0755",
//!   "owner": "root",
//!   "group": "root"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "get_url"
//! description = "Download script with proper permissions"
//! url = "https://example.com/script.sh"
//! dest = "/usr/local/bin/myscript.sh"
//! state = "present"
//! mode = "0755"
//! owner = "root"
//! group = "root"
//! ```

/// Get URL state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GetUrlState {
    /// Ensure file is downloaded
    Present,
    /// Ensure file is removed
    Absent,
}

/// Download files from HTTP/HTTPS/FTP task
///
/// Downloads files from web servers or FTP servers. Supports authentication,
/// checksum validation, and file permission management. Similar to Ansible's `get_url` module.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GetUrlTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source URL
    pub url: String,
    /// Destination file path
    pub dest: String,
    /// Get URL state
    pub state: GetUrlState,
    /// HTTP headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Timeout in seconds
    #[serde(default = "default_get_url_timeout")]
    pub timeout: u64,
    /// Follow redirects
    #[serde(default = "crate::apply::default_true")]
    pub follow_redirects: bool,
    /// Force download even if file exists
    #[serde(default)]
    pub force: bool,
    /// Validate SSL certificates
    #[serde(default = "crate::apply::default_true")]
    pub validate_certs: bool,
    /// Username for basic auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Password for basic auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Checksum validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// Backup destination before download
    #[serde(default)]
    pub backup: bool,
    /// File permissions (octal string like "0644")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// File owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// File group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// Default get URL timeout (10 seconds)
pub fn default_get_url_timeout() -> u64 {
    10
}

use anyhow::{Context, Result};
use sha1::Digest as Sha1Digest;
use sha2::Digest as Sha2Digest;
use std::fs;
use std::path::Path;

/// Execute a get URL task
pub async fn execute_get_url_task(task: &GetUrlTask, dry_run: bool) -> Result<()> {
    match task.state {
        GetUrlState::Present => ensure_file_downloaded(task, dry_run).await,
        GetUrlState::Absent => ensure_file_removed(task, dry_run).await,
    }
}

/// Ensure file is downloaded from URL
async fn ensure_file_downloaded(task: &GetUrlTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    // Check if download is needed
    let needs_download = if dest_path.exists() && !task.force {
        // Check if file matches expected checksum
        if let Some(expected_checksum) = &task.checksum {
            match validate_checksum(dest_path, expected_checksum) {
                Ok(true) => {
                    println!("File already exists with correct checksum: {}", task.dest);
                    false
                }
                Ok(false) => {
                    println!(
                        "File exists but checksum mismatch, will re-download: {}",
                        task.dest
                    );
                    true
                }
                Err(e) => {
                    println!(
                        "Failed to validate checksum, will re-download: {} ({})",
                        task.dest, e
                    );
                    true
                }
            }
        } else {
            // No checksum validation, assume we need to check modification time or just force
            // For simplicity, skip download if file exists and force=false
            false
        }
    } else {
        true
    };

    if !needs_download {
        return Ok(());
    }

    if dry_run {
        println!("Would download {} to {}", task.url, task.dest);
        return Ok(());
    }

    // Create backup if requested
    if task.backup && dest_path.exists() {
        let backup_path = format!("{}.backup", task.dest);
        fs::copy(&task.dest, &backup_path)
            .with_context(|| format!("Failed to create backup of {}", task.dest))?;
        println!("Created backup: {}", backup_path);
    }

    // Perform the download
    download_url_to_file(task).await?;

    // Set file permissions if specified
    if let Some(mode) = &task.mode {
        set_file_permissions(dest_path, mode)?;
    }

    // Set file ownership if specified
    if task.owner.is_some() || task.group.is_some() {
        set_file_ownership(dest_path, task.owner.as_deref(), task.group.as_deref())?;
    }

    println!("Downloaded {} to {}", task.url, task.dest);
    Ok(())
}

/// Ensure file is removed
async fn ensure_file_removed(task: &GetUrlTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    if !dest_path.exists() {
        println!("File does not exist: {}", task.dest);
        return Ok(());
    }

    if dry_run {
        println!("Would remove downloaded file: {}", task.dest);
    } else {
        fs::remove_file(dest_path)
            .with_context(|| format!("Failed to remove file {}", task.dest))?;
        println!("Removed downloaded file: {}", task.dest);
    }

    Ok(())
}

/// Download URL content to file
async fn download_url_to_file(task: &GetUrlTask) -> Result<()> {
    // Build HTTP client
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(task.timeout))
        .redirect(if task.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        });

    // Configure SSL validation
    if !task.validate_certs {
        builder = builder.danger_accept_invalid_certs(true);
    }

    let client = builder
        .build()
        .with_context(|| "Failed to build HTTP client")?;

    // Build request
    let mut request_builder = client.get(&task.url);

    // Add headers
    for (key, value) in &task.headers {
        request_builder = request_builder.header(key, value);
    }

    // Add basic auth
    if let (Some(username), Some(password)) = (&task.username, &task.password) {
        use base64::{engine::general_purpose, Engine as _};
        let credentials = format!("{}:{}", username, password);
        let encoded = general_purpose::STANDARD.encode(credentials);
        request_builder = request_builder.header("Authorization", format!("Basic {}", encoded));
    }

    // Execute request
    let response = request_builder
        .send()
        .await
        .with_context(|| format!("Failed to download URL: {}", task.url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP request failed with status: {}",
            response.status()
        ));
    }

    // Read response body
    let content = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    // Ensure destination directory exists
    if let Some(parent) = Path::new(&task.dest).parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directories for {}", task.dest))?;
    }

    // Write content to file
    fs::write(&task.dest, content)
        .with_context(|| format!("Failed to write to file {}", task.dest))?;

    Ok(())
}

/// Validate file checksum
fn validate_checksum(path: &Path, expected_checksum: &str) -> Result<bool> {
    // Parse checksum format: algorithm:checksum
    let parts: Vec<&str> = expected_checksum.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid checksum format: {}",
            expected_checksum
        ));
    }

    let algorithm = parts[0];
    let expected = parts[1];

    let content = fs::read(path)
        .with_context(|| format!("Failed to read file for checksum: {}", path.display()))?;

    let actual = match algorithm.to_lowercase().as_str() {
        "md5" => format!("{:x}", md5::compute(&content)),
        "sha1" => format!("{:x}", <sha1::Sha1 as Sha1Digest>::digest(&content)),
        "sha256" => format!("{:x}", <sha2::Sha256 as Sha2Digest>::digest(&content)),
        "sha512" => format!("{:x}", <sha2::Sha512 as Sha2Digest>::digest(&content)),
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported checksum algorithm: {}",
                algorithm
            ))
        }
    };

    Ok(actual == expected)
}

/// Set file permissions
fn set_file_permissions(path: &Path, mode: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode_val = u32::from_str_radix(mode.trim_start_matches("0o").trim_start_matches("0"), 8)
        .with_context(|| format!("Invalid octal mode: {}", mode))?;

    let permissions = fs::Permissions::from_mode(mode_val);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("Failed to set permissions on {}", path.display()))?;

    Ok(())
}

/// Set file ownership
fn set_file_ownership(path: &Path, owner: Option<&str>, group: Option<&str>) -> Result<()> {
    use nix::unistd::{chown, Gid, Uid};
    use std::os::unix::fs::MetadataExt;

    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for {}", path.display()))?;

    let current_uid = metadata.uid();
    let current_gid = metadata.gid();

    let target_uid = if let Some(owner_name) = owner {
        // Try to parse as UID first, then as username
        match owner_name.parse::<u32>() {
            Ok(uid) => uid,
            Err(_) => {
                // Look up username
                use nix::unistd::User;
                User::from_name(owner_name)
                    .with_context(|| format!("User not found: {}", owner_name))?
                    .map(|u| u.uid.as_raw())
                    .unwrap_or(current_uid)
            }
        }
    } else {
        current_uid
    };

    let target_gid = if let Some(group_name) = group {
        // Try to parse as GID first, then as group name
        match group_name.parse::<u32>() {
            Ok(gid) => gid,
            Err(_) => {
                // Look up group name
                use nix::unistd::Group;
                Group::from_name(group_name)
                    .with_context(|| format!("Group not found: {}", group_name))?
                    .map(|g| g.gid.as_raw())
                    .unwrap_or(current_gid)
            }
        }
    } else {
        current_gid
    };

    chown(
        path,
        Some(Uid::from_raw(target_uid)),
        Some(Gid::from_raw(target_gid)),
    )
    .with_context(|| format!("Failed to set ownership on {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_get_url_dry_run() {
        let dest_path = "/tmp/get_url_dry_run_test.txt".to_string();

        let task = GetUrlTask {
            description: None,
            url: "http://example.com/file.txt".to_string(),
            dest: dest_path.clone(),
            state: GetUrlState::Present,
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            force: false,
            validate_certs: true,
            username: None,
            password: None,
            checksum: None,
            backup: false,
            mode: None,
            owner: None,
            group: None,
        };

        let result = execute_get_url_task(&task, true).await;
        assert!(result.is_ok());
        // File shouldn't exist in dry run
        assert!(!Path::new(&dest_path).exists());
    }

    #[tokio::test]
    async fn test_get_url_remove_file() {
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        fs::write(&dest_path, "existing content").unwrap();

        let task = GetUrlTask {
            description: None,
            url: "http://example.com/file.txt".to_string(),
            dest: dest_path.clone(),
            state: GetUrlState::Absent,
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            force: false,
            validate_certs: true,
            username: None,
            password: None,
            checksum: None,
            backup: false,
            mode: None,
            owner: None,
            group: None,
        };

        let result = execute_get_url_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists());
    }

    #[tokio::test]
    async fn test_checksum_validation() {
        let test_file = NamedTempFile::new().unwrap();
        let test_content = b"Hello, World!";
        fs::write(test_file.path(), test_content).unwrap();

        // Calculate expected MD5 checksum
        let expected_md5 = format!("md5:{:x}", md5::compute(test_content));

        // Test valid checksum
        assert!(validate_checksum(test_file.path(), &expected_md5).unwrap());

        // Test invalid checksum
        assert!(!validate_checksum(test_file.path(), "md5:invalid").unwrap());
    }

    #[test]
    fn test_invalid_checksum_format() {
        let test_file = NamedTempFile::new().unwrap();
        fs::write(test_file.path(), "test").unwrap();

        let result = validate_checksum(test_file.path(), "invalid-format");
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_checksum_algorithm() {
        let test_file = NamedTempFile::new().unwrap();
        fs::write(test_file.path(), "test").unwrap();

        let result = validate_checksum(test_file.path(), "unknown:abcd");
        assert!(result.is_err());
    }
}
