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
//! ## Register command output
//!
//! This example runs a command and registers its output for use in subsequent tasks.
//!
//! **YAML Format:**
//! ```yaml
//! - type: command
//!   description: "Check system uptime"
//!   command: uptime
//!   register: uptime_result
//!
//! - type: debug
//!   msg: "The system uptime is: {{ uptime_result.stdout }}"
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "command",
//!     "description": "Check system uptime",
//!     "command": "uptime",
//!     "register": "uptime_result"
//!   },
//!   {
//!     "type": "debug",
//!     "msg": "The system uptime is: {{ uptime_result.stdout }}"
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "command"
//! description = "Check system uptime"
//! command = "uptime"
//! register = "uptime_result"
//!
//! [[tasks]]
//! type = "debug"
//! msg = "The system uptime is: {{ uptime_result.stdout }}"
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
///
/// # Registered Outputs
/// - `stdout` (String): The standard output of the command
/// - `stderr` (String): The standard error of the command
/// - `rc` (i32): The exit code of the command
/// - `changed` (bool): Whether the command was actually run
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
    /// Whether to stream output in real-time (useful for long-running commands)
    #[serde(default)]
    pub stream_output: bool,
}

use anyhow::{Context, Result};
use chrono;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::apply::executor::TaskExecutor;

/// Execute a command task
pub async fn execute_command_task(task: &CommandTask, executor: &TaskExecutor) -> Result<serde_yaml::Value> {
    // Check if command should be run idempotently
    if task.idempotent && is_command_already_run(task, executor)? {
        println!("Command already executed (idempotent): {}", task.command);
        let mut result = serde_yaml::Mapping::new();
        result.insert(
            serde_yaml::Value::String("changed".to_string()),
            serde_yaml::Value::Bool(false),
        );
        result.insert(
            serde_yaml::Value::String("skipped".to_string()),
            serde_yaml::Value::Bool(true),
        );
        return Ok(serde_yaml::Value::Mapping(result));
    }

    if executor.dry_run() {
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
        let mut result = serde_yaml::Mapping::new();
        result.insert(
            serde_yaml::Value::String("changed".to_string()),
            serde_yaml::Value::Bool(false),
        );
        result.insert(
            serde_yaml::Value::String("dry_run".to_string()),
            serde_yaml::Value::Bool(true),
        );
        Ok(serde_yaml::Value::Mapping(result))
    } else {
        let output = run_command(task).await?;
        println!("Executed command: {}", task.command);

        // Mark command as run for idempotency
        if task.idempotent {
            mark_command_as_run(task, executor)?;
        }

        Ok(output)
    }
}

/// Run the actual command
async fn run_command(task: &CommandTask) -> Result<serde_yaml::Value> {
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

    if task.stream_output {
        // Stream output in real-time
        run_command_streaming(task, cmd).await
    } else {
        // Buffer output (original behavior)
        run_command_buffered(task, cmd).await
    }
}

/// Run command with buffered output (original behavior)
async fn run_command_buffered(task: &CommandTask, mut cmd: Command) -> Result<serde_yaml::Value> {
    // Set up I/O - capture output
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    // Execute the command
    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute command: {}", task.command))?;

    // Check exit code
    let exit_code = output.status.code().unwrap_or(-1);
    if exit_code != task.exit_code {
        return Err(anyhow::anyhow!(
            "Command exited with code {} (expected {}): {}",
            exit_code,
            task.exit_code,
            task.command
        ));
    }

    // Prepare result
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let mut result = serde_yaml::Mapping::new();
    result.insert(
        serde_yaml::Value::String("stdout".to_string()),
        serde_yaml::Value::String(stdout),
    );
    result.insert(
        serde_yaml::Value::String("stderr".to_string()),
        serde_yaml::Value::String(stderr),
    );
    result.insert(
        serde_yaml::Value::String("rc".to_string()),
        serde_yaml::Value::Number(exit_code.into()),
    );
    result.insert(
        serde_yaml::Value::String("changed".to_string()),
        serde_yaml::Value::Bool(true),
    );

    Ok(serde_yaml::Value::Mapping(result))
}

/// Run command with streaming output
async fn run_command_streaming(task: &CommandTask, cmd: Command) -> Result<serde_yaml::Value> {
    use tokio::io::{AsyncBufReadExt, BufReader};

    // Convert to tokio command for async I/O
    let mut cmd = tokio::process::Command::from(cmd);

    // Set up I/O for streaming
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    // Spawn the command
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn command: {}", task.command))?;

    // Get handles to stdout and stderr
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Create buffered readers
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    // Collect output for result
    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    // Stream output in real-time
    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        println!("STDOUT: {}", line);
                        stdout_lines.push(line);
                    }
                    Ok(None) => break, // EOF
                    Err(e) => {
                        eprintln!("Error reading stdout: {}", e);
                        break;
                    }
                }
            }
            line = stderr_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        eprintln!("STDERR: {}", line);
                        stderr_lines.push(line);
                    }
                    Ok(None) => break, // EOF
                    Err(e) => {
                        eprintln!("Error reading stderr: {}", e);
                        break;
                    }
                }
            }
        }
    }

    // Wait for the command to complete
    let status = child
        .wait()
        .await
        .with_context(|| format!("Failed to wait for command: {}", task.command))?;

    let exit_code = status.code().unwrap_or(-1);
    if exit_code != task.exit_code {
        return Err(anyhow::anyhow!(
            "Command exited with code {} (expected {}): {}",
            exit_code,
            task.exit_code,
            task.command
        ));
    }

    // Prepare result
    let stdout = stdout_lines.join("\n");
    let stderr = stderr_lines.join("\n");

    let mut result = serde_yaml::Mapping::new();
    result.insert(
        serde_yaml::Value::String("stdout".to_string()),
        serde_yaml::Value::String(stdout),
    );
    result.insert(
        serde_yaml::Value::String("stderr".to_string()),
        serde_yaml::Value::String(stderr),
    );
    result.insert(
        serde_yaml::Value::String("rc".to_string()),
        serde_yaml::Value::Number(exit_code.into()),
    );
    result.insert(
        serde_yaml::Value::String("changed".to_string()),
        serde_yaml::Value::Bool(true),
    );
    result.insert(
        serde_yaml::Value::String("streamed".to_string()),
        serde_yaml::Value::Bool(true),
    );

    Ok(serde_yaml::Value::Mapping(result))
}

/// Parse a command string into program and arguments
fn parse_command(command: &str) -> Result<(String, Vec<String>)> {
    // Use shlex for proper shell-like parsing that handles quotes, escapes, etc.
    let parts: Vec<String> = shlex::split(command).ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to parse command (unmatched quotes or invalid syntax): {}",
            command
        )
    })?;

    if parts.is_empty() {
        return Err(anyhow::anyhow!("Empty command"));
    }

    // Validate that the command doesn't contain shell metacharacters that could be dangerous
    // This is a basic check - in a production system you'd want more sophisticated validation
    let dangerous_chars = [';', '&', '|', '<', '>', '`', '$', '(', ')'];
    for &ch in dangerous_chars.iter() {
        if command.contains(ch) {
            return Err(anyhow::anyhow!(
                "Command contains potentially dangerous shell metacharacter '{}'. Use explicit arguments instead: {}",
                ch, command
            ));
        }
    }

    let program = parts[0].clone();
    let args = parts[1..].to_vec();

    Ok((program, args))
}

/// Check if an idempotent command has already been run
fn is_command_already_run(task: &CommandTask, executor: &TaskExecutor) -> Result<bool> {
    let state_file = get_command_state_file(task, executor);
    if !state_file.exists() {
        return Ok(false);
    }

    // Read and parse the state file
    let content = fs::read_to_string(&state_file)
        .with_context(|| format!("Failed to read state file: {:?}", state_file))?;

    let state: CommandState = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse state file: {:?}", state_file))?;

    // Check if the command state matches
    Ok(state.matches(task))
}

/// Mark a command as having been run
fn mark_command_as_run(task: &CommandTask, executor: &TaskExecutor) -> Result<()> {
    let state_file = get_command_state_file(task, executor);

    // Create state directory if it doesn't exist
    if let Some(parent) = state_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create state directory: {:?}", parent))?;
    }

    // Create state object
    let state = CommandState::from_task(task);

    // Write state file
    let content = serde_json::to_string_pretty(&state)
        .with_context(|| "Failed to serialize command state")?;

    fs::write(&state_file, content)
        .with_context(|| format!("Failed to write state file: {:?}", state_file))?;

    Ok(())
}

/// Get the state file path for a command
fn get_command_state_file(task: &CommandTask, executor: &TaskExecutor) -> PathBuf {
    // Use the configured state directory from the executor config
    let state_dir = &executor.config().state_dir;

    let hash = hash_command(task);
    Path::new(&state_dir)
        .join("commands")
        .join(format!("{}.json", hash))
}

/// Command execution state for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandState {
    /// Command that was executed
    command: String,
    /// Working directory
    cwd: Option<String>,
    /// Environment variables (sorted for consistent hashing)
    env: Vec<(String, String)>,
    /// Hash of the command for verification
    hash: String,
    /// Timestamp when command was executed
    executed_at: chrono::DateTime<chrono::Utc>,
}

impl CommandState {
    /// Create a new command state from a task
    fn from_task(task: &CommandTask) -> Self {
        let mut env: Vec<_> = task
            .env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        env.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            command: task.command.clone(),
            cwd: task.cwd.clone(),
            env,
            hash: hash_command(task),
            executed_at: chrono::Utc::now(),
        }
    }

    /// Check if this state matches a task
    fn matches(&self, task: &CommandTask) -> bool {
        if self.command != task.command || self.cwd != task.cwd {
            return false;
        }

        // Check environment variables
        let mut task_env: Vec<_> = task
            .env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        task_env.sort_by(|a, b| a.0.cmp(&b.0));

        self.env == task_env
    }
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
            stream_output: false,
        };

        let executor = TaskExecutor::new(true);
        let result = execute_command_task(&task, &executor).await;
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
            stream_output: false,
        };

        let executor = TaskExecutor::new(false);
        let result = execute_command_task(&task, &executor).await;
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
            stream_output: false,
        };

        let executor = TaskExecutor::new(false);
        let result = execute_command_task(&task, &executor).await;
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
            stream_output: false,
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
            stream_output: false,
        };

        let hash1 = hash_command(&task1);
        let hash2 = hash_command(&task2);

        assert_ne!(hash1, hash2); // Different working directory should produce different hash
    }
}
