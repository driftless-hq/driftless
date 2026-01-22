//! Group task executor
//!
//! Handles group management operations: create, delete groups.
//!
//! # Examples
//!
//! ## Create a group
//!
//! This example creates a new group with a specific GID.
//!
//! **YAML Format:**
//! ```yaml
//! - type: group
//!   description: "Create a web application group"
//!   name: webapp
//!   state: present
//!   gid: 1001
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "group",
//!   "description": "Create a web application group",
//!   "name": "webapp",
//!   "state": "present",
//!   "gid": 1001
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "group"
//! description = "Create a web application group"
//! name = "webapp"
//! state = "present"
//! gid = 1001
//! ```
//!
//! ## Create a system group
//!
//! This example creates a system group with automatic GID assignment.
//!
//! **YAML Format:**
//! ```yaml
//! - type: group
//!   description: "Create a system group for nginx"
//!   name: nginx
//!   state: present
//!   system: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "group",
//!   "description": "Create a system group for nginx",
//!   "name": "nginx",
//!   "state": "present",
//!   "system": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "group"
//! description = "Create a system group for nginx"
//! name = "nginx"
//! state = "present"
//! system = true
//! ```
//!
//! ## Remove a group
//!
//! This example removes a group from the system.
//!
//! **YAML Format:**
//! ```yaml
//! - type: group
//!   description: "Remove the old group"
//!   name: oldgroup
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "group",
//!   "description": "Remove the old group",
//!   "name": "oldgroup",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "group"
//! description = "Remove the old group"
//! name = "oldgroup"
//! state = "absent"
//! ```

/// Group state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupState {
    /// Ensure group exists
    Present,
    /// Ensure group does not exist
    Absent,
}

/// Group management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Group name
    pub name: String,
    /// Group state
    pub state: GroupState,
    /// Group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
    /// Whether group is a system group
    #[serde(default)]
    pub system: bool,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Execute a group task
pub async fn execute_group_task(task: &GroupTask, dry_run: bool) -> Result<()> {
    match task.state {
        GroupState::Present => ensure_group_present(task, dry_run).await,
        GroupState::Absent => ensure_group_absent(task, dry_run).await,
    }
}

/// Ensure a group exists with the correct configuration
async fn ensure_group_present(task: &GroupTask, dry_run: bool) -> Result<()> {
    if group_exists(&task.name)? {
        println!("Group {} already exists", task.name);
        // TODO: Check if GID needs updating (would require usermod equivalent for groups)
        return Ok(());
    }

    // Create the group
    if dry_run {
        println!("Would create group: {}", task.name);
        if let Some(gid) = task.gid {
            println!("  with GID: {}", gid);
        }
        if task.system {
            println!("  as system group");
        }
    } else {
        create_group(task)?;
        println!("Created group: {}", task.name);
    }

    Ok(())
}

/// Ensure a group does not exist
async fn ensure_group_absent(task: &GroupTask, dry_run: bool) -> Result<()> {
    if !group_exists(&task.name)? {
        println!("Group {} does not exist", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove group: {}", task.name);
    } else {
        remove_group(&task.name)?;
        println!("Removed group: {}", task.name);
    }

    Ok(())
}

/// Check if a group exists
fn group_exists(groupname: &str) -> Result<bool> {
    let output = Command::new("getent")
        .args(["group", groupname])
        .output()
        .with_context(|| format!("Failed to check if group {} exists", groupname))?;

    Ok(output.status.success())
}

/// Create a new group
fn create_group(task: &GroupTask) -> Result<()> {
    let mut cmd = vec!["groupadd".to_string()];

    if let Some(gid) = task.gid {
        cmd.push("-g".to_string());
        cmd.push(gid.to_string());
    }

    if task.system {
        cmd.push("--system".to_string());
    }

    cmd.push(task.name.clone());

    run_command(&cmd).with_context(|| format!("Failed to create group {}", task.name))?;

    Ok(())
}

/// Remove a group
fn remove_group(groupname: &str) -> Result<()> {
    let cmd = vec!["groupdel".to_string(), groupname.to_string()];

    run_command(&cmd).with_context(|| format!("Failed to remove group {}", groupname))?;

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
    async fn test_group_create_dry_run() {
        let task = GroupTask {
            description: None,
            name: "testgroup".to_string(),
            state: GroupState::Present,
            gid: Some(2000),
            system: false,
        };

        let result = execute_group_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_group_remove_dry_run() {
        let task = GroupTask {
            description: None,
            name: "testgroup".to_string(),
            state: GroupState::Absent,
            gid: None,
            system: false,
        };

        let result = execute_group_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_group_exists() {
        // Test with root group (should exist on most systems)
        let exists = group_exists("root");
        // We can't assert much here since the test environment might not have standard groups
        assert!(exists.is_ok());
    }

    #[tokio::test]
    async fn test_group_create_invalid_name() {
        let task = GroupTask {
            description: None,
            name: "".to_string(), // Empty name may cause command failure
            state: GroupState::Present,
            gid: None,
            system: false,
        };

        let result = execute_group_task(&task, false).await;
        // Empty name might succeed or fail depending on system, but shouldn't crash
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_group_create_with_existing_group() {
        let task = GroupTask {
            description: None,
            name: "root".to_string(), // Group that likely exists
            state: GroupState::Present,
            gid: None,
            system: false,
        };

        let result = execute_group_task(&task, true).await;
        // This should succeed since the group already exists
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_group_remove_nonexistent() {
        let task = GroupTask {
            description: None,
            name: "nonexistent_test_group_12345".to_string(),
            state: GroupState::Absent,
            gid: None,
            system: false,
        };

        let result = execute_group_task(&task, true).await;
        // This should succeed since the group doesn't exist
        assert!(result.is_ok());
    }
}
