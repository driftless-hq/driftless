//! SSH authorized keys management
//!
//! This module manages SSH authorized keys for user accounts.
//! It supports adding, removing, and managing SSH public keys
//! in authorized_keys files with proper permissions and validation.
//!
//! # Examples
//!
//! ## Add SSH public key
//!
//! This example adds an SSH public key for a user.
//!
//! **YAML Format:**
//! ```yaml
//! - type: authorized_key
//!   description: "Add SSH key for admin user"
//!   user: admin
//!   state: present
//!   key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... user@host"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "authorized_key",
//!   "description": "Add SSH key for admin user",
//!   "user": "admin",
//!   "state": "present",
//!   "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... user@host"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "authorized_key"
//! description = "Add SSH key for admin user"
//! user = "admin"
//! state = "present"
//! key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... user@host"
//! ```
//!
//! ## Add SSH key from file
//!
//! This example adds an SSH public key from a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: authorized_key
//!   description: "Add SSH key from file"
//!   user: deploy
//!   state: present
//!   key_file: /tmp/id_rsa.pub
//!   comment: "Deployment key"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "authorized_key",
//!   "description": "Add SSH key from file",
//!   "user": "deploy",
//!   "state": "present",
//!   "key_file": "/tmp/id_rsa.pub",
//!   "comment": "Deployment key"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "authorized_key"
//! description = "Add SSH key from file"
//! user = "deploy"
//! state = "present"
//! key_file = "/tmp/id_rsa.pub"
//! comment = "Deployment key"
//! ```
//!
//! ## Add SSH key with restrictions
//!
//! This example adds an SSH key with command restrictions.
//!
//! **YAML Format:**
//! ```yaml
//! - type: authorized_key
//!   description: "Add restricted SSH key"
//!   user: backup
//!   state: present
//!   key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... backup@host"
//!   key_options: "command=\"/usr/local/bin/backup.sh\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "authorized_key",
//!   "description": "Add restricted SSH key",
//!   "user": "backup",
//!   "state": "present",
//!   "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... backup@host",
//!   "key_options": "command=\"/usr/local/bin/backup.sh\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "authorized_key"
//! description = "Add restricted SSH key"
//! user = "backup"
//! state = "present"
//! key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... backup@host"
//! key_options = "command=\"/usr/local/bin/backup.sh\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding"
//! ```
//!
//! ## Remove SSH key
//!
//! This example removes an SSH public key for a user.
//!
//! **YAML Format:**
//! ```yaml
//! - type: authorized_key
//!   description: "Remove SSH key"
//!   user: olduser
//!   state: absent
//!   key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... olduser@host"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "authorized_key",
//!   "description": "Remove SSH key",
//!   "user": "olduser",
//!   "state": "absent",
//!   "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... olduser@host"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "authorized_key"
//! description = "Remove SSH key"
//! user = "olduser"
//! state = "absent"
//! key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... olduser@host"
//! ```

use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{bail, Context, Result};

/// SSH authorized key management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorizedKeyTask {
    /// Optional description of what this task does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Target user for SSH key management
    pub user: String,

    /// SSH state (present/absent)
    pub state: AuthorizedKeyState,

    /// SSH public key content (inline)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Path to SSH public key file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_file: Option<String>,

    /// Key options (comma-separated list)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_options: Option<String>,

    /// Path to authorized_keys file (defaults to ~/.ssh/authorized_keys)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Whether to create .ssh directory if it doesn't exist
    #[serde(default = "default_true")]
    pub create_ssh_dir: bool,

    /// Whether to manage SSH directory permissions
    #[serde(default = "default_true")]
    pub manage_dir: bool,

    /// Whether to validate key format
    #[serde(default = "default_true")]
    pub validate_key: bool,

    /// Whether to deduplicate keys
    #[serde(default = "default_true")]
    pub unique: bool,

    /// Comment to identify this key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Authorized key state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorizedKeyState {
    /// Ensure key is present
    Present,
    /// Ensure key is absent
    Absent,
}

/// Execute SSH authorized key management task
pub async fn execute_authorized_key_task(task: &AuthorizedKeyTask, dry_run: bool) -> Result<()> {
    let (authorized_keys_path, ssh_dir) = if let Some(path) = &task.path {
        let authorized_keys_path = Path::new(path).to_path_buf();
        let ssh_dir = authorized_keys_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        (authorized_keys_path, ssh_dir)
    } else {
        let home_dir = get_user_home_dir(&task.user).context(format!(
            "Failed to get home directory for user {}",
            task.user
        ))?;
        let ssh_dir = home_dir.join(".ssh");
        let authorized_keys_path = ssh_dir.join("authorized_keys");
        (authorized_keys_path, ssh_dir)
    };

    match task.state {
        AuthorizedKeyState::Present => {
            ensure_key_present(task, &authorized_keys_path, &ssh_dir, dry_run).await
        }
        AuthorizedKeyState::Absent => ensure_key_absent(task, &authorized_keys_path, dry_run).await,
    }
}

/// Ensure SSH key is present in authorized_keys file
async fn ensure_key_present(
    task: &AuthorizedKeyTask,
    authorized_keys_path: &Path,
    ssh_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    // Get the key content
    let key_content = get_key_content(task)?;

    // Validate key format if requested
    if task.validate_key && !is_valid_ssh_key(&key_content) {
        bail!("Invalid SSH key format: {}", key_content);
    }

    // Create SSH directory if needed
    if task.create_ssh_dir && !ssh_dir.exists() {
        if dry_run {
            println!("DRY RUN: Would create SSH directory: {}", ssh_dir.display());
        } else {
            fs::create_dir_all(ssh_dir).with_context(|| {
                format!("Failed to create SSH directory: {}", ssh_dir.display())
            })?;
        }
    }

    // Set SSH directory permissions if managing directory
    if task.manage_dir && ssh_dir.exists() {
        if dry_run {
            println!(
                "DRY RUN: Would set SSH directory permissions to 0700: {}",
                ssh_dir.display()
            );
        } else {
            let permissions = fs::Permissions::from_mode(0o700);
            fs::set_permissions(ssh_dir, permissions).with_context(|| {
                format!(
                    "Failed to set SSH directory permissions: {}",
                    ssh_dir.display()
                )
            })?;
        }
    }

    // Read existing authorized_keys file
    let existing_content = if authorized_keys_path.exists() {
        fs::read_to_string(authorized_keys_path).with_context(|| {
            format!(
                "Failed to read authorized_keys file: {}",
                authorized_keys_path.display()
            )
        })?
    } else {
        String::new()
    };

    // Parse existing keys
    let mut existing_keys = parse_authorized_keys(&existing_content);

    // Format the new key entry
    let key_entry = format_key_entry(task, &key_content);

    // Check if key already exists
    let key_exists = existing_keys.contains(&key_entry);

    if key_exists && !task.unique {
        // Key already exists and we're not deduplicating
        return Ok(());
    }

    // Add or replace the key
    if task.unique {
        // Remove any existing instances of this key
        existing_keys.retain(|k| !k.contains(&key_content));
    }

    // Add the new key if it doesn't exist or if we're not in unique mode
    if !key_exists || !task.unique {
        existing_keys.insert(key_entry);
    }

    // Write the updated authorized_keys file
    let new_content = existing_keys.into_iter().collect::<Vec<_>>().join("\n") + "\n";

    if dry_run {
        println!(
            "DRY RUN: Would write to authorized_keys file: {}",
            authorized_keys_path.display()
        );
        println!("DRY RUN: Content would be:\n{}", new_content);
    } else {
        fs::write(authorized_keys_path, new_content).with_context(|| {
            format!(
                "Failed to write authorized_keys file: {}",
                authorized_keys_path.display()
            )
        })?;
    }

    // Set authorized_keys file permissions
    if authorized_keys_path.exists() && !dry_run {
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(authorized_keys_path, permissions).with_context(|| {
            format!(
                "Failed to set authorized_keys file permissions: {}",
                authorized_keys_path.display()
            )
        })?;
    }

    Ok(())
}

/// Ensure SSH key is absent from authorized_keys file
async fn ensure_key_absent(
    task: &AuthorizedKeyTask,
    authorized_keys_path: &Path,
    dry_run: bool,
) -> Result<()> {
    if !authorized_keys_path.exists() {
        return Ok(());
    }

    // Get the key content to remove
    let key_content = match get_key_content(task) {
        Ok(content) => content,
        Err(_) => return Ok(()), // If we can't get the key content, assume it's already absent
    };

    // Read existing authorized_keys file
    let existing_content = fs::read_to_string(authorized_keys_path).with_context(|| {
        format!(
            "Failed to read authorized_keys file: {}",
            authorized_keys_path.display()
        )
    })?;

    // Parse existing keys
    let mut existing_keys = parse_authorized_keys(&existing_content);

    // Remove keys containing the target key content
    let original_count = existing_keys.len();
    existing_keys.retain(|k| !k.contains(&key_content));

    if existing_keys.len() == original_count {
        // No keys were removed
        return Ok(());
    }

    // Write the updated authorized_keys file
    let new_content = if existing_keys.is_empty() {
        String::new()
    } else {
        existing_keys.into_iter().collect::<Vec<_>>().join("\n") + "\n"
    };

    if dry_run {
        println!(
            "DRY RUN: Would update authorized_keys file: {}",
            authorized_keys_path.display()
        );
        println!("DRY RUN: Content would be:\n{}", new_content);
    } else {
        fs::write(authorized_keys_path, new_content).with_context(|| {
            format!(
                "Failed to write authorized_keys file: {}",
                authorized_keys_path.display()
            )
        })?;
    }

    Ok(())
}

/// Get the SSH key content from task parameters
fn get_key_content(task: &AuthorizedKeyTask) -> Result<String> {
    if let Some(key) = &task.key {
        Ok(key.clone())
    } else if let Some(key_file) = &task.key_file {
        fs::read_to_string(key_file)
            .with_context(|| format!("Failed to read SSH key file: {}", key_file))
    } else {
        bail!("Either 'key' or 'key_file' must be specified");
    }
}

/// Get user's home directory
fn get_user_home_dir(username: &str) -> Result<std::path::PathBuf> {
    // Try to get home directory from /etc/passwd
    let passwd_content = fs::read_to_string("/etc/passwd").context("Failed to read /etc/passwd")?;

    for line in passwd_content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 6 && parts[0] == username {
            return Ok(std::path::PathBuf::from(parts[5]));
        }
    }

    bail!("User {} not found in /etc/passwd", username);
}

/// Parse authorized_keys file content into individual key entries
fn parse_authorized_keys(content: &str) -> HashSet<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect()
}

/// Format a key entry with options and comment
fn format_key_entry(task: &AuthorizedKeyTask, key_content: &str) -> String {
    let mut parts = Vec::new();

    // Add key options if specified
    if let Some(options) = &task.key_options {
        if !options.trim().is_empty() {
            parts.push(options.trim().to_string());
        }
    }

    // Add the key content
    parts.push(key_content.to_string());

    // Add comment if specified
    if let Some(comment) = &task.comment {
        if !comment.trim().is_empty() {
            parts.push(format!("# {}", comment.trim()));
        }
    }

    parts.join(" ")
}

/// Validate SSH key format
fn is_valid_ssh_key(key: &str) -> bool {
    // Basic SSH key format validation
    // SSH keys typically start with ssh-rsa, ssh-ed25519, ecdsa-sha2-nistp256, etc.
    let key_parts: Vec<&str> = key.split_whitespace().collect();

    if key_parts.len() < 2 {
        return false;
    }

    let key_type = key_parts[0];
    let valid_types = [
        "ssh-rsa",
        "ssh-ed25519",
        "ecdsa-sha2-nistp256",
        "ecdsa-sha2-nistp384",
        "ecdsa-sha2-nistp521",
        "ssh-dss",
    ];

    valid_types.contains(&key_type)
}

pub fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_authorized_key_present() {
        let temp_dir = TempDir::new().unwrap();
        let ssh_dir = temp_dir.path().join(".ssh");
        let authorized_keys = ssh_dir.join("authorized_keys");

        let task = AuthorizedKeyTask {
            description: Some("Add SSH key".to_string()),
            user: "testuser".to_string(),
            state: AuthorizedKeyState::Present,
            key: Some(
                "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDh8BWk test@example.com".to_string(),
            ),
            key_file: None,
            key_options: Some("no-port-forwarding".to_string()),
            path: Some(authorized_keys.to_str().unwrap().to_string()),
            create_ssh_dir: true,
            manage_dir: true,
            validate_key: true,
            unique: true,
            comment: Some("Test key".to_string()),
        };

        // Mock the home directory lookup
        execute_authorized_key_task(&task, false).await.unwrap();

        assert!(ssh_dir.exists());
        assert!(authorized_keys.exists());

        let content = fs::read_to_string(&authorized_keys).unwrap();
        assert!(content.contains("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDh8BWk"));
        assert!(content.contains("no-port-forwarding"));
        assert!(content.contains("# Test key"));
    }

    #[tokio::test]
    async fn test_authorized_key_absent() {
        let temp_dir = TempDir::new().unwrap();
        let authorized_keys = temp_dir.path().join("authorized_keys");

        // Create initial file with a key
        let initial_content =
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDh8BWk test@example.com\n";
        fs::write(&authorized_keys, initial_content).unwrap();

        let task = AuthorizedKeyTask {
            description: Some("Remove SSH key".to_string()),
            user: "testuser".to_string(),
            state: AuthorizedKeyState::Absent,
            key: Some(
                "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDh8BWk test@example.com".to_string(),
            ),
            key_file: None,
            key_options: None,
            path: Some(authorized_keys.to_str().unwrap().to_string()),
            create_ssh_dir: false,
            manage_dir: false,
            validate_key: false,
            unique: false,
            comment: None,
        };

        execute_authorized_key_task(&task, false).await.unwrap();

        let content = fs::read_to_string(&authorized_keys).unwrap();
        assert!(!content.contains("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDh8BWk"));
    }

    #[test]
    fn test_is_valid_ssh_key() {
        assert!(is_valid_ssh_key(
            "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDh8BWk"
        ));
        assert!(is_valid_ssh_key(
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGmJyR2l test"
        ));
        assert!(!is_valid_ssh_key("invalid-key"));
        assert!(!is_valid_ssh_key(""));
    }
}
