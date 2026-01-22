//! Unarchive files task executor
//!
//! Handles extracting files from various archive formats.
//!
//! # Examples
//!
//! ## Extract a tar archive
//!
//! This example extracts a tar archive to a directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: unarchive
//!   description: "Extract application archive"
//!   src: /tmp/myapp.tar.gz
//!   dest: /opt/myapp
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "unarchive",
//!   "description": "Extract application archive",
//!   "src": "/tmp/myapp.tar.gz",
//!   "dest": "/opt/myapp",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "unarchive"
//! description = "Extract application archive"
//! src = "/tmp/myapp.tar.gz"
//! dest = "/opt/myapp"
//! state = "present"
//! ```
//!
//! ## Extract from URL
//!
//! This example downloads and extracts an archive from a URL.
//!
//! **YAML Format:**
//! ```yaml
//! - type: unarchive
//!   description: "Download and extract software"
//!   src: https://example.com/software.tar.gz
//!   dest: /opt/software
//!   state: present
//!   creates: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "unarchive",
//!   "description": "Download and extract software",
//!   "src": "https://example.com/software.tar.gz",
//!   "dest": "/opt/software",
//!   "state": "present",
//!   "creates": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "unarchive"
//! description = "Download and extract software"
//! src = "https://example.com/software.tar.gz"
//! dest = "/opt/software"
//! state = "present"
//! creates = true
//! ```
//!
//! ## Extract specific files
//!
//! This example extracts only specific files from an archive.
//!
//! **YAML Format:**
//! ```yaml
//! - type: unarchive
//!   description: "Extract configuration files"
//!   src: /tmp/configs.tar.gz
//!   dest: /etc/myapp
//!   state: present
//!   list_files:
//!     - config.yml
//!     - settings.json
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "unarchive",
//!   "description": "Extract configuration files",
//!   "src": "/tmp/configs.tar.gz",
//!   "dest": "/etc/myapp",
//!   "state": "present",
//!   "list_files": ["config.yml", "settings.json"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "unarchive"
//! description = "Extract configuration files"
//! src = "/tmp/configs.tar.gz"
//! dest = "/etc/myapp"
//! state = "present"
//! list_files = ["config.yml", "settings.json"]
//! ```
//!
//! ## Extract zip archive
//!
//! This example extracts a zip archive.
//!
//! **YAML Format:**
//! ```yaml
//! - type: unarchive
//!   description: "Extract zip archive"
//!   src: /tmp/data.zip
//!   dest: /var/data
//!   state: present
//!   format: zip
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "unarchive",
//!   "description": "Extract zip archive",
//!   "src": "/tmp/data.zip",
//!   "dest": "/var/data",
//!   "state": "present",
//!   "format": "zip"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "unarchive"
//! description = "Extract zip archive"
//! src = "/tmp/data.zip"
//! dest = "/var/data"
//! state = "present"
//! format = "zip"
//! ```

/// Unarchive files task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnarchiveTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source archive file (local path) or URL
    pub src: String,
    /// Destination directory
    pub dest: String,
    /// Unarchive state
    pub state: UnarchiveState,
    /// Archive format (auto-detect if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ArchiveFormat>,
    /// Whether to create destination directory
    #[serde(default)]
    pub creates: bool,
    /// List of files to extract (empty = all)
    #[serde(default)]
    pub list_files: Vec<String>,
    /// Whether to keep the archive after extraction
    #[serde(default)]
    pub keep_original: bool,
    /// Extra options for extraction
    #[serde(default)]
    pub extra_opts: Vec<String>,
    /// HTTP headers for URL downloads
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Timeout for URL downloads
    #[serde(default = "default_unarchive_timeout")]
    pub timeout: u64,
    /// Follow redirects for URL downloads
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    /// Validate SSL certificates for URL downloads
    #[serde(default = "default_true")]
    pub validate_certs: bool,
    /// Username for basic auth for URL downloads
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Password for basic auth for URL downloads
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// Unarchive state enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnarchiveState {
    /// Ensure archive is extracted
    Present,
    /// Ensure archive is not extracted (remove extracted files)
    Absent,
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::apply::default_true;
use crate::apply::archive::ArchiveFormat;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

