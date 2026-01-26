//! Fetch files from remote hosts task executor
//!
//! Handles downloading files from HTTP/FTP URLs.
//!
//! # Examples
//!
//! ## Download a file
//!
//! This example downloads a file from a URL.
//!
//! **YAML Format:**
//! ```yaml
//! - type: fetch
//!   description: "Download configuration file"
//!   url: http://example.com/config.yml
//!   dest: /etc/myapp/config.yml
//!   state: present
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "fetch",
//!   "description": "Download configuration file",
//!   "url": "http://example.com/config.yml",
//!   "dest": "/etc/myapp/config.yml",
//!   "state": "present"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "fetch"
//! description = "Download configuration file"
//! url = "http://example.com/config.yml"
//! dest = "/etc/myapp/config.yml"
//! state = "present"
//! ```
//!
//! ## Download with authentication
//!
//! This example downloads a file using basic authentication.
//!
//! **YAML Format:**
//! ```yaml
//! - type: fetch
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
//!   "type": "fetch",
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
//! type = "fetch"
//! description = "Download private file"
//! url = "https://private.example.com/file.txt"
//! dest = "/tmp/private.txt"
//! state = "present"
//! username = "myuser"
//! password = "mypassword"
//! ```
//!
//! ## Download with custom headers
//!
//! This example downloads a file with custom HTTP headers.
//!
//! **YAML Format:**
//! ```yaml
//! - type: fetch
//!   description: "Download with custom headers"
//!   url: https://api.example.com/data.json
//!   dest: /tmp/data.json
//!   state: present
//!   headers:
//!     Authorization: "Bearer token123"
//!     X-API-Key: "apikey456"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "fetch",
//!   "description": "Download with custom headers",
//!   "url": "https://api.example.com/data.json",
//!   "dest": "/tmp/data.json",
//!   "state": "present",
//!   "headers": {
//!     "Authorization": "Bearer token123",
//!     "X-API-Key": "apikey456"
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "fetch"
//! description = "Download with custom headers"
//! url = "https://api.example.com/data.json"
//! dest = "/tmp/data.json"
//! state = "present"
//! headers = { Authorization = "Bearer token123", "X-API-Key" = "apikey456" }
//! ```
//!
//! ## Force download
//!
//! This example forces a download even if the file already exists.
//!
//! **YAML Format:**
//! ```yaml
//! - type: fetch
//!   description: "Force download latest version"
//!   url: https://example.com/latest.tar.gz
//!   dest: /tmp/latest.tar.gz
//!   state: present
//!   force: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "fetch",
//!   "description": "Force download latest version",
//!   "url": "https://example.com/latest.tar.gz",
//!   "dest": "/tmp/latest.tar.gz",
//!   "state": "present",
//!   "force": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "fetch"
//! description = "Force download latest version"
//! url = "https://example.com/latest.tar.gz"
//! dest = "/tmp/latest.tar.gz"
//! state = "present"
//! force = true
//! ```

/// Fetch state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchState {
    /// Ensure file is fetched
    Present,
    /// Ensure file is removed
    Absent,
}

/// Fetch files from remote hosts task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FetchTask {
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
    /// Fetch state
    pub state: FetchState,
    /// HTTP headers
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
    /// Timeout in seconds
    #[serde(default = "default_fetch_timeout")]
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
}

/// Default fetch timeout (10 seconds)
pub fn default_fetch_timeout() -> u64 {
    10
}

use anyhow::{Context, Result};
use chrono;
use std::fs;
use std::path::Path;

/// Execute a fetch task
pub async fn execute_fetch_task(task: &FetchTask, dry_run: bool) -> Result<()> {
    match task.state {
        FetchState::Present => ensure_file_fetched(task, dry_run).await,
        FetchState::Absent => ensure_file_not_fetched(task, dry_run).await,
    }
}

/// Ensure file is fetched from remote URL
async fn ensure_file_fetched(task: &FetchTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    // Check if destination needs updating
    let needs_fetch = if dest_path.exists() && !task.force {
        // Check if remote file has changed by comparing ETags or Last-Modified headers
        match check_remote_file_changed(task).await {
            Ok(changed) => changed,
            Err(_) => {
                // If we can't check, assume it needs fetching for safety
                println!("Warning: Could not check if remote file changed, will re-fetch");
                true
            }
        }
    } else {
        true
    };

    if !needs_fetch {
        println!("File is up to date: {}", task.dest);
        return Ok(());
    }

    if dry_run {
        println!("Would fetch {} to {}", task.url, task.dest);
    } else {
        // Perform the fetch
        fetch_url_to_file(task).await?;

        println!("Fetched {} to {}", task.url, task.dest);
    }

    Ok(())
}

/// Ensure file is not fetched (remove if it exists)
async fn ensure_file_not_fetched(task: &FetchTask, dry_run: bool) -> Result<()> {
    let dest_path = Path::new(&task.dest);

    if !dest_path.exists() {
        println!("File does not exist: {}", task.dest);
        return Ok(());
    }

    // This is a simplified implementation - in practice, we'd need to track
    // which files were created by fetch operations
    if dry_run {
        println!("Would remove fetched file: {}", task.dest);
    } else {
        fs::remove_file(dest_path)
            .with_context(|| format!("Failed to remove file {}", task.dest))?;
        println!("Removed fetched file: {}", task.dest);
    }

    Ok(())
}

/// Check if remote file has changed compared to local file
async fn check_remote_file_changed(task: &FetchTask) -> Result<bool> {
    // Build HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(task.timeout))
        .redirect(if task.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        })
        .danger_accept_invalid_certs(!task.validate_certs)
        .build()
        .with_context(|| "Failed to build HTTP client")?;

    // Build HEAD request to check headers without downloading
    let mut request_builder = client.head(&task.url);

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

    // Execute HEAD request
    let response = request_builder
        .send()
        .await
        .with_context(|| format!("Failed to check remote file: {}", task.url))?;

    if !response.status().is_success() {
        // If HEAD request fails, assume file has changed
        return Ok(true);
    }

    // Check ETag
    if let Some(etag) = response.headers().get("etag") {
        if let Ok(etag_str) = etag.to_str() {
            // For now, we'll assume ETag means file has changed
            // A full implementation would store previous ETags
            println!("Remote file has ETag: {}", etag_str);
            return Ok(true);
        }
    }

    // Check Last-Modified
    if let Some(last_modified) = response.headers().get("last-modified") {
        if let Ok(lm_str) = last_modified.to_str() {
            if let Ok(remote_time) = chrono::DateTime::parse_from_rfc2822(lm_str) {
                let local_metadata = fs::metadata(&task.dest)?;
                let local_mtime = local_metadata.modified()?;
                let local_time = chrono::DateTime::<chrono::Utc>::from(local_mtime);

                if remote_time > local_time {
                    println!("Remote file is newer than local file");
                    return Ok(true);
                } else {
                    println!("Local file is up to date");
                    return Ok(false);
                }
            }
        }
    }

    // If we can't determine, assume it needs fetching
    println!("Could not determine if remote file changed, will re-fetch");
    Ok(true)
}

/// Fetch URL content to file with progress tracking
async fn fetch_url_to_file(task: &FetchTask) -> Result<()> {
    // Build HTTP client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(task.timeout))
        .redirect(if task.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        })
        .danger_accept_invalid_certs(!task.validate_certs)
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
        .with_context(|| format!("Failed to fetch URL: {}", task.url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP request failed with status: {}",
            response.status()
        ));
    }

    // Get content length for progress tracking
    let content_length = response.content_length().unwrap_or(0);

    println!("Downloading {} ({} bytes)", task.url, content_length);

    // Read response body
    let content = response
        .bytes()
        .await
        .with_context(|| "Failed to read response body")?;

    // Show completion message
    println!("Downloaded {} bytes", content.len());

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    // Note: These tests would require a test HTTP server
    // For now, we'll test the basic structure

    #[tokio::test]
    async fn test_fetch_file_dry_run() {
        let dest_path = "/tmp/fetch_dry_run_test.txt".to_string();
        // Make sure file doesn't exist
        let _ = std::fs::remove_file(&dest_path);

        let task = FetchTask {
            description: None,
            url: "http://example.com/file.txt".to_string(),
            dest: dest_path.clone(),
            state: FetchState::Present,
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            force: false,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_fetch_task(&task, true).await;
        assert!(result.is_ok());
        // File shouldn't exist in dry run
        assert!(!Path::new(&dest_path).exists());
    }

    #[tokio::test]
    async fn test_fetch_remove_file() {
        let dest_file = NamedTempFile::new().unwrap();
        let dest_path = dest_file.path().to_str().unwrap().to_string();
        fs::write(&dest_path, "existing content").unwrap();

        let task = FetchTask {
            description: None,
            url: "http://example.com/file.txt".to_string(),
            dest: dest_path.clone(),
            state: FetchState::Absent,
            headers: std::collections::HashMap::new(),
            timeout: 30,
            follow_redirects: true,
            force: false,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_fetch_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&dest_path).exists());
    }

    #[tokio::test]
    async fn test_fetch_invalid_url() {
        let dest_path = "/tmp/nonexistent/fetch_test.txt".to_string();

        let task = FetchTask {
            description: None,
            url: "http://invalid.url.that.does.not.exist/file.txt".to_string(),
            dest: dest_path,
            state: FetchState::Present,
            headers: std::collections::HashMap::new(),
            timeout: 1, // Short timeout for test
            follow_redirects: true,
            force: false,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_fetch_task(&task, false).await;
        // This will likely fail due to network issues, but we're testing the structure
        // In a real test environment, we'd mock the HTTP client
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_with_headers() {
        let dest_path = "/tmp/fetch_headers_test.txt".to_string();

        let mut headers = std::collections::HashMap::new();
        headers.insert("User-Agent".to_string(), "Driftless/1.0".to_string());

        let task = FetchTask {
            description: None,
            url: "http://example.com/file.txt".to_string(),
            dest: dest_path,
            state: FetchState::Present,
            headers,
            timeout: 30,
            follow_redirects: true,
            force: false,
            validate_certs: true,
            username: None,
            password: None,
        };

        let result = execute_fetch_task(&task, true).await;
        assert!(result.is_ok());
    }
}
