//! Sudoers configuration management
//!
//! This module manages sudoers configuration files, allowing users to
//! add or remove sudo privileges for users and groups with proper validation.
//!
//! # Examples
//!
//! ## Grant sudo access to user
//!
//! This example grants full sudo access to a user.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sudoers
//!   description: "Grant sudo access to admin user"
//!   state: present
//!   name: admin
//!   commands: ["ALL"]
//!   hosts: ["ALL"]
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sudoers",
//!   "description": "Grant sudo access to admin user",
//!   "state": "present",
//!   "name": "admin",
//!   "commands": ["ALL"],
//!   "hosts": ["ALL"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sudoers"
//! description = "Grant sudo access to admin user"
//! state = "present"
//! name = "admin"
//! commands = ["ALL"]
//! hosts = ["ALL"]
//! ```
//!
//! ## Grant sudo access to group
//!
//! This example grants sudo access to a group.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sudoers
//!   description: "Grant sudo access to wheel group"
//!   state: present
//!   name: wheel
//!   group: true
//!   commands: ["ALL"]
//!   hosts: ["ALL"]
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sudoers",
//!   "description": "Grant sudo access to wheel group",
//!   "state": "present",
//!   "name": "wheel",
//!   "group": true,
//!   "commands": ["ALL"],
//!   "hosts": ["ALL"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sudoers"
//! description = "Grant sudo access to wheel group"
//! state = "present"
//! name = "wheel"
//! group = true
//! commands = ["ALL"]
//! hosts = ["ALL"]
//! ```
//!
//! ## Grant passwordless sudo for specific commands
//!
//! This example grants passwordless sudo access for specific commands.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sudoers
//!   description: "Grant passwordless sudo for service management"
//!   state: present
//!   name: deploy
//!   commands: ["/usr/bin/systemctl", "/usr/bin/service"]
//!   hosts: ["ALL"]
//!   nopasswd: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sudoers",
//!   "description": "Grant passwordless sudo for service management",
//!   "state": "present",
//!   "name": "deploy",
//!   "commands": ["/usr/bin/systemctl", "/usr/bin/service"],
//!   "hosts": ["ALL"],
//!   "nopasswd": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sudoers"
//! description = "Grant passwordless sudo for service management"
//! state = "present"
//! name = "deploy"
//! commands = ["/usr/bin/systemctl", "/usr/bin/service"]
//! hosts = ["ALL"]
//! nopasswd = true
//! ```
//!
//! ## Remove sudo privileges
//!
//! This example removes sudo privileges from a user.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sudoers
//!   description: "Remove sudo access from user"
//!   state: absent
//!   name: olduser
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sudoers",
//!   "description": "Remove sudo access from user",
//!   "state": "absent",
//!   "name": "olduser"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sudoers"
//! description = "Remove sudo access from user"
//! state = "absent"
//! name = "olduser"
//! ```

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{bail, Context, Result};

/// Sudoers configuration management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SudoersTask {
    /// Optional description of what this task does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Sudoers state (present/absent)
    pub state: SudoersState,

    /// User or group to grant sudo privileges
    pub name: String,

    /// Whether this is a group (prefix with %)
    #[serde(default)]
    pub group: bool,

    /// Commands to allow (defaults to ALL)
    #[serde(default = "default_all_commands")]
    pub commands: Vec<String>,

    /// Hosts to allow (defaults to ALL)
    #[serde(default = "default_all_hosts")]
    pub hosts: Vec<String>,

    /// NOPASSWD option (don't require password)
    #[serde(default)]
    pub nopasswd: bool,

    /// NOEXEC option (prevent shell escapes)
    #[serde(default)]
    pub noexec: bool,

    /// SETENV option (allow environment variable setting)
    #[serde(default)]
    pub setenv: bool,

    /// Run as user (defaults to ALL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runas: Option<String>,

    /// Path to sudoers file (defaults to /etc/sudoers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Whether to validate sudoers syntax after changes
    #[serde(default = "default_true")]
    pub validate: bool,

    /// Backup file before modification
    #[serde(default)]
    pub backup: bool,
}

/// Sudoers state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SudoersState {
    /// Ensure sudoers entry is present
    Present,
    /// Ensure sudoers entry is absent
    Absent,
}

/// Execute sudoers configuration task
pub async fn execute_sudoers_task(task: &SudoersTask, dry_run: bool) -> Result<()> {
    let sudoers_path = task.path.as_deref().unwrap_or("/etc/sudoers");

    match task.state {
        SudoersState::Present => ensure_sudoers_entry_present(task, sudoers_path, dry_run).await,
        SudoersState::Absent => ensure_sudoers_entry_absent(task, sudoers_path, dry_run).await,
    }
}