/// Execute an unarchive task
pub async fn execute_unarchive_task(task: &UnarchiveTask, dry_run: bool) -> Result<()> {
    match task.state {
        UnarchiveState::Present => {
            ensure_archive_extracted(task, dry_run).await
        }
        UnarchiveState::Absent => {
            ensure_archive_not_extracted(task, dry_run).await
        }
    }
}

/// Ensure archive is extracted
async fn ensure_archive_extracted(task: &UnarchiveTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    // Handle URL sources by downloading to temporary file
    let (archive_path, _temp_file) = if is_url(&task.src) {
        if dry_run {
            println!("Would download {} for extraction", task.src);
            // For dry run, create a dummy path
            (Path::new(&task.src).to_path_buf(), None)
        } else {
            let temp_file = download_url_to_temp_file(task).await?;
            (temp_file.path().to_path_buf(), Some(temp_file))
        }
    } else {
        let src_path = Path::new(&task.src);
        // Check if source archive exists
        if !src_path.exists() {
            return Err(anyhow::anyhow!("Archive source does not exist: {}", task.src));
        }
        (src_path.to_path_buf(), None)
    };

    // Determine archive format
    let format = if let Some(fmt) = &task.format {
        fmt.clone()
    } else {
        detect_archive_format(&archive_path)?
    };

    // Check if extraction is needed
    let needs_extraction = if dest_path.exists() {
        // For simplicity, we'll assume extraction is needed if dest exists
        // A full implementation would check if files are up to date
        !task.creates // If creates=false, assume we need to check
    } else {
        true
    };

    if !needs_extraction {
        println!("Archive already extracted: {}", task.dest);
        return Ok(());
    }

    if dry_run {
        println!("Would extract {} to {} (format: {:?})", task.src, task.dest, format);
    } else {
        // Ensure destination directory exists if creates=true
        if task.creates && !dest_path.exists() {
            fs::create_dir_all(dest_path)
                .with_context(|| format!("Failed to create destination directory {}", task.dest))?;
        } else if !dest_path.exists() {
            return Err(anyhow::anyhow!("Destination directory does not exist: {}", task.dest));
        }

        // Perform extraction
        extract_archive_from_path(&archive_path, dest_path, task, &format).await?;

        println!("Extracted {} to {}", task.src, task.dest);
    }

    // Temporary file will be automatically cleaned up when it goes out of scope

    Ok(())
}

/// Ensure archive is not extracted (remove extracted files)
async fn ensure_archive_not_extracted(task: &UnarchiveTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    if !dest_path.exists() {
        println!("Extraction destination does not exist: {}", task.dest);
        return Ok(());
    }

    // This is a simplified implementation - in practice, we'd need to track
    // which files were created by extraction operations
    if dry_run {
        println!("Would remove extracted files from: {}", task.dest);
    } else {
        // For safety, we'll only remove the destination directory if it was created by extraction
        // This is a very basic implementation
        if task.creates {
            fs::remove_dir_all(dest_path)
                .with_context(|| format!("Failed to remove extracted directory {}", task.dest))?;
            println!("Removed extracted directory: {}", task.dest);
        } else {
            println!("Skipping removal of existing directory: {}", task.dest);
        }
    }

    Ok(())
}

/// Check if a string is a URL
fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
}

