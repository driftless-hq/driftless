//! User task executor
//!
//! Handles user and group management operations: create, modify, delete users and groups.
//!
//! # Examples
//!
//! ## Create a user with basic settings
//!
//! This example creates a new user with a home directory and default shell.
//!
//! **YAML Format:**
//! ```yaml
//! - type: user
//!   description: "Create a web application user"
//!   name: webapp
//!   state: present
//!   uid: 1001
//!   gid: 1001
//!   home: /home/webapp
//!   shell: /bin/bash
//!   create_home: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "user",
//!   "description": "Create a web application user",
//!   "name": "webapp",
//!   "state": "present",
//!   "uid": 1001,
//!   "gid": 1001,
//!   "home": "/home/webapp",
//!   "shell": "/bin/bash",
//!   "create_home": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "user"
//! description = "Create a web application user"
//! name = "webapp"
//! state = "present"
//! uid = 1001
//! gid = 1001
//! home = "/home/webapp"
//! shell = "/bin/bash"
//! create_home = true
//! ```
//!
//! ## Create a system user
//!
//! This example creates a system user with specific UID/GID and no home directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: user
//!   description: "Create a system user for nginx"
//!   name: nginx
//!   state: present
//!   uid: 33
//!   gid: 33
//!   home: /var/lib/nginx
//!   shell: /usr/sbin/nologin
//!   create_home: false
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "user",
//!   "description": "Create a system user for nginx",
//!   "name": "nginx",
//!   "state": "present",
//!   "uid": 33,
//!   "gid": 33,
//!   "home": "/var/lib/nginx",
//!   "shell": "/usr/sbin/nologin",
//!   "create_home": false
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "user"
//! description = "Create a system user for nginx"
//! name = "nginx"
//! state = "present"
//! uid = 33
//! gid = 33
//! home = "/var/lib/nginx"
//! shell = "/usr/sbin/nologin"
//! create_home = false
//! ```
//!
//! ## Remove a user
//!
//! This example removes a user account from the system.
//!
//! **YAML Format:**
//! ```yaml
//! - type: user
//!   description: "Remove the old user account"
//!   name: olduser
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "user",
//!   "description": "Remove the old user account",
//!   "name": "olduser",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "user"
//! description = "Remove the old user account"
//! name = "olduser"
//! state = "absent"
//! ```

/// User state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserState {
    /// Ensure user exists
    Present,
    /// Ensure user does not exist
    Absent,
}

/// User and group management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Username
    pub name: String,
    /// User state
    pub state: UserState,
    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    /// Group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
    /// Home directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home: Option<String>,
    /// Shell
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Additional groups
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub groups: Vec<String>,
    /// Password (hashed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Whether to create home directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_home: Option<bool>,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Validate user task parameters
