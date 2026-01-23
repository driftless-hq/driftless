//! UFW (Uncomplicated Firewall) management
//!
//! This module manages UFW firewall rules on Ubuntu/Debian systems.
//! It supports enabling/disabling UFW, adding/removing rules for ports and services.
//!
//! # Examples
//!
//! ## Enable UFW firewall
//!
//! This example enables the UFW firewall.
//!
//! **YAML Format:**
//! ```yaml
//! - type: ufw
//!   description: "Enable UFW firewall"
//!   state: enabled
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "ufw",
//!   "description": "Enable UFW firewall",
//!   "state": "enabled"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "ufw"
//! description = "Enable UFW firewall"
//! state = "enabled"
//! ```
//!
//! ## Allow SSH access
//!
//! This example allows SSH connections on port 22.
//!
//! **YAML Format:**
//! ```yaml
//! - type: ufw
//!   description: "Allow SSH access"
//!   state: allow
//!   port: "22"
//!   proto: tcp
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "ufw",
//!   "description": "Allow SSH access",
//!   "state": "allow",
//!   "port": "22",
//!   "proto": "tcp"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "ufw"
//! description = "Allow SSH access"
//! state = "allow"
//! port = "22"
//! proto = "tcp"
//! ```
//!
//! ## Allow HTTP and HTTPS
//!
//! This example allows web traffic on ports 80 and 443.
//!
//! **YAML Format:**
//! ```yaml
//! - type: ufw
//!   description: "Allow web traffic"
//!   state: allow
//!   port: "80,443"
//!   proto: tcp
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "ufw",
//!   "description": "Allow web traffic",
//!   "state": "allow",
//!   "port": "80,443",
//!   "proto": "tcp"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "ufw"
//! description = "Allow web traffic"
//! state = "allow"
//! port = "80,443"
//! proto = "tcp"
//! ```
//!
//! ## Deny specific IP address
//!
//! This example denies all traffic from a specific IP address.
//!
//! **YAML Format:**
//! ```yaml
//! - type: ufw
//!   description: "Block specific IP address"
//!   state: deny
//!   from: 192.168.1.100
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "ufw",
//!   "description": "Block specific IP address",
//!   "state": "deny",
//!   "from": "192.168.1.100"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "ufw"
//! description = "Block specific IP address"
//! state = "deny"
//! from = "192.168.1.100"
//! ```

use anyhow::{bail, Context, Result};
use std::process::Command;

/// UFW firewall management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UfwTask {
    /// Optional description of what this task does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// UFW state
    pub state: UfwState,

    /// Rule to manage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,

    /// Port to manage (e.g., "80", "443/tcp", "53/udp")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,

    /// Source IP/network (for from parameter)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,

    /// Destination IP/network (for to parameter)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,

    /// Interface to apply rule to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,

    /// Direction (in/out)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,

    /// Protocol (tcp/udp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proto: Option<String>,

    /// Logging level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<String>,

    /// Default policy for chains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// UFW state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UfwState {
    /// Enable UFW
    Enabled,
    /// Disable UFW
    Disabled,
    /// Reload UFW rules
    Reload,
    /// Reset UFW to defaults
    Reset,
    /// Allow traffic
    Allow,
    /// Deny traffic
    Deny,
    /// Reject traffic
    Reject,
    /// Limit traffic (rate limiting)
    Limit,
    /// Delete rule
    Delete,
    /// Set logging level
    Logging,
    /// Set default policy
    Default,
}

/// Execute UFW firewall task
pub async fn execute_ufw_task(task: &UfwTask, dry_run: bool) -> Result<()> {
    // Check if UFW is available
    if !dry_run && !is_ufw_available()? {
        bail!("UFW is not available on this system");
    }

    match task.state {
        UfwState::Enabled => ensure_ufw_enabled(dry_run).await,
        UfwState::Disabled => ensure_ufw_disabled(dry_run).await,
        UfwState::Reload => reload_ufw(dry_run).await,
        UfwState::Reset => reset_ufw(dry_run).await,
        UfwState::Allow => add_ufw_rule(task, "allow", dry_run).await,
        UfwState::Deny => add_ufw_rule(task, "deny", dry_run).await,
        UfwState::Reject => add_ufw_rule(task, "reject", dry_run).await,
        UfwState::Limit => add_ufw_rule(task, "limit", dry_run).await,
        UfwState::Delete => delete_ufw_rule(task, dry_run).await,
        UfwState::Logging => set_ufw_logging(task, dry_run).await,
        UfwState::Default => set_ufw_default(task, dry_run).await,
    }
}

/// Ensure UFW is enabled
async fn ensure_ufw_enabled(dry_run: bool) -> Result<()> {
    if is_ufw_enabled()? {
        return Ok(());
    }

    let cmd = "ufw --force enable";

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(cmd)?;
    }

    Ok(())
}

/// Ensure UFW is disabled
async fn ensure_ufw_disabled(dry_run: bool) -> Result<()> {
    if !is_ufw_enabled()? {
        return Ok(());
    }

    let cmd = "ufw disable";

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(cmd)?;
    }

    Ok(())
}

/// Reload UFW rules
async fn reload_ufw(dry_run: bool) -> Result<()> {
    let cmd = "ufw reload";

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(cmd)?;
    }

    Ok(())
}

/// Reset UFW to defaults
async fn reset_ufw(dry_run: bool) -> Result<()> {
    let cmd = "ufw --force reset";

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(cmd)?;
    }

    Ok(())
}

/// Add a UFW rule
async fn add_ufw_rule(task: &UfwTask, action: &str, dry_run: bool) -> Result<()> {
    let mut cmd_parts = vec!["ufw"];

    // Add action
    cmd_parts.push(action);

    // Add direction if specified
    if let Some(direction) = &task.direction {
        cmd_parts.push(direction);
    }

    // Add port if specified
    if let Some(port) = &task.port {
        cmd_parts.push(port);
    }

    // Add protocol if specified
    if let Some(proto) = &task.proto {
        cmd_parts.push(proto);
    }

    // Add from if specified
    if let Some(from) = &task.from {
        cmd_parts.push("from");
        cmd_parts.push(from);
    }

    // Add to if specified
    if let Some(to) = &task.to {
        cmd_parts.push("to");
        cmd_parts.push(to);
    }

    // Add interface if specified
    if let Some(interface) = &task.interface {
        cmd_parts.push("on");
        cmd_parts.push(interface);
    }

    let cmd = cmd_parts.join(" ");

    // Check if rule already exists
    if rule_exists(&cmd_parts[2..])? {
        return Ok(());
    }

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(&cmd)?;
    }

    Ok(())
}

/// Delete a UFW rule
async fn delete_ufw_rule(task: &UfwTask, dry_run: bool) -> Result<()> {
    let mut cmd_parts = vec!["ufw", "delete"];

    // Add rule if specified
    if let Some(rule) = &task.rule {
        cmd_parts.push(rule);
        let cmd = cmd_parts.join(" ");

        if dry_run {
            println!("DRY RUN: Would execute: {}", cmd);
        } else {
            run_ufw_cmd(&cmd)?;
        }
        return Ok(());
    }

    // Otherwise build rule from parameters
    // Add direction if specified
    if let Some(direction) = &task.direction {
        cmd_parts.push(direction);
    }

    // Add port if specified
    if let Some(port) = &task.port {
        cmd_parts.push(port);
    }

    // Add protocol if specified
    if let Some(proto) = &task.proto {
        cmd_parts.push(proto);
    }

    // Add from if specified
    if let Some(from) = &task.from {
        cmd_parts.push("from");
        cmd_parts.push(from);
    }

    // Add to if specified
    if let Some(to) = &task.to {
        cmd_parts.push("to");
        cmd_parts.push(to);
    }

    // Add interface if specified
    if let Some(interface) = &task.interface {
        cmd_parts.push("on");
        cmd_parts.push(interface);
    }

    let cmd = cmd_parts.join(" ");

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(&cmd)?;
    }

    Ok(())
}

/// Set UFW logging level
async fn set_ufw_logging(task: &UfwTask, dry_run: bool) -> Result<()> {
    let logging = task
        .logging
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Logging level must be specified"))?;
    let cmd = format!("ufw logging {}", logging);

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(&cmd)?;
    }

    Ok(())
}

/// Set UFW default policy
async fn set_ufw_default(task: &UfwTask, dry_run: bool) -> Result<()> {
    let default_policy = task
        .default
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Default policy must be specified"))?;
    let cmd = format!("ufw default {}", default_policy);

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_ufw_cmd(&cmd)?;
    }

    Ok(())
}

/// Check if UFW is available
fn is_ufw_available() -> Result<bool> {
    let output = Command::new("which")
        .arg("ufw")
        .output()
        .context("Failed to check if UFW is available")?;

    Ok(output.status.success())
}

/// Check if UFW is enabled
fn is_ufw_enabled() -> Result<bool> {
    let output = run_ufw_cmd("ufw status")?;
    Ok(output.contains("Status: active"))
}

/// Check if a rule already exists
fn rule_exists(rule_parts: &[&str]) -> Result<bool> {
    let output = run_ufw_cmd("ufw status")?;

    // Parse the output to check for existing rules
    // This is a simplified check - in practice, UFW status output is complex
    let rule_str = rule_parts.join(" ");
    Ok(output.contains(&rule_str))
}

/// Run a UFW command and return its output
fn run_ufw_cmd(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("Failed to execute UFW command: {}", cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("UFW command failed: {} (stderr: {})", cmd, stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ufw_task_validation() {
        // Test logging without logging level specified - should fail
        let task = UfwTask {
            description: None,
            state: UfwState::Logging,
            rule: None,
            port: None,
            from: None,
            to: None,
            interface: None,
            direction: None,
            proto: None,
            logging: None,
            default: None,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { execute_ufw_task(&task, true).await });

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Logging level must be specified"));
    }

    #[test]
    fn test_ufw_task_default_validation() {
        // Test default without default policy specified - should fail
        let task = UfwTask {
            description: None,
            state: UfwState::Default,
            rule: None,
            port: None,
            from: None,
            to: None,
            interface: None,
            direction: None,
            proto: None,
            logging: None,
            default: None,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { execute_ufw_task(&task, true).await });

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Default policy must be specified"));
    }
}