/// Ensure sudoers entry is present
async fn ensure_sudoers_entry_present(
    task: &SudoersTask,
    sudoers_path: &str,
    dry_run: bool,
) -> Result<()> {
    // Validate the entry format first
    let entry = format_sudoers_entry(task)?;

    // Read existing sudoers file
    let content = if Path::new(sudoers_path).exists() {
        fs::read_to_string(sudoers_path)
            .with_context(|| format!("Failed to read sudoers file: {}", sudoers_path))?
    } else {
        String::new()
    };

    // Check if entry already exists
    if content.lines().any(|line| line.trim() == entry.trim()) {
        return Ok(());
    }

    // Create backup if requested
    if task.backup && Path::new(sudoers_path).exists() && !dry_run {
        let backup_path = format!("{}.backup", sudoers_path);
        fs::copy(sudoers_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path))?;
    }

    // Add the entry
    let new_content = if content.is_empty() {
        entry.clone()
    } else {
        format!("{}\n{}", content.trim_end(), entry)
    };

    if dry_run {
        println!("DRY RUN: Would add sudoers entry to {}:", sudoers_path);
        println!("DRY RUN: {}", entry);
    } else {
        // Write with temporary file for safety
        let temp_path = format!("{}.tmp", sudoers_path);
        fs::write(&temp_path, &new_content)
            .with_context(|| format!("Failed to write temporary sudoers file: {}", temp_path))?;

        // Validate syntax if requested
        if task.validate {
            validate_sudoers_syntax(&temp_path)?;
        }

        // Atomic move
        fs::rename(&temp_path, sudoers_path)
            .with_context(|| format!("Failed to update sudoers file: {}", sudoers_path))?;

        // Set proper permissions (0440)
        let metadata = fs::metadata(sudoers_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o440);
        fs::set_permissions(sudoers_path, permissions)
            .with_context(|| format!("Failed to set sudoers file permissions: {}", sudoers_path))?;
    }

    Ok(())
}

/// Ensure sudoers entry is absent
async fn ensure_sudoers_entry_absent(
    task: &SudoersTask,
    sudoers_path: &str,
    dry_run: bool,
) -> Result<()> {
    if !Path::new(sudoers_path).exists() {
        return Ok(());
    }

    // Validate the entry format first
    let entry = format_sudoers_entry(task)?;

    // Read existing sudoers file
    let content = fs::read_to_string(sudoers_path)
        .with_context(|| format!("Failed to read sudoers file: {}", sudoers_path))?;

    // Check if entry exists
    let lines: Vec<&str> = content.lines().collect();
    let entry_exists = lines.iter().any(|line| line.trim() == entry.trim());

    if !entry_exists {
        return Ok(());
    }

    // Create backup if requested
    if task.backup && !dry_run {
        let backup_path = format!("{}.backup", sudoers_path);
        fs::copy(sudoers_path, &backup_path)
            .with_context(|| format!("Failed to create backup: {}", backup_path))?;
    }

    // Remove the entry
    let new_lines: Vec<&str> = lines
        .into_iter()
        .filter(|line| line.trim() != entry.trim())
        .collect();

    let new_content = new_lines.join("\n");
    let new_content = if new_content.is_empty() {
        new_content
    } else {
        new_content + "\n"
    };

    if dry_run {
        println!("DRY RUN: Would remove sudoers entry from {}:", sudoers_path);
        println!("DRY RUN: {}", entry);
    } else {
        // Write with temporary file for safety
        let temp_path = format!("{}.tmp", sudoers_path);
        fs::write(&temp_path, &new_content)
            .with_context(|| format!("Failed to write temporary sudoers file: {}", temp_path))?;

        // Validate syntax if requested
        if task.validate {
            validate_sudoers_syntax(&temp_path)?;
        }

        // Atomic move
        fs::rename(&temp_path, sudoers_path)
            .with_context(|| format!("Failed to update sudoers file: {}", sudoers_path))?;
    }

    Ok(())
}