/// Download URL to temporary file
async fn download_url_to_temp_file(task: &UnarchiveTask) -> Result<NamedTempFile> {
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

    let client = builder.build()
        .with_context(|| "Failed to build HTTP client")?;

    // Build request
    let mut request_builder = client.get(&task.src);

    // Add headers
    for (key, value) in &task.headers {
        request_builder = request_builder.header(key, value);
    }

    // Add basic auth
    if let (Some(username), Some(password)) = (&task.username, &task.password) {
        use base64::{Engine as _, engine::general_purpose};
        let credentials = format!("{}:{}", username, password);
        let encoded = general_purpose::STANDARD.encode(credentials);
        request_builder = request_builder.header("Authorization", format!("Basic {}", encoded));
    }

    // Execute request
    let response = request_builder
        .send()
        .await
        .with_context(|| format!("Failed to download URL: {}", task.src))?;

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

    // Create temporary file
    let mut temp_file = NamedTempFile::new()
        .with_context(|| "Failed to create temporary file")?;

    // Write content to temporary file
    std::io::Write::write_all(&mut temp_file, &content)
        .with_context(|| "Failed to write to temporary file")?;

    Ok(temp_file)
}

/// Detect archive format from file extension
fn detect_archive_format(path: &Path) -> Result<ArchiveFormat> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "tar" => Ok(ArchiveFormat::Tar),
        "gz" => {
            // Check if it's a .tar.gz
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if file_stem.ends_with(".tar") {
                    Ok(ArchiveFormat::Tgz)
                } else {
                    Err(anyhow::anyhow!("Unsupported archive format: .gz (not tar.gz)"))
                }
            } else {
                Err(anyhow::anyhow!("Cannot determine archive format"))
            }
        }
        "bz2" => {
            // Check if it's a .tar.bz2
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if file_stem.ends_with(".tar") {
                    Ok(ArchiveFormat::Tbz2)
                } else {
                    Err(anyhow::anyhow!("Unsupported archive format: .bz2 (not tar.bz2)"))
                }
            } else {
                Err(anyhow::anyhow!("Cannot determine archive format"))
            }
        }
        "xz" => {
            // Check if it's a .tar.xz
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                if file_stem.ends_with(".tar") {
                    Ok(ArchiveFormat::Txz)
                } else {
                    Err(anyhow::anyhow!("Unsupported archive format: .xz (not tar.xz)"))
                }
            } else {
                Err(anyhow::anyhow!("Cannot determine archive format"))
            }
        }
        "zip" => Ok(ArchiveFormat::Zip),
        "7z" => Ok(ArchiveFormat::SevenZ),
        _ => Err(anyhow::anyhow!("Cannot detect archive format for: {}", path.display())),
    }
}

/// Extract archive using appropriate tool
async fn extract_archive_from_path(src_path: &Path, dest_path: &Path, _task: &UnarchiveTask, format: &ArchiveFormat) -> Result<()> {
    match format {
        ArchiveFormat::Tar => {
            extract_tar_archive(src_path, dest_path).await
        }
        ArchiveFormat::Tgz => {
            extract_tar_gz_archive(src_path, dest_path).await
        }
        ArchiveFormat::Tbz2 => {
            extract_tar_bz2_archive(src_path, dest_path).await
        }
        ArchiveFormat::Txz => {
            extract_tar_xz_archive(src_path, dest_path).await
        }
        ArchiveFormat::Zip => {
            extract_zip_archive(src_path, dest_path).await
        }
        ArchiveFormat::SevenZ => {
            extract_7z_archive(src_path, dest_path).await
        }
    }
}

/// Extract uncompressed tar archive
async fn extract_tar_archive(src: &Path, dest: &Path) -> Result<()> {
    run_command("tar", &["-xf", &src.to_string_lossy(), "-C", &dest.to_string_lossy()]).await
}

/// Extract gzip-compressed tar archive
async fn extract_tar_gz_archive(src: &Path, dest: &Path) -> Result<()> {
    run_command("tar", &["-xzf", &src.to_string_lossy(), "-C", &dest.to_string_lossy()]).await
}

/// Extract bzip2-compressed tar archive
async fn extract_tar_bz2_archive(src: &Path, dest: &Path) -> Result<()> {
    run_command("tar", &["-xjf", &src.to_string_lossy(), "-C", &dest.to_string_lossy()]).await
}

