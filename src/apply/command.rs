//! Command task executor
//!
//! Handles execution of shell commands with proper environment and working directory support.
//!
//! # Examples
//!
//! ## Run a simple command
//!
//! This example runs a basic shell command.
//!
//! **YAML Format:**
//! ```yaml
//! - type: command
//!   description: "Update package list"
//!   command: apt-get update
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "command",
//!   "description": "Update package list",
//!   "command": "apt-get update"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "command"
//! description = "Update package list"
//! command = "apt-get update"
//! ```
//!
//! ## Run command with specific working directory
//!
//! This example runs a command in a specific directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: command
//!   description: "Build application in project directory"
//!   command: make build
//!   cwd: /opt/myapp
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "command",
//!   "description": "Build application in project directory",
//!   "command": "make build",
//!   "cwd": "/opt/myapp"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "command"
//! description = "Build application in project directory"
//! command = "make build"
//! cwd = "/opt/myapp"
//! ```
//!
//! ## Run command as specific user
//!
//! This example runs a command as a specific user.
//!
//! **YAML Format:**
//! ```yaml
//! - type: command
//!   description: "Restart nginx service"
//!   command: systemctl restart nginx
//!   user: root
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "command",
//!   "description": "Restart nginx service",
//!   "command": "systemctl restart nginx",
//!   "user": "root"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "command"
//! description = "Restart nginx service"
//! command = "systemctl restart nginx"
//! user = "root"
//! ```
//!
//! ## Idempotent command
//!
//! This example runs a command only if it hasn't been run before.
//!
//! **YAML Format:**
//! ```yaml
//! - type: command
//!   description: "Initialize database (idempotent)"
//!   command: /opt/myapp/init-db.sh
//!   idempotent: true
//!   exit_code: 0
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "command",
//!   "description": "Initialize database (idempotent)",
//!   "command": "/opt/myapp/init-db.sh",
//!   "idempotent": true,
//!   "exit_code": 0
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "command"
//! description = "Initialize database (idempotent)"
//! command = "/opt/myapp/init-db.sh"
//! idempotent = true
//! exit_code = 0
//! ```

/// Command execution task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CommandTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Command to execute
    pub command: String,
    /// Working directory for command execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub env: std::collections::HashMap<String, String>,
    /// Whether command should be idempotent (only run if not already applied)
    #[serde(default)]
    pub idempotent: bool,
    /// Expected exit code (default: 0)
    #[serde(default = "default_exit_code")]
    pub exit_code: i32,
    /// Whether to run command as a specific user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Whether to run command as a specific group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

use anyhow::{Context, Result};
use std::process::{Command, Stdio};

/// Execute a command task
pub async fn execute_command_task(task: &CommandTask, dry_run: bool) -> Result<()> {
    // Check if command should be run idempotently
    if task.idempotent && is_command_already_run(task)? {
        println!("Command already executed (idempotent): {}", task.command);
        return Ok(());
    }

    if dry_run {
        println!("Would run command: {}", task.command);
        if let Some(cwd) = &task.cwd {
            println!("  in directory: {}", cwd);
        }
        if !task.env.is_empty() {
            println!("  with environment variables: {:?}", task.env);
        }
        if task.user.is_some() || task.group.is_some() {
            println!("  as user: {:?}, group: {:?}", task.user, task.group);
        }
    } else {
        run_command(task).await?;
        println!("Executed command: {}", task.command);

        // Mark command as run for idempotency
        if task.idempotent {
            mark_command_as_run(task)?;
        }
    }

    Ok(())
}

/// Run the actual command
async fn run_command(task: &CommandTask) -> Result<()> {
    // Parse the command string into program and arguments
    let (program, args) = parse_command(&task.command)?;

    // Build the command
    let mut cmd = Command::new(program);
    cmd.args(args);

    // Set working directory if specified
    if let Some(cwd) = &task.cwd {
        cmd.current_dir(cwd);
    }

    // Set environment variables
    for (key, value) in &task.env {
        cmd.env(key, value);
    }

    // Configure user/group if specified (simplified - would need privilege escalation in real impl)
    if task.user.is_some() || task.group.is_some() {
        println!("Note: User/group execution not fully implemented yet");
        println!(
            "Would run as user: {:?}, group: {:?}",
            task.user, task.group
        );
    }

    // Set up I/O - inherit stdin/stdout/stderr for interactive commands
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Execute the command
    let status = cmd
        .status()
        .with_context(|| format!("Failed to execute command: {}", task.command))?;

    // Check exit code
    let exit_code = status.code().unwrap_or(-1);
    if exit_code != task.exit_code {
        return Err(anyhow::anyhow!(
            "Command exited with code {} (expected {}): {}",
            exit_code,
            task.exit_code,
            task.command
        ));
    }

    Ok(())
}

/// Parse a command string into program and arguments
fn parse_command(command: &str) -> Result<(String, Vec<String>)> {
    // Simple shell-like parsing - split on spaces for now
    // In a real implementation, you'd want proper shell parsing
    let parts: Vec<String> = shlex::split(command)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse command: {}", command))?;

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    let program = parts[0].clone();
    let args = parts[1..].to_vec();

    Ok((program, args))
}

/// Check if an idempotent command has already been run
fn is_command_already_run(task: &CommandTask) -> Result<bool> {
    // Simple implementation: check for a marker file
    // In a real implementation, you'd use a proper state store
    let marker_path = format!("/tmp/driftless_cmd_{}", hash_command(task));
    Ok(std::path::Path::new(&marker_path).exists())
}

/// Mark a command as having been run
fn mark_command_as_run(task: &CommandTask) -> Result<()> {
    let marker_path = format!("/tmp/driftless_cmd_{}", hash_command(task));
    std::fs::write(&marker_path, "")
        .with_context(|| format!("Failed to create marker file: {}", marker_path))?;
    Ok(())
}

/// Generate a hash of the command for idempotency tracking
fn hash_command(task: &CommandTask) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    task.command.hash(&mut hasher);
    task.cwd.hash(&mut hasher);

    // Hash environment variables individually since HashMap doesn't implement Hash
    for (key, value) in &task.env {
        key.hash(&mut hasher);
        value.hash(&mut hasher);
    }

    format!("{:x}", hasher.finish())
}

/// Default exit code (0)
pub fn default_exit_code() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_command_execution_dry_run() {
        let task = CommandTask {
            description: None,
            command: "echo hello".to_string(),
            cwd: None,
            env: HashMap::new(),
            idempotent: false,
            exit_code: 0,
            user: None,
            group: None,
        };

        let result = execute_command_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_execution_real() {
        let task = CommandTask {
            description: None,
            command: "echo hello".to_string(),
            cwd: None,
            env: HashMap::new(),
            idempotent: false,
            exit_code: 0,
            user: None,
            group: None,
        };

        let result = execute_command_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_with_wrong_exit_code() {
        let task = CommandTask {
            description: None,
            command: "false".to_string(), // Command that exits with 1
            cwd: None,
            env: HashMap::new(),
            idempotent: false,
            exit_code: 0, // But we expect 0
            user: None,
            group: None,
        };

        let result = execute_command_task(&task, false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exited with code 1"));
    }

    #[test]
    fn test_parse_command() {
        let (program, args) = parse_command("ls -la /tmp").unwrap();
        assert_eq!(program, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);

        let (program, args) = parse_command("echo 'hello world'").unwrap();
        assert_eq!(program, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_hash_command() {
        let task1 = CommandTask {
            description: None,
            command: "echo hello".to_string(),
            cwd: None,
            env: HashMap::new(),
            idempotent: false,
            exit_code: 0,
            user: None,
            group: None,
        };

        let task2 = CommandTask {
            description: None,
            command: "echo hello".to_string(),
            cwd: Some("/tmp".to_string()),
            env: HashMap::new(),
            idempotent: false,
            exit_code: 0,
            user: None,
            group: None,
        };

        let hash1 = hash_command(&task1);
        let hash2 = hash_command(&task2);

        assert_ne!(hash1, hash2); // Different working directory should produce different hash
    }
}