fn validate_user_task(task: &UserTask) -> Result<()> {
    // Validate username
    if task.name.is_empty() {
        return Err(anyhow::anyhow!("Username cannot be empty"));
    }

    if task.name.len() > 32 {
        return Err(anyhow::anyhow!("Username too long (max 32 characters)"));
    }

    // Username should contain only alphanumeric characters, underscore, dash, and dot
    if !task
        .name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(anyhow::anyhow!("Username contains invalid characters (only alphanumeric, underscore, dash, and dot allowed)"));
    }

    // Reserved usernames
    let reserved_names = [
        "root",
        "daemon",
        "bin",
        "sys",
        "sync",
        "games",
        "man",
        "lp",
        "mail",
        "news",
        "uucp",
        "proxy",
        "www-data",
        "backup",
        "list",
        "irc",
        "gnats",
        "nobody",
        "systemd-network",
        "systemd-resolve",
        "syslog",
        "messagebus",
        "systemd-timesync",
        "systemd-coredump",
        "_apt",
        "tss",
        "uuidd",
        "tcpdump",
        "landscape",
        "pollinate",
        "sshd",
        "systemd-oom",
    ];
    if reserved_names.contains(&task.name.as_str()) {
        // Allow reserved names if the user already exists
        if user_exists(&task.name).unwrap_or(false) {
            // User exists, allow it
        } else {
            return Err(anyhow::anyhow!("Username '{}' is reserved", task.name));
        }
    }

    // Validate UID range (if provided)
    if let Some(uid) = task.uid {
        if uid == 0 && task.name != "root" {
            return Err(anyhow::anyhow!("Only root user can have UID 0"));
        }
        if uid > 65535 {
            return Err(anyhow::anyhow!("UID must be between 0 and 65535"));
        }
    }

    // Validate GID range (if provided)
    if let Some(gid) = task.gid {
        if gid > 65535 {
            return Err(anyhow::anyhow!("GID must be between 0 and 65535"));
        }
    }

    // Validate home directory path (if provided)
    if let Some(home) = &task.home {
        if !home.starts_with('/') {
            return Err(anyhow::anyhow!("Home directory must be an absolute path"));
        }
        if home.contains("..") {
            return Err(anyhow::anyhow!("Home directory path cannot contain '..'"));
        }
    }

    // Validate shell path (if provided)
    if let Some(shell) = &task.shell {
        if !shell.starts_with('/') {
            return Err(anyhow::anyhow!("Shell must be an absolute path"));
        }
        // Check if shell exists and is executable
        if !std::path::Path::new(shell).exists() {
            return Err(anyhow::anyhow!("Shell '{}' does not exist", shell));
        }
    }

    // Validate group names
    for group in &task.groups {
        if group.is_empty() {
            return Err(anyhow::anyhow!("Group name cannot be empty"));
        }
        if group.len() > 32 {
            return Err(anyhow::anyhow!(
                "Group name '{}' too long (max 32 characters)",
                group
            ));
        }
        if !group
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(anyhow::anyhow!(
                "Group name '{}' contains invalid characters",
                group
            ));
        }
    }

    Ok(())
}

/// Execute a user task
pub async fn execute_user_task(task: &UserTask, dry_run: bool) -> Result<()> {
    // Validate task parameters
    validate_user_task(task)?;

    match task.state {
        UserState::Present => ensure_user_present(task, dry_run).await,
        UserState::Absent => ensure_user_absent(task, dry_run).await,
    }
}

/// Ensure a user exists with the correct configuration
async fn ensure_user_present(task: &UserTask, dry_run: bool) -> Result<()> {
    if user_exists(&task.name)? {
        // User exists, check if properties need updating
        update_user_if_needed(task, dry_run).await?;
    } else {
        // User doesn't exist, create it
        create_user(task, dry_run).await?;
    }

    Ok(())
}

/// Ensure a user does not exist
async fn ensure_user_absent(task: &UserTask, dry_run: bool) -> Result<()> {
    if !user_exists(&task.name)? {
        println!("User {} does not exist", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove user: {}", task.name);
    } else {
        remove_user(&task.name)?;
        println!("Removed user: {}", task.name);
    }

    Ok(())
}

/// Check if a user exists
fn user_exists(username: &str) -> Result<bool> {
    let output = Command::new("getent")
        .args(["passwd", username])
        .output()
        .with_context(|| format!("Failed to check if user {} exists", username))?;

    Ok(output.status.success())
}

/// Create a new user
async fn create_user(task: &UserTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would create user: {}", task.name);
        if let Some(uid) = task.uid {
            println!("  with UID: {}", uid);
        }
        if let Some(gid) = task.gid {
            println!("  with GID: {}", gid);
        }
        if let Some(home) = &task.home {
            println!("  with home directory: {}", home);
        }
        if let Some(shell) = &task.shell {
            println!("  with shell: {}", shell);
        }
        if let Some(create_home) = task.create_home {
            println!("  create home: {}", create_home);
        }
        if !task.groups.is_empty() {
            println!("  additional groups: {:?}", task.groups);
        }
    } else {
        // Build useradd command
        let mut cmd = vec!["useradd".to_string()];

        if let Some(uid) = task.uid {
            cmd.push("-u".to_string());
            cmd.push(uid.to_string());
        }

        if let Some(gid) = task.gid {
            cmd.push("-g".to_string());
            cmd.push(gid.to_string());
        }

        if let Some(home) = &task.home {
            cmd.push("-d".to_string());
            cmd.push(home.clone());
        }

        if let Some(shell) = &task.shell {
            cmd.push("-s".to_string());
            cmd.push(shell.clone());
        }

        if let Some(create_home) = task.create_home {
            if create_home {
                cmd.push("-m".to_string());
            } else {
                cmd.push("-M".to_string());
            }
        }

        if !task.groups.is_empty() {
            cmd.push("-G".to_string());
            cmd.push(task.groups.join(","));
        }

        cmd.push(task.name.clone());

        run_command(&cmd).with_context(|| format!("Failed to create user {}", task.name))?;

        // Set password if provided
        if let Some(password) = &task.password {
            set_user_password(&task.name, password, dry_run)?;
        }

        println!("Created user: {}", task.name);
    }

    Ok(())
}

