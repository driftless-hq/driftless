//! SELinux policy management
//!
//! This module manages SELinux policies, contexts, and boolean values.
//! It supports setting file contexts, managing booleans, and policy operations.
//!
//! # Examples
//!
//! ## Enable SELinux boolean
//!
//! This example enables an SELinux boolean.
//!
//! **YAML Format:**
//! ```yaml
//! - type: selinux
//!   description: "Enable httpd_can_network_connect"
//!   state: on
//!   boolean: httpd_can_network_connect
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "selinux",
//!   "description": "Enable httpd_can_network_connect",
//!   "state": "on",
//!   "boolean": "httpd_can_network_connect"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "selinux"
//! description = "Enable httpd_can_network_connect"
//! state = "on"
//! boolean = "httpd_can_network_connect"
//! ```
//!
//! ## Set file context
//!
//! This example sets the SELinux context for a file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: selinux
//!   description: "Set httpd context for web directory"
//!   state: context
//!   target: /var/www/html
//!   setype: httpd_sys_content_t
//!   recurse: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "selinux",
//!   "description": "Set httpd context for web directory",
//!   "state": "context",
//!   "target": "/var/www/html",
//!   "setype": "httpd_sys_content_t",
//!   "recurse": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "selinux"
//! description = "Set httpd context for web directory"
//! state = "context"
//! target = "/var/www/html"
//! setype = "httpd_sys_content_t"
//! recurse = true
//! ```
//!
//! ## Set SELinux to enforcing mode
//!
//! This example sets SELinux to enforcing mode.
//!
//! **YAML Format:**
//! ```yaml
//! - type: selinux
//!   description: "Set SELinux to enforcing mode"
//!   state: enforcing
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "selinux",
//!   "description": "Set SELinux to enforcing mode",
//!   "state": "enforcing"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "selinux"
//! description = "Set SELinux to enforcing mode"
//! state = "enforcing"
//! ```
//!
//! ## Restore file contexts
//!
//! This example restores SELinux contexts for files.
//!
//! **YAML Format:**
//! ```yaml
//! - type: selinux
//!   description: "Restore SELinux contexts"
//!   state: restorecon
//!   target: /etc/httpd
//!   recurse: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "selinux",
//!   "description": "Restore SELinux contexts",
//!   "state": "restorecon",
//!   "target": "/etc/httpd",
//!   "recurse": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "selinux"
//! description = "Restore SELinux contexts"
//! state = "restorecon"
//! target = "/etc/httpd"
//! recurse = true
//! ```

use std::process::Command;
use std::path::Path;
use anyhow::{Result, Context, bail};

/// SELinux policy management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SelinuxTask {
    /// Optional description of what this task does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// SELinux state (present/absent/enforcing/permissive/disabled)
    pub state: SelinuxState,

    /// SELinux boolean to manage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean: Option<String>,

    /// File/directory to set context on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// SELinux context to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// SELinux type to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setype: Option<String>,

    /// SELinux user to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seuser: Option<String>,

    /// SELinux role to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serole: Option<String>,

    /// SELinux level/range to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serange: Option<String>,

    /// Whether to recurse into directories
    #[serde(default)]
    pub recurse: bool,

    /// Whether to follow symlinks
    #[serde(default)]
    pub follow: bool,

    /// Whether to ignore missing files
    #[serde(default)]
    pub ignore_missing: bool,

    /// Whether to make changes persistent
    #[serde(default)]
    pub persistent: bool,

    /// Policy type (targeted/mls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
}

/// SELinux state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelinuxState {
    /// Ensure boolean is on
    On,
    /// Ensure boolean is off
    Off,
    /// Set enforcing mode
    Enforcing,
    /// Set permissive mode
    Permissive,
    /// Set disabled mode
    Disabled,
    /// Set file context
    Context,
    /// Restore file context from policy
    Restorecon,
}

/// Execute SELinux policy task
pub async fn execute_selinux_task(
    task: &SelinuxTask,
    dry_run: bool,
) -> Result<()> {
    // Check if SELinux is available
    if !dry_run && !is_selinux_available()? {
        bail!("SELinux is not available on this system");
    }

    match task.state {
        SelinuxState::On => {
            ensure_selinux_boolean_on(task, dry_run).await
        }
        SelinuxState::Off => {
            ensure_selinux_boolean_off(task, dry_run).await
        }
        SelinuxState::Enforcing => {
            set_selinux_enforcing(dry_run).await
        }
        SelinuxState::Permissive => {
            set_selinux_permissive(dry_run).await
        }
        SelinuxState::Disabled => {
            set_selinux_disabled(dry_run).await
        }
        SelinuxState::Context => {
            set_selinux_context(task, dry_run).await
        }
        SelinuxState::Restorecon => {
            restore_selinux_context(task, dry_run).await
        }
    }
}

/// Ensure SELinux boolean is on
async fn ensure_selinux_boolean_on(
    task: &SelinuxTask,
    dry_run: bool,
) -> Result<()> {
    let boolean = task.boolean.as_ref().ok_or_else(|| anyhow::anyhow!("Boolean name must be specified"))?;

    if is_selinux_boolean_on(boolean)? {
        return Ok(());
    }

    let cmd = format!("setsebool -P {} on", boolean);

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_selinux_cmd(&cmd)?;
    }

    Ok(())
}

/// Ensure SELinux boolean is off
async fn ensure_selinux_boolean_off(
    task: &SelinuxTask,
    dry_run: bool,
) -> Result<()> {
    let boolean = task.boolean.as_ref().ok_or_else(|| anyhow::anyhow!("Boolean name must be specified"))?;

    if !is_selinux_boolean_on(boolean)? {
        return Ok(());
    }

    let cmd = format!("setsebool -P {} off", boolean);

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_selinux_cmd(&cmd)?;
    }

    Ok(())
}

/// Set SELinux to enforcing mode
async fn set_selinux_enforcing(dry_run: bool) -> Result<()> {
    if is_selinux_enforcing()? {
        return Ok(());
    }

    let cmd = "setenforce 1";

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_selinux_cmd(cmd)?;
    }

    Ok(())
}

/// Set SELinux to permissive mode
async fn set_selinux_permissive(dry_run: bool) -> Result<()> {
    if !is_selinux_enforcing()? {
        return Ok(());
    }

    let cmd = "setenforce 0";

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_selinux_cmd(cmd)?;
    }

    Ok(())
}

/// Set SELinux to disabled mode
async fn set_selinux_disabled(_dry_run: bool) -> Result<()> {
    if is_selinux_disabled()? {
        return Ok(());
    }

    bail!("Disabling SELinux requires a system reboot and configuration changes. Use setenforce 0 for temporary permissive mode.");
}

/// Set SELinux context on files
async fn set_selinux_context(
    task: &SelinuxTask,
    dry_run: bool,
) -> Result<()> {
    let target = task.target.as_ref().ok_or_else(|| anyhow::anyhow!("Target path must be specified"))?;

    if !Path::new(target).exists() && !task.ignore_missing {
        bail!("Target path does not exist: {}", target);
    }

    if !Path::new(target).exists() && task.ignore_missing {
        return Ok(());
    }

    let mut cmd_parts = vec!["chcon".to_string()];

    if task.recurse {
        cmd_parts.push("-R".to_string());
    }

    // Build context specification
    let mut context_parts = Vec::new();
    if let Some(seuser) = &task.seuser {
        context_parts.push(format!("u:{}", seuser));
    }
    if let Some(serole) = &task.serole {
        context_parts.push(format!("r:{}", serole));
    }
    if let Some(setype) = &task.setype {
        context_parts.push(format!("t:{}", setype));
    }
    if let Some(serange) = &task.serange {
        context_parts.push(format!("s:{}", serange));
    }

    if !context_parts.is_empty() {
        cmd_parts.push("--type".to_string());
        let context_value = context_parts.join(":");
        cmd_parts.push(context_value);
    }

    cmd_parts.push(target.to_string());

    let cmd = cmd_parts.join(" ");

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_selinux_cmd(&cmd)?;
    }

    Ok(())
}

/// Restore SELinux context from policy
async fn restore_selinux_context(
    task: &SelinuxTask,
    dry_run: bool,
) -> Result<()> {
    let target = task.target.as_ref().ok_or_else(|| anyhow::anyhow!("Target path must be specified"))?;

    if !Path::new(target).exists() && !task.ignore_missing {
        bail!("Target path does not exist: {}", target);
    }

    if !Path::new(target).exists() && task.ignore_missing {
        return Ok(());
    }

    let mut cmd_parts = vec!["restorecon".to_string()];

    if task.recurse {
        cmd_parts.push("-R".to_string());
    }

    if task.follow {
        cmd_parts.push("-L".to_string());
    }

    cmd_parts.push(target.to_string());

    let cmd = cmd_parts.join(" ");

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_selinux_cmd(&cmd)?;
    }

    Ok(())
}

/// Check if SELinux is available
fn is_selinux_available() -> Result<bool> {
    let output = Command::new("selinuxenabled")
        .output()
        .context("Failed to check SELinux status")?;

    // selinuxenabled returns 0 if SELinux is enabled, 1 if disabled
    Ok(output.status.success())
}

/// Check if SELinux boolean is on
fn is_selinux_boolean_on(boolean: &str) -> Result<bool> {
    let output = run_selinux_cmd(&format!("getsebool {}", boolean))?;
    Ok(output.contains("on"))
}

/// Check if SELinux is in enforcing mode
fn is_selinux_enforcing() -> Result<bool> {
    let output = run_selinux_cmd("getenforce")?;
    Ok(output.trim() == "Enforcing")
}

/// Check if SELinux is disabled
fn is_selinux_disabled() -> Result<bool> {
    let output = run_selinux_cmd("getenforce")?;
    Ok(output.trim() == "Disabled")
}

/// Run a SELinux command and return its output
fn run_selinux_cmd(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("Failed to execute SELinux command: {}", cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("SELinux command failed: {} (stderr: {})", cmd, stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selinux_task_validation() {
        // Test with no boolean specified for On state - should fail
        let task = SelinuxTask {
            description: None,
            state: SelinuxState::On,
            boolean: None,
            target: None,
            context: None,
            setype: None,
            seuser: None,
            serole: None,
            serange: None,
            recurse: false,
            follow: false,
            ignore_missing: false,
            persistent: false,
            policy: None,
        };

        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            execute_selinux_task(&task, true).await
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Boolean name must be specified"));
    }

    #[test]
    fn test_selinux_task_context_validation() {
        // Test with no target specified for Context state - should fail
        let task = SelinuxTask {
            description: None,
            state: SelinuxState::Context,
            boolean: None,
            target: None,
            context: None,
            setype: Some("httpd_sys_content_t".to_string()),
            seuser: None,
            serole: None,
            serange: None,
            recurse: false,
            follow: false,
            ignore_missing: false,
            persistent: false,
            policy: None,
        };

        let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
            execute_selinux_task(&task, true).await
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Target path must be specified"));
    }
}