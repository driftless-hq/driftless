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

/// Execute a user task
pub async fn execute_user_task(task: &UserTask, dry_run: bool) -> Result<()> {
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

/// Update an existing user if properties differ
async fn update_user_if_needed(task: &UserTask, dry_run: bool) -> Result<()> {
    // This is a simplified implementation
    // In a real system, you'd compare current user properties with desired state
    // and make appropriate changes using usermod

    println!("User {} already exists", task.name);

    // Check if password needs updating
    if let Some(password) = &task.password {
        // Check if password is different (simplified check)
        if needs_password_update(&task.name, password)? {
            set_user_password(&task.name, password, dry_run)?;
        }
    }

    // TODO: Check other properties like UID, GID, home, shell, groups
    // and update as needed using usermod

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

/// Check if a user's password needs updating
fn needs_password_update(_username: &str, _new_password: &str) -> Result<bool> {
    // Simplified implementation - always assume password needs updating
    // In a real system, you'd compare hashed passwords or check modification time
    Ok(true)
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