/// Structure to hold current user properties
#[derive(Debug)]
struct UserProperties {
    uid: u32,
    gid: u32,
    home: String,
    shell: String,
}

/// Get current properties of a user
fn get_current_user_properties(username: &str) -> Result<UserProperties> {
    let output = Command::new("getent")
        .args(["passwd", username])
        .output()
        .with_context(|| format!("Failed to get user info for {}", username))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("User {} not found", username));
    }

    let passwd_line =
        String::from_utf8(output.stdout).with_context(|| "Failed to parse getent output")?;

    let fields: Vec<&str> = passwd_line.trim().split(':').collect();
    if fields.len() < 7 {
        return Err(anyhow::anyhow!("Invalid passwd entry format"));
    }

    let uid = fields[2]
        .parse::<u32>()
        .with_context(|| format!("Invalid UID in passwd entry: {}", fields[2]))?;
    let gid = fields[3]
        .parse::<u32>()
        .with_context(|| format!("Invalid GID in passwd entry: {}", fields[3]))?;
    let home = fields[5].to_string();
    let shell = fields[6].to_string();

    Ok(UserProperties {
        uid,
        gid,
        home,
        shell,
    })
}

/// Get supplementary groups for a user
fn get_user_groups(username: &str) -> Result<Vec<String>> {
    let output = Command::new("groups")
        .arg(username)
        .output()
        .with_context(|| format!("Failed to get groups for user {}", username))?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get groups for user {}",
            username
        ));
    }

    let groups_line =
        String::from_utf8(output.stdout).with_context(|| "Failed to parse groups output")?;

    // Format: username : group1 group2 group3
    let parts: Vec<&str> = groups_line.trim().split(" : ").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid groups output format"));
    }

    let groups: Vec<String> = parts[1].split_whitespace().map(|s| s.to_string()).collect();

    Ok(groups)
}

/// Update an existing user if properties differ
async fn update_user_if_needed(task: &UserTask, dry_run: bool) -> Result<()> {
    println!("User {} already exists", task.name);

    // Get current user properties
    let current_props = get_current_user_properties(&task.name)?;

    // Check each property and update if needed
    let mut needs_update = false;
    let mut update_cmd = vec!["usermod".to_string()];

    // Check UID
    if let Some(desired_uid) = task.uid {
        if current_props.uid != desired_uid {
            update_cmd.push("-u".to_string());
            update_cmd.push(desired_uid.to_string());
            needs_update = true;
        }
    }

    // Check GID
    if let Some(desired_gid) = task.gid {
        if current_props.gid != desired_gid {
            update_cmd.push("-g".to_string());
            update_cmd.push(desired_gid.to_string());
            needs_update = true;
        }
    }

    // Check home directory
    if let Some(desired_home) = &task.home {
        if current_props.home != *desired_home {
            update_cmd.push("-d".to_string());
            update_cmd.push(desired_home.clone());
            needs_update = true;
        }
    }

    // Check shell
    if let Some(desired_shell) = &task.shell {
        if current_props.shell != *desired_shell {
            update_cmd.push("-s".to_string());
            update_cmd.push(desired_shell.clone());
            needs_update = true;
        }
    }

    // Check additional groups
    if !task.groups.is_empty() {
        // Get current supplementary groups
        let current_groups = get_user_groups(&task.name)?;
        let desired_groups: std::collections::HashSet<_> = task.groups.iter().collect();
        let current_groups_set: std::collections::HashSet<_> = current_groups.iter().collect();

        if desired_groups != current_groups_set {
            update_cmd.push("-G".to_string());
            update_cmd.push(task.groups.join(","));
            needs_update = true;
        }
    }

    // Execute update if needed
    if needs_update {
        update_cmd.push(task.name.clone());

        if dry_run {
            println!(
                "Would update user {} with command: {}",
                task.name,
                update_cmd.join(" ")
            );
        } else {
            run_command(&update_cmd)
                .with_context(|| format!("Failed to update user {}", task.name))?;
            println!("Updated user: {}", task.name);
        }
    } else {
        println!("User {} properties are already correct", task.name);
    }

    // Check password separately (always update if provided, as we can't easily verify)
    if let Some(password) = &task.password {
        set_user_password(&task.name, password, dry_run)?;
    }

    Ok(())
}

/// Remove a user
fn remove_user(username: &str) -> Result<()> {
    let cmd = vec![
        "userdel".to_string(),
        "-r".to_string(),
        username.to_string(),
    ];

    run_command(&cmd).with_context(|| format!("Failed to remove user {}", username))?;

    Ok(())
}

/// Set a user's password
fn set_user_password(username: &str, password: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Would set password for user: {}", username);
    } else {
        // Note: This is a simplified implementation
        // In a real system, you'd need to handle password hashing properly
        // and might use tools like chpasswd

        let password_line = format!("{}:{}", username, password);

        let mut passwd_cmd = Command::new("chpasswd")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .with_context(|| "Failed to spawn chpasswd command")?;

        if let Some(stdin) = passwd_cmd.stdin.as_mut() {
            use std::io::Write;
            stdin
                .write_all(password_line.as_bytes())
                .with_context(|| "Failed to write to chpasswd stdin")?;
        }

        let status = passwd_cmd
            .wait()
            .with_context(|| "Failed to wait for chpasswd command")?;

        if !status.success() {
            return Err(anyhow::anyhow!("chpasswd command failed"));
        }

        println!("Set password for user: {}", username);
    }

    Ok(())
}

/// Run a command and return the result
fn run_command(cmd: &[String]) -> Result<()> {
    if cmd.is_empty() {
        return Ok(());
    }

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .status()
        .with_context(|| format!("Failed to execute command: {}", cmd.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_create_dry_run() {
        let task = UserTask {
            description: None,
            name: "testuser".to_string(),
            state: UserState::Present,
            uid: Some(1001),
            gid: Some(1001),
            home: Some("/home/testuser".to_string()),
            shell: Some("/bin/bash".to_string()),
            groups: vec!["wheel".to_string()],
            password: Some("testpass".to_string()),
            create_home: Some(true),
        };

        let result = execute_user_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_user_remove_dry_run() {
        let task = UserTask {
            description: None,
            name: "testuser".to_string(),
            state: UserState::Absent,
            uid: None,
            gid: None,
            home: None,
            shell: None,
            groups: vec![],
            password: None,
            create_home: None,
        };

        let result = execute_user_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_user_exists() {
        // Test with root user (should exist on most systems)
        let exists = user_exists("root");
        // We can't assert much here since the test environment might not have standard users
        assert!(exists.is_ok());
    }
}
