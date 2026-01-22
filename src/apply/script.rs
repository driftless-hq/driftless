//! Execute local scripts task executor
//!
//! Handles execution of local script files with various options.
//!
//! # Examples
//!
//! ## Execute a script
//!
//! This example executes a setup script.
//!
//! **YAML Format:**
//! ```yaml
//! - type: script
//!   description: "Run setup script"
//!   path: /usr/local/bin/setup.sh
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "script",
//!   "description": "Run setup script",
//!   "path": "/usr/local/bin/setup.sh"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "script"
//! description = "Run setup script"
//! path = "/usr/local/bin/setup.sh"
//! ```
//!
//! ## Execute script with parameters
//!
//! This example executes a script with command line arguments.
//!
//! **YAML Format:**
//! ```yaml
//! - type: script
//!   description: "Run deployment script with environment"
//!   path: /opt/deploy/deploy.sh
//!   params: ["production", "--verbose"]
//!   chdir: /opt/deploy
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "script",
//!   "description": "Run deployment script with environment",
//!   "path": "/opt/deploy/deploy.sh",
//!   "params": ["production", "--verbose"],
//!   "chdir": "/opt/deploy"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "script"
//! description = "Run deployment script with environment"
//! path = "/opt/deploy/deploy.sh"
//! params = ["production", "--verbose"]
//! chdir = "/opt/deploy"
//! ```
//!
//! ## Execute script with environment variables
//!
//! This example executes a script with custom environment variables.
//!
//! **YAML Format:**
//! ```yaml
//! - type: script
//!   description: "Run script with environment"
//!   path: /usr/local/bin/configure.sh
//!   environment:
//!     DATABASE_URL: "postgresql://localhost/mydb"
//!     API_KEY: "secret-key"
//!   timeout: 300
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "script",
//!   "description": "Run script with environment",
//!   "path": "/usr/local/bin/configure.sh",
//!   "environment": {
//!     "DATABASE_URL": "postgresql://localhost/mydb",
//!     "API_KEY": "secret-key"
//!   },
//!   "timeout": 300
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "script"
//! description = "Run script with environment"
//! path = "/usr/local/bin/configure.sh"
//! environment = { DATABASE_URL = "postgresql://localhost/mydb", API_KEY = "secret-key" }
//! timeout = 300
//! ```
//!
//! ## Execute script with creates/removes checks
//!
//! This example executes a script only if certain conditions are met.
//!
//! **YAML Format:**
//! ```yaml
//! - type: script
//!   description: "Run initialization script"
//!   path: /usr/local/bin/init.sh
//!   creates: true
//!   timeout: 600
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "script",
//!   "description": "Run initialization script",
//!   "path": "/usr/local/bin/init.sh",
//!   "creates": true,
//!   "timeout": 600
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "script"
//! description = "Run initialization script"
//! path = "/usr/local/bin/init.sh"
//! creates = true
//! timeout = 600
//! ```

/// Execute local scripts task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Path to the script file
    pub path: String,
    /// Script parameters/arguments
    #[serde(default)]
    pub params: Vec<String>,
    /// Working directory for script execution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chdir: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,
    /// Execution timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
    /// Whether the script creates resources
    #[serde(default)]
    pub creates: bool,
    /// Whether the script removes resources
    #[serde(default)]
    pub removes: bool,
    /// Force script execution
    #[serde(default)]
    pub force: bool,
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Execute a script task
pub async fn execute_script_task(task: &ScriptTask, dry_run: bool) -> Result<()> {
    let script_path = Path::new(&task.path);

    // Check if script exists
    if !script_path.exists() {
        return Err(anyhow::anyhow!("Script does not exist: {}", task.path));
    }

    // Check if script is executable
    if !is_executable(script_path)? {
        return Err(anyhow::anyhow!("Script is not executable: {}", task.path));
    }

    // Check if script creates/removes resources
    if task.creates && !dry_run {
        // Check if resources already exist
        // This is a simplified check - in practice, you'd check for specific files/directories
        println!("Note: 'creates' check not fully implemented for scripts");
    }

    if task.removes && !dry_run {
        // Check if resources need to be removed
        // This is a simplified check - in practice, you'd check for specific files/directories
        println!("Note: 'removes' check not fully implemented for scripts");
    }

    if dry_run {
        println!("Would execute script: {} with params: {:?}", task.path, task.params);
        if let Some(ref chdir) = task.chdir {
            println!("  (in directory: {})", chdir);
        }
        if !task.environment.is_empty() {
            println!("  (with environment variables: {} vars)", task.environment.len());
        }
        return Ok(());
    }

    // Execute the script
    let mut command = Command::new(&task.path);

    // Add parameters
    command.args(&task.params);

    // Set working directory
    if let Some(ref chdir) = task.chdir {
        command.current_dir(chdir);
    }

    // Set environment variables
    for (key, value) in &task.environment {
        command.env(key, value);
    }

    // Set timeout if specified
    if let Some(timeout_secs) = task.timeout {
        // Note: In a real implementation, you'd use tokio::process::Command
        // with timeout handling. For now, we'll execute synchronously.
        println!("Note: Script timeout not implemented (would timeout after {}s)", timeout_secs);
    }

    let output = command
        .output()
        .with_context(|| format!("Failed to execute script: {}", task.path))?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(anyhow::anyhow!(
            "Script execution failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            stdout,
            stderr
        ));
    }

    println!("Executed script: {}", task.path);
    Ok(())
}

/// Check if a file is executable
fn is_executable(path: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for {}", path.display()))?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check if any execute bit is set (owner, group, or other)
    Ok(mode & 0o111 != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_script_execution_dry_run() {
        let script_file = NamedTempFile::new().unwrap();
        let script_path = script_file.path().to_str().unwrap().to_string();

        // Create a simple script
        let script_content = "#!/bin/bash\necho 'Hello World'\n";
        fs::write(&script_path, script_content).unwrap();

        // Make it executable
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        let task = ScriptTask {
            description: None,
            path: script_path.clone(),
            params: vec!["arg1".to_string(), "arg2".to_string()],
            chdir: Some("/tmp".to_string()),
            environment: vec![("TEST_VAR".to_string(), "test_value".to_string())].into_iter().collect(),
            timeout: Some(30),
            creates: false,
            removes: false,
            force: false,
        };

        let result = execute_script_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_script_nonexistent() {
        let task = ScriptTask {
            description: None,
            path: "/nonexistent/script.sh".to_string(),
            params: vec![],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            creates: false,
            removes: false,
            force: false,
        };

        let result = execute_script_task(&task, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Script does not exist"));
    }

    #[tokio::test]
    async fn test_script_not_executable() {
        let script_file = NamedTempFile::new().unwrap();
        let script_path = script_file.path().to_str().unwrap().to_string();

        // Create a script but don't make it executable
        let script_content = "#!/bin/bash\necho 'Hello World'\n";
        fs::write(&script_path, script_content).unwrap();

        // Ensure it's not executable (remove execute permissions)
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&script_path, perms).unwrap();

        let task = ScriptTask {
            description: None,
            path: script_path,
            params: vec![],
            chdir: None,
            environment: HashMap::new(),
            timeout: None,
            creates: false,
            removes: false,
            force: false,
        };

        let result = execute_script_task(&task, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Script is not executable"));
    }

    #[test]
    fn test_is_executable() {
        let test_file = NamedTempFile::new().unwrap();
        let file_path = test_file.path();

        // Test non-executable file
        let result = is_executable(file_path);
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Make it executable
        let mut perms = fs::metadata(file_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(file_path, perms).unwrap();

        // Test executable file
        let result = is_executable(file_path);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}