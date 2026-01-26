//! Execute commands without shell processing task executor
//!
//! Handles execution of commands directly without shell interpretation.
//!
//! # Examples
//!
//! ## Execute a simple command
//!
//! This example executes the `ls` command.
//!
//! **YAML Format:**
//! ```yaml
//! - type: raw
//!   description: "List directory contents"
//!   executable: ls
//!   args: ["-la", "/tmp"]
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "raw",
//!   "description": "List directory contents",
//!   "executable": "ls",
//!   "args": ["-la", "/tmp"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "raw"
//! description = "List directory contents"
//! executable = "ls"
//! args = ["-la", "/tmp"]
//! ```
//!
//! ## Execute command with environment variables
//!
//! This example executes a command with custom environment variables.
//!
//! **YAML Format:**
//! ```yaml
//! - type: raw
//!   description: "Run command with environment"
//!   executable: /usr/local/bin/myapp
//!   args: ["--config", "/etc/myapp/config.json"]
//!   environment:
//!     DATABASE_URL: "postgresql://localhost/mydb"
//!     LOG_LEVEL: "debug"
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "raw",
//!   "description": "Run command with environment",
//!   "executable": "/usr/local/bin/myapp",
//!   "args": ["--config", "/etc/myapp/config.json"],
//!   "environment": {
//!     "DATABASE_URL": "postgresql://localhost/mydb",
//!     "LOG_LEVEL": "debug"
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "raw"
//! description = "Run command with environment"
//! executable = "/usr/local/bin/myapp"
//! args = ["--config", "/etc/myapp/config.json"]
//! environment = { DATABASE_URL = "postgresql://localhost/mydb", LOG_LEVEL = "debug" }
//! ```
//!
//! ## Execute command with timeout
//!
//! This example executes a command with a timeout.
//!
//! **YAML Format:**
//! ```yaml
//! - type: raw
//!   description: "Run command with timeout"
//!   executable: sleep
//!   args: ["30"]
//!   timeout: 10
//!   ignore_errors: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "raw",
//!   "description": "Run command with timeout",
//!   "executable": "sleep",
//!   "args": ["30"],
//!   "timeout": 10,
//!   "ignore_errors": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "raw"
//! description = "Run command with timeout"
//! executable = "sleep"
//! args = ["30"]
//! timeout = 10
//! ignore_errors = true
//! ```
//!
//! ## Execute command in specific directory
//!
//! This example executes a command in a specific working directory.
//!
//! **YAML Format:**
//! ```yaml
//! - type: raw
//!   description: "Run command in project directory"
//!   executable: make
//!   args: ["build"]
//!   chdir: /opt/myproject
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "raw",
//!   "description": "Run command in project directory",
//!   "executable": "make",
//!   "args": ["build"],
//!   "chdir": "/opt/myproject"
//! }
//! ## Execute command with creates/removes checks
//!
//! This example executes a command that creates specific files and checks for their existence.
//!
//! **YAML Format:**
//! ```yaml
//! - type: raw
//!   description: "Create configuration file"
//!   executable: touch
//!   args: ["/etc/myapp/config.conf"]
//!   creates: true
//!   creates_files: ["/etc/myapp/config.conf"]
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "raw",
//!   "description": "Create configuration file",
//!   "executable": "touch",
//!   "args": ["/etc/myapp/config.conf"],
//!   "creates": true,
//!   "creates_files": ["/etc/myapp/config.conf"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "raw"
//! description = "Create configuration file"
//! executable = "touch"
//! args = ["/etc/myapp/config.conf"]
//! creates = true
//! creates_files = ["/etc/myapp/config.conf"]
//! ```

/// Execute commands without shell processing task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Command to execute (argv\[0\])
    pub executable: String,
    /// Command arguments (argv[1..])
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for command execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chdir: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Execution timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    /// Expected exit codes (defaults to \[0\])
    #[serde(default = "default_exit_codes")]
    pub exit_codes: Vec<i32>,
    /// Whether to ignore errors
    #[serde(default)]
    pub ignore_errors: bool,
    /// Whether the command creates resources
    #[serde(default)]
    pub creates: bool,
    /// Whether the command removes resources
    #[serde(default)]
    pub removes: bool,
    /// Files/directories created by the command (for creates check)
    #[serde(default)]
    pub creates_files: Vec<String>,
    /// Files/directories removed by the command (for removes check)
    #[serde(default)]
    pub removes_files: Vec<String>,
    /// Force command execution
    #[serde(default)]
    pub force: bool,
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Default exit codes (\[0\])
pub fn default_exit_codes() -> Vec<i32> {
    vec![0]
}
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Execute a raw task
pub async fn execute_raw_task(task: &RawTask, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "Would execute raw command: {} with args: {:?}",
            task.executable, task.args
        );
        if let Some(ref chdir) = task.chdir {
            println!("  (in directory: {})", chdir);
        }
        if !task.environment.is_empty() {
            println!(
                "  (with environment variables: {} vars)",
                task.environment.len()
            );
        }
        if let Some(timeout) = task.timeout {
            println!("  (with timeout: {}s)", timeout);
        }
        return Ok(());
    }

    // Check if command creates/removes resources
    if task.creates && !dry_run && !task.force && !task.creates_files.is_empty() {
        // Check if any of the files/directories that should be created already exist
        for file_path in &task.creates_files {
            let path = Path::new(file_path);
            if path.exists() {
                println!(
                    "Command creates resources that already exist, skipping: {}",
                    file_path
                );
                return Ok(());
            }
        }
    }

    if task.removes && !dry_run && !task.force && !task.removes_files.is_empty() {
        // Check if any of the files/directories that should be removed don't exist
        let mut all_exist = true;
        for file_path in &task.removes_files {
            let path = Path::new(file_path);
            if !path.exists() {
                all_exist = false;
                break;
            }
        }
        if !all_exist {
            println!("Command removes resources that don't exist, skipping");
            return Ok(());
        }
    }

    // Validate executable exists and is executable
    let executable_path = Path::new(&task.executable);
    if executable_path.is_absolute() || task.executable.contains('/') {
        // For absolute paths or paths with separators, check if the file exists
        if !executable_path.exists() {
            return Err(anyhow::anyhow!(
                "Executable does not exist: {}",
                task.executable
            ));
        }

        // Check if executable is actually executable
        use std::os::unix::fs::PermissionsExt;
        let metadata = executable_path.metadata().with_context(|| {
            format!("Failed to get metadata for executable: {}", task.executable)
        })?;

        if (metadata.permissions().mode() & 0o111) == 0 {
            return Err(anyhow::anyhow!(
                "Executable is not executable: {}",
                task.executable
            ));
        }
    } else {
        // For commands without path separators, assume they're in PATH
        // The actual execution will fail if they're not found
        println!(
            "Warning: Command '{}' specified without full path - assuming it's in PATH",
            task.executable
        );
    }

    // Validate working directory if specified
    if let Some(ref chdir) = task.chdir {
        let chdir_path = Path::new(chdir);
        if !chdir_path.exists() {
            return Err(anyhow::anyhow!(
                "Working directory does not exist: {}",
                chdir
            ));
        }
        if !chdir_path.is_dir() {
            return Err(anyhow::anyhow!(
                "Working directory is not a directory: {}",
                chdir
            ));
        }
    }

    // Execute the command directly (no shell processing)
    let mut command = Command::new(&task.executable);
    command.args(&task.args);

    // Set working directory if specified
    if let Some(ref chdir) = task.chdir {
        command.current_dir(chdir);
    }

    // Set environment variables
    for (key, value) in &task.environment {
        command.env(key, value);
    }

    // Configure stdio
    command.stdin(Stdio::null());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    // Set timeout if specified
    let timeout_duration = task.timeout.map(|t| Duration::from_secs(t as u64));

    println!("Executing raw command: {} {:?}", task.executable, task.args);

    let output = if let Some(timeout) = timeout_duration {
        // Execute with timeout
        tokio::time::timeout(timeout, tokio::process::Command::from(command).output())
            .await
            .with_context(|| {
                format!(
                    "Command timed out after {}s: {} {:?}",
                    timeout.as_secs(),
                    task.executable,
                    task.args
                )
            })?
            .with_context(|| {
                format!(
                    "Failed to execute command: {} {:?}",
                    task.executable, task.args
                )
            })?
    } else {
        // Execute without timeout
        tokio::process::Command::from(command)
            .output()
            .await
            .with_context(|| {
                format!(
                    "Failed to execute command: {} {:?}",
                    task.executable, task.args
                )
            })?
    };

    // Check exit code
    let exit_code = output.status.code().unwrap_or(-1);
    let success = task.exit_codes.contains(&exit_code);

    if success {
        println!(
            "Raw command executed successfully (exit code: {})",
            exit_code
        );

        // Print stdout if any
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            println!("stdout: {}", stdout.trim());
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        if task.ignore_errors {
            println!(
                "Raw command failed (exit code: {}) but errors ignored",
                exit_code
            );
            if !stdout.trim().is_empty() {
                println!("stdout: {}", stdout.trim());
            }
            if !stderr.trim().is_empty() {
                println!("stderr: {}", stderr.trim());
            }
        } else {
            return Err(anyhow::anyhow!(
                "Raw command failed with exit code {}: {} {:?}\nstdout: {}\nstderr: {}",
                exit_code,
                task.executable,
                task.args,
                stdout,
                stderr
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_raw_command_dry_run() {
        let mut environment = HashMap::new();
        environment.insert("TEST_VAR".to_string(), "test_value".to_string());

        let task = RawTask {
            description: None,
            executable: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            chdir: Some("/tmp".to_string()),
            environment,
            timeout: Some(30),
            exit_codes: vec![0],
            ignore_errors: false,
            creates: false,
            removes: false,
            creates_files: vec![],
            removes_files: vec![],
            force: false,
        };

        let result = execute_raw_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_raw_command_success() {
        let task = RawTask {
            description: None,
            executable: "echo".to_string(),
            args: vec!["test".to_string()],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            exit_codes: vec![0],
            ignore_errors: false,
            creates: false,
            removes: false,
            creates_files: vec![],
            removes_files: vec![],
            force: false,
        };

        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_raw_command_failure_ignored() {
        let task = RawTask {
            description: None,
            executable: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 1".to_string()],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            exit_codes: vec![0], // Expect success but command will fail
            ignore_errors: true, // But ignore errors
            creates: false,
            removes: false,
            creates_files: vec![],
            removes_files: vec![],
            force: false,
        };

        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok()); // Should succeed because ignore_errors=true
    }

    #[tokio::test]
    async fn test_raw_command_custom_exit_codes() {
        let task = RawTask {
            description: None,
            executable: "sh".to_string(),
            args: vec!["-c".to_string(), "exit 42".to_string()],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            exit_codes: vec![42], // Expect exit code 42
            ignore_errors: false,
            creates: false,
            removes: false,
            creates_files: vec![],
            removes_files: vec![],
            force: false,
        };

        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok()); // Should succeed because 42 is in exit_codes
    }

    #[tokio::test]
    async fn test_raw_command_nonexistent() {
        let task = RawTask {
            description: None,
            executable: "/nonexistent/command".to_string(),
            args: vec![],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            exit_codes: vec![0],
            ignore_errors: false,
            creates: false,
            removes: false,
            creates_files: vec![],
            removes_files: vec![],
            force: false,
        };

        let result = execute_raw_task(&task, false).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Executable does not exist"));
    }

    #[tokio::test]
    async fn test_raw_command_creates_validation() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary file that should be created
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();
        drop(temp_file); // Remove it so the command can create it

        let task = RawTask {
            description: None,
            executable: "touch".to_string(),
            args: vec![temp_path.clone()],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            exit_codes: vec![0],
            ignore_errors: false,
            creates: true,
            removes: false,
            creates_files: vec![temp_path.clone()],
            removes_files: vec![],
            force: false,
        };

        // First run should succeed (file doesn't exist)
        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok());
        assert!(Path::new(&temp_path).exists());

        // Second run should skip (file already exists)
        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok()); // Should succeed but skip execution

        // Clean up
        fs::remove_file(&temp_path).unwrap();
    }

    #[tokio::test]
    async fn test_raw_command_removes_validation() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary file that should be removed
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();
        fs::write(&temp_path, "test content").unwrap();

        let task = RawTask {
            description: None,
            executable: "rm".to_string(),
            args: vec![temp_path.clone()],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            exit_codes: vec![0],
            ignore_errors: false,
            creates: false,
            removes: true,
            creates_files: vec![],
            removes_files: vec![temp_path.clone()],
            force: false,
        };

        // First run should succeed (file exists)
        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok());
        assert!(!Path::new(&temp_path).exists());

        // Second run should skip (file doesn't exist)
        let result = execute_raw_task(&task, false).await;
        assert!(result.is_ok()); // Should succeed but skip execution
    }
}