/// Format a sudoers entry
fn format_sudoers_entry(task: &SudoersTask) -> Result<String> {
    // Validate name
    if task.name.is_empty() {
        bail!("User/group name cannot be empty");
    }

    // Validate commands
    if task.commands.is_empty() {
        bail!("Commands list cannot be empty");
    }

    // Validate hosts
    if task.hosts.is_empty() {
        bail!("Hosts list cannot be empty");
    }

    let mut parts = Vec::new();

    // User/group specification
    let user_spec = if task.group {
        format!("%{}", task.name)
    } else {
        task.name.clone()
    };
    parts.push(user_spec);

    // Hosts specification
    let hosts_spec = if task.hosts.len() == 1 && task.hosts[0] == "ALL" {
        "ALL".to_string()
    } else {
        format!("({})", task.hosts.join(","))
    };
    parts.push(hosts_spec);

    // Options
    let mut options = Vec::new();
    if task.nopasswd {
        options.push("NOPASSWD");
    }
    if task.noexec {
        options.push("NOEXEC");
    }
    if task.setenv {
        options.push("SETENV");
    }

    // Run as specification
    let runas_spec = task.runas.as_deref().unwrap_or("ALL");

    // Commands specification
    let commands_spec = if task.commands.len() == 1 && task.commands[0] == "ALL" {
        "ALL".to_string()
    } else {
        task.commands.join(", ")
    };

    // Format the final entry
    if options.is_empty() {
        Ok(format!(
            "{} {} = ({}) {}",
            parts[0], parts[1], runas_spec, commands_spec
        ))
    } else {
        Ok(format!(
            "{} {} = ({}) {}: {}",
            parts[0],
            parts[1],
            runas_spec,
            options.join(":"),
            commands_spec
        ))
    }
}

/// Validate sudoers syntax using visudo
fn validate_sudoers_syntax(file_path: &str) -> Result<()> {
    use std::process::Command;

    let output = Command::new("visudo")
        .args(["-c", "-f", file_path])
        .output()
        .context("Failed to run visudo for syntax validation")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Sudoers syntax validation failed: {}", stderr);
    }

    Ok(())
}

pub fn default_true() -> bool {
    true
}

fn default_all_commands() -> Vec<String> {
    vec!["ALL".to_string()]
}

fn default_all_hosts() -> Vec<String> {
    vec!["ALL".to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sudoers_present() {
        let temp_dir = TempDir::new().unwrap();
        let sudoers_file = temp_dir.path().join("sudoers");

        let task = SudoersTask {
            description: Some("Add sudo privileges".to_string()),
            state: SudoersState::Present,
            name: "testuser".to_string(),
            group: false,
            commands: vec!["ALL".to_string()],
            hosts: vec!["ALL".to_string()],
            nopasswd: true,
            noexec: false,
            setenv: false,
            runas: Some("root".to_string()),
            path: Some(sudoers_file.to_str().unwrap().to_string()),
            validate: false, // Skip validation in tests
            backup: false,
        };

        execute_sudoers_task(&task, false).await.unwrap();

        let content = fs::read_to_string(&sudoers_file).unwrap();
        assert!(content.contains("testuser ALL = (root) NOPASSWD: ALL"));
    }

    #[tokio::test]
    async fn test_sudoers_absent() {
        let temp_dir = TempDir::new().unwrap();
        let sudoers_file = temp_dir.path().join("sudoers");

        // Create initial file with an entry
        let initial_content = "testuser ALL = (root) NOPASSWD: ALL\n";
        fs::write(&sudoers_file, initial_content).unwrap();

        let task = SudoersTask {
            description: Some("Remove sudo privileges".to_string()),
            state: SudoersState::Absent,
            name: "testuser".to_string(),
            group: false,
            commands: vec!["ALL".to_string()],
            hosts: vec!["ALL".to_string()],
            nopasswd: true,
            noexec: false,
            setenv: false,
            runas: Some("root".to_string()),
            path: Some(sudoers_file.to_str().unwrap().to_string()),
            validate: false,
            backup: false,
        };

        execute_sudoers_task(&task, false).await.unwrap();

        let content = fs::read_to_string(&sudoers_file).unwrap();
        assert!(!content.contains("testuser"));
    }

    #[test]
    fn test_format_sudoers_entry() {
        let task = SudoersTask {
            description: None,
            state: SudoersState::Present,
            name: "testuser".to_string(),
            group: false,
            commands: vec!["ALL".to_string()],
            hosts: vec!["ALL".to_string()],
            nopasswd: true,
            noexec: false,
            setenv: false,
            runas: Some("root".to_string()),
            path: None,
            validate: true,
            backup: false,
        };

        let entry = format_sudoers_entry(&task).unwrap();
        assert_eq!(entry, "testuser ALL = (root) NOPASSWD: ALL");
    }

    #[test]
    fn test_format_sudoers_entry_group() {
        let task = SudoersTask {
            description: None,
            state: SudoersState::Present,
            name: "admin".to_string(),
            group: true,
            commands: vec!["/usr/bin/apt".to_string()],
            hosts: vec!["localhost".to_string()],
            nopasswd: false,
            noexec: true,
            setenv: false,
            runas: None,
            path: None,
            validate: true,
            backup: false,
        };

        let entry = format_sudoers_entry(&task).unwrap();
        assert_eq!(entry, "%admin (localhost) = (ALL) NOEXEC: /usr/bin/apt");
    }
}