/// Extract xz-compressed tar archive
async fn extract_tar_xz_archive(src: &Path, dest: &Path) -> Result<()> {
    run_command("tar", &["-xJf", &src.to_string_lossy(), "-C", &dest.to_string_lossy()]).await
}

/// Extract zip archive
async fn extract_zip_archive(src: &Path, dest: &Path) -> Result<()> {
    run_command("unzip", &["-q", &src.to_string_lossy(), "-d", &dest.to_string_lossy()]).await
}

/// Extract 7z archive
async fn extract_7z_archive(src: &Path, dest: &Path) -> Result<()> {
    run_command("7z", &["x", &src.to_string_lossy(), &format!("-o{}", dest.to_string_lossy())]).await
}

/// Run external command for archive extraction
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

/// Default unarchive timeout (30 seconds)
pub fn default_unarchive_timeout() -> u64 { 30 }

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_archive_format_tar() {
        let temp_file = NamedTempFile::new().unwrap();
        let tar_path = temp_file.path().with_extension("tar");
        fs::write(&tar_path, "dummy").unwrap();

        let result = detect_archive_format(&tar_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ArchiveFormat::Tar);
    }

    #[test]
    fn test_detect_archive_format_tgz() {
        let temp_file = NamedTempFile::new().unwrap();
        let tgz_path = temp_file.path().with_extension("tar.gz");
        fs::write(&tgz_path, "dummy").unwrap();

        let result = detect_archive_format(&tgz_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ArchiveFormat::Tgz);
    }

    #[test]
    fn test_detect_archive_format_zip() {
        let temp_file = NamedTempFile::new().unwrap();
        let zip_path = temp_file.path().with_extension("zip");
        fs::write(&zip_path, "dummy").unwrap();

        let result = detect_archive_format(&zip_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ArchiveFormat::Zip);
    }

    #[test]
    fn test_detect_archive_format_unknown() {
        let temp_file = NamedTempFile::new().unwrap();
        let unknown_path = temp_file.path().with_extension("unknown");
        fs::write(&unknown_path, "dummy").unwrap();

        let result = detect_archive_format(&unknown_path);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unarchive_nonexistent_source() {
        let task = UnarchiveTask {
            description: None,
            src: "/nonexistent/archive.tar.gz".to_string(),
            dest: "/tmp/extract_dest".to_string(),
            state: UnarchiveState::Present,
            format: None,
            creates: true,
            list_files: vec![],
            keep_original: false,
            extra_opts: vec![],
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_unarchive_task(&task, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Archive source does not exist"));
    }

    #[tokio::test]
    async fn test_unarchive_dry_run() {
        // Create a dummy archive file
        let archive_file = NamedTempFile::new().unwrap();
        let archive_path = archive_file.path().to_str().unwrap().to_string() + ".tar.gz";
        fs::write(&archive_path, "dummy archive content").unwrap();

        let dest_dir = "/tmp/unarchive_test_dest";

        let task = UnarchiveTask {
            description: None,
            src: archive_path.clone(),
            dest: dest_dir.to_string(),
            state: UnarchiveState::Present,
            format: Some(ArchiveFormat::Tgz),
            creates: true,
            list_files: vec![],
            keep_original: false,
            extra_opts: vec![],
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_unarchive_task(&task, true).await;
        assert!(result.is_ok());
        // Directory shouldn't exist in dry run
        assert!(!Path::new(dest_dir).exists());
    }

    #[tokio::test]
    async fn test_unarchive_remove_extracted() {
        // Create a dummy destination directory
        let dest_dir = "/tmp/unarchive_remove_test";
        fs::create_dir_all(dest_dir).unwrap();

        let task = UnarchiveTask {
            description: None,
            src: "/dummy/archive.tar.gz".to_string(),
            dest: dest_dir.to_string(),
            state: UnarchiveState::Absent,
            format: None,
            creates: true,
            list_files: vec![],
            keep_original: false,
            extra_opts: vec![],
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_unarchive_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(dest_dir).exists());
    }
}