//! Firewalld firewall management
//!
//! This module manages firewalld firewall rules and configuration.
//! It supports adding/removing services, ports, and rich rules.
//!
//! # Examples
//!
//! ## Allow SSH service
//!
//! This example allows SSH service through the firewall.
//!
//! **YAML Format:**
//! ```yaml
//! - type: firewalld
//!   description: "Allow SSH access"
//!   state: present
//!   service: ssh
//!   zone: public
//!   permanent: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "firewalld",
//!   "description": "Allow SSH access",
//!   "state": "present",
//!   "service": "ssh",
//!   "zone": "public",
//!   "permanent": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "firewalld"
//! description = "Allow SSH access"
//! state = "present"
//! service = "ssh"
//! zone = "public"
//! permanent = true
//! ```
//!
//! ## Allow custom port
//!
//! This example allows traffic on a custom port.
//!
//! **YAML Format:**
//! ```yaml
//! - type: firewalld
//!   description: "Allow web traffic on port 8080"
//!   state: present
//!   port: "8080/tcp"
//!   zone: public
//!   permanent: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "firewalld",
//!   "description": "Allow web traffic on port 8080",
//!   "state": "present",
//!   "port": "8080/tcp",
//!   "zone": "public",
//!   "permanent": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "firewalld"
//! description = "Allow web traffic on port 8080"
//! state = "present"
//! port = "8080/tcp"
//! zone = "public"
//! permanent = true
//! ```
//!
//! ## Add rich rule
//!
//! This example adds a rich rule for advanced firewall configuration.
//!
//! **YAML Format:**
//! ```yaml
//! - type: firewalld
//!   description: "Allow traffic from specific IP"
//!   state: present
//!   rich_rule: 'rule family="ipv4" source address="192.168.1.100" accept'
//!   zone: public
//!   permanent: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "firewalld",
//!   "description": "Allow traffic from specific IP",
//!   "state": "present",
//!   "rich_rule": "rule family=\"ipv4\" source address=\"192.168.1.100\" accept",
//!   "zone": "public",
//!   "permanent": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "firewalld"
//! description = "Allow traffic from specific IP"
//! state = "present"
//! rich_rule = 'rule family="ipv4" source address="192.168.1.100" accept'
//! zone = "public"
//! permanent = true
//! ```
//!
//! ## Remove firewall rule
//!
//! This example removes a firewall rule.
//!
//! **YAML Format:**
//! ```yaml
//! - type: firewalld
//!   description: "Remove SSH access"
//!   state: absent
//!   service: ssh
//!   zone: public
//!   permanent: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "firewalld",
//!   "description": "Remove SSH access",
//!   "state": "absent",
//!   "service": "ssh",
//!   "zone": "public",
//!   "permanent": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "firewalld"
//! description = "Remove SSH access"
//! state = "absent"
//! service = "ssh"
//! zone = "public"
//! permanent = true
//! ```

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Firewalld firewall management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FirewalldTask {
    /// Optional description of what this task does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Firewall state (present/absent)
    pub state: FirewalldState,

    /// Service to manage (e.g., "http", "ssh")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// Port to manage (e.g., "8080/tcp", "53/udp")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,

    /// Rich rule to manage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rich_rule: Option<String>,

    /// Zone to manage (defaults to "public")
    #[serde(default = "default_zone")]
    pub zone: String,

    /// Whether to make changes permanent
    #[serde(default = "default_true")]
    pub permanent: bool,

    /// Whether to reload firewall after changes
    #[serde(default = "default_true")]
    pub reload: bool,

    /// Whether to check if firewalld is running
    #[serde(default = "default_true")]
    pub check_running: bool,
}

/// Firewalld state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewalldState {
    /// Ensure rule is present
    Present,
    /// Ensure rule is absent
    Absent,
}

/// Execute firewalld firewall task
pub async fn execute_firewalld_task(task: &FirewalldTask, dry_run: bool) -> Result<()> {
    // Check if firewalld is running if requested
    if task.check_running && !is_firewalld_running()? {
        bail!("firewalld is not running");
    }

    // Validate that exactly one of service, port, or rich_rule is specified
    let mut rule_count = 0;
    if task.service.is_some() {
        rule_count += 1;
    }
    if task.port.is_some() {
        rule_count += 1;
    }
    if task.rich_rule.is_some() {
        rule_count += 1;
    }

    if rule_count != 1 {
        bail!("Exactly one of 'service', 'port', or 'rich_rule' must be specified");
    }

    match task.state {
        FirewalldState::Present => ensure_firewalld_rule_present(task, dry_run).await,
        FirewalldState::Absent => ensure_firewalld_rule_absent(task, dry_run).await,
    }
}

/// Ensure firewalld rule is present
async fn ensure_firewalld_rule_present(task: &FirewalldTask, dry_run: bool) -> Result<()> {
    let mut commands = Vec::new();

    if let Some(service) = &task.service {
        if !is_service_enabled(&task.zone, service)? {
            commands.push(format!(
                "firewall-cmd --zone={} --add-service={}",
                task.zone, service
            ));
            if task.permanent {
                commands.push(format!(
                    "firewall-cmd --zone={} --add-service={} --permanent",
                    task.zone, service
                ));
            }
        }
    } else if let Some(port) = &task.port {
        if !is_port_enabled(&task.zone, port)? {
            commands.push(format!(
                "firewall-cmd --zone={} --add-port={}",
                task.zone, port
            ));
            if task.permanent {
                commands.push(format!(
                    "firewall-cmd --zone={} --add-port={} --permanent",
                    task.zone, port
                ));
            }
        }
    } else if let Some(rich_rule) = &task.rich_rule {
        if !is_rich_rule_enabled(&task.zone, rich_rule)? {
            commands.push(format!(
                "firewall-cmd --zone={} --add-rich-rule='{}'",
                task.zone, rich_rule
            ));
            if task.permanent {
                commands.push(format!(
                    "firewall-cmd --zone={} --add-rich-rule='{}' --permanent",
                    task.zone, rich_rule
                ));
            }
        }
    }

    // Execute commands
    for cmd in &commands {
        if dry_run {
            println!("DRY RUN: Would execute: {}", cmd);
        } else {
            run_firewall_cmd(cmd)?;
        }
    }

    // Reload if requested and we made changes
    if task.reload && !commands.is_empty() && !dry_run {
        run_firewall_cmd("firewall-cmd --reload")?;
    }

    Ok(())
}

/// Ensure firewalld rule is absent
async fn ensure_firewalld_rule_absent(task: &FirewalldTask, dry_run: bool) -> Result<()> {
    let mut commands = Vec::new();

    if let Some(service) = &task.service {
        if is_service_enabled(&task.zone, service)? {
            commands.push(format!(
                "firewall-cmd --zone={} --remove-service={}",
                task.zone, service
            ));
            if task.permanent {
                commands.push(format!(
                    "firewall-cmd --zone={} --remove-service={} --permanent",
                    task.zone, service
                ));
            }
        }
    } else if let Some(port) = &task.port {
        if is_port_enabled(&task.zone, port)? {
            commands.push(format!(
                "firewall-cmd --zone={} --remove-port={}",
                task.zone, port
            ));
            if task.permanent {
                commands.push(format!(
                    "firewall-cmd --zone={} --remove-port={} --permanent",
                    task.zone, port
                ));
            }
        }
    } else if let Some(rich_rule) = &task.rich_rule {
        if is_rich_rule_enabled(&task.zone, rich_rule)? {
            commands.push(format!(
                "firewall-cmd --zone={} --remove-rich-rule='{}'",
                task.zone, rich_rule
            ));
            if task.permanent {
                commands.push(format!(
                    "firewall-cmd --zone={} --remove-rich-rule='{}' --permanent",
                    task.zone, rich_rule
                ));
            }
        }
    }

    // Execute commands
    for cmd in &commands {
        if dry_run {
            println!("DRY RUN: Would execute: {}", cmd);
        } else {
            run_firewall_cmd(cmd)?;
        }
    }

    // Reload if requested and we made changes
    if task.reload && !commands.is_empty() && !dry_run {
        run_firewall_cmd("firewall-cmd --reload")?;
    }

    Ok(())
}

/// Check if firewalld is running
fn is_firewalld_running() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["is-active", "firewalld"])
        .output()
        .context("Failed to check firewalld status")?;

    Ok(output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "active")
}

/// Check if a service is enabled in a zone
fn is_service_enabled(zone: &str, service: &str) -> Result<bool> {
    let output = run_firewall_cmd(&format!(
        "firewall-cmd --zone={} --query-service={}",
        zone, service
    ))?;
    Ok(output.trim() == "yes")
}

/// Check if a port is enabled in a zone
fn is_port_enabled(zone: &str, port: &str) -> Result<bool> {
    let output = run_firewall_cmd(&format!(
        "firewall-cmd --zone={} --query-port={}",
        zone, port
    ))?;
    Ok(output.trim() == "yes")
}

/// Check if a rich rule is enabled in a zone
fn is_rich_rule_enabled(zone: &str, rich_rule: &str) -> Result<bool> {
    let output = run_firewall_cmd(&format!(
        "firewall-cmd --zone={} --query-rich-rule='{}'",
        zone, rich_rule
    ))?;
    Ok(output.trim() == "yes")
}

/// Run a firewall command and return its output
fn run_firewall_cmd(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("Failed to execute firewall command: {}", cmd))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Firewall command failed: {} (stderr: {})", cmd, stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn default_zone() -> String {
    "public".to_string()
}

pub fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firewalld_task_validation() {
        // Test with no rule specified - should fail
        let task = FirewalldTask {
            description: None,
            state: FirewalldState::Present,
            service: None,
            port: None,
            rich_rule: None,
            zone: "public".to_string(),
            permanent: true,
            reload: true,
            check_running: false,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { execute_firewalld_task(&task, true).await });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Exactly one of"));
    }

    #[test]
    fn test_firewalld_task_multiple_rules() {
        // Test with multiple rules specified - should fail
        let task = FirewalldTask {
            description: None,
            state: FirewalldState::Present,
            service: Some("http".to_string()),
            port: Some("8080/tcp".to_string()),
            rich_rule: None,
            zone: "public".to_string(),
            permanent: true,
            reload: true,
            check_running: false,
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { execute_firewalld_task(&task, true).await });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Exactly one of"));
    }
}
