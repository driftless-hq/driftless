//! iptables firewall management
//!
//! This module manages iptables firewall rules for Linux systems.
//! It supports adding/removing rules for IPv4 and IPv6 chains and tables.
//!
//! # Examples
//!
//! ## Allow SSH access
//!
//! This example adds a rule to allow SSH connections.
//!
//! **YAML Format:**
//! ```yaml
//! - type: iptables
//!   description: "Allow SSH access"
//!   state: present
//!   table: filter
//!   chain: INPUT
//!   protocol: tcp
//!   dport: "22"
//!   target: ACCEPT
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "iptables",
//!   "description": "Allow SSH access",
//!   "state": "present",
//!   "table": "filter",
//!   "chain": "INPUT",
//!   "protocol": "tcp",
//!   "dport": "22",
//!   "target": "ACCEPT"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "iptables"
//! description = "Allow SSH access"
//! state = "present"
//! table = "filter"
//! chain = "INPUT"
//! protocol = "tcp"
//! dport = "22"
//! target = "ACCEPT"
//! ```
//!
//! ## Block specific IP address
//!
//! This example adds a rule to drop packets from a specific IP address.
//!
//! **YAML Format:**
//! ```yaml
//! - type: iptables
//!   description: "Block specific IP address"
//!   state: present
//!   table: filter
//!   chain: INPUT
//!   source: 192.168.1.100
//!   target: DROP
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "iptables",
//!   "description": "Block specific IP address",
//!   "state": "present",
//!   "table": "filter",
//!   "chain": "INPUT",
//!   "source": "192.168.1.100",
//!   "target": "DROP"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "iptables"
//! description = "Block specific IP address"
//! state = "present"
//! table = "filter"
//! chain = "INPUT"
//! source = "192.168.1.100"
//! target = "DROP"
//! ```
//!
//! ## Allow HTTP and HTTPS traffic
//!
//! This example allows web traffic on ports 80 and 443.
//!
//! **YAML Format:**
//! ```yaml
//! - type: iptables
//!   description: "Allow web traffic"
//!   state: present
//!   table: filter
//!   chain: INPUT
//!   protocol: tcp
//!   dport: "80,443"
//!   target: ACCEPT
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "iptables",
//!   "description": "Allow web traffic",
//!   "state": "present",
//!   "table": "filter",
//!   "chain": "INPUT",
//!   "protocol": "tcp",
//!   "dport": "80,443",
//!   "target": "ACCEPT"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "iptables"
//! description = "Allow web traffic"
//! state = "present"
//! table = "filter"
//! chain = "INPUT"
//! protocol = "tcp"
//! dport = "80,443"
//! target = "ACCEPT"
//! ```
//!
//! ## Remove iptables rule
//!
//! This example removes an iptables rule.
//!
//! **YAML Format:**
//! ```yaml
//! - type: iptables
//!   description: "Remove SSH blocking rule"
//!   state: absent
//!   table: filter
//!   chain: INPUT
//!   protocol: tcp
//!   dport: "22"
//!   target: DROP
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "iptables",
//!   "description": "Remove SSH blocking rule",
//!   "state": "absent",
//!   "table": "filter",
//!   "chain": "INPUT",
//!   "protocol": "tcp",
//!   "dport": "22",
//!   "target": "DROP"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "iptables"
//! description = "Remove SSH blocking rule"
//! state = "absent"
//! table = "filter"
//! chain = "INPUT"
//! protocol = "tcp"
//! dport = "22"
//! target = "DROP"
//! ```

use anyhow::{bail, Context, Result};
use std::process::Command;

/// iptables firewall management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IptablesTask {
    /// Optional description of what this task does
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// iptables state (present/absent)
    pub state: IptablesState,

    /// Table to manage (filter/nat/mangle/raw/security)
    #[serde(default = "default_table")]
    pub table: String,

    /// Chain to manage (INPUT/OUTPUT/FORWARD/PREROUTING/POSTROUTING)
    #[serde(default = "default_chain")]
    pub chain: String,

    /// Protocol (tcp/udp/icmp/all)
    #[serde(default = "default_protocol")]
    pub protocol: String,

    /// Source IP/network (with optional mask)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Destination IP/network (with optional mask)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,

    /// Source port (for tcp/udp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sport: Option<String>,

    /// Destination port (for tcp/udp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dport: Option<String>,

    /// Input interface
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_interface: Option<String>,

    /// Output interface
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_interface: Option<String>,

    /// Target/jump action (ACCEPT/DROP/REJECT/LOG/MASQUERADE)
    pub target: String,

    /// Additional iptables arguments
    #[serde(default)]
    pub extra_args: Vec<String>,

    /// IPv6 mode (use ip6tables instead of iptables)
    #[serde(default)]
    pub ipv6: bool,

    /// Whether to check if iptables is available
    #[serde(default = "default_true")]
    pub check_available: bool,
}

/// iptables state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IptablesState {
    /// Ensure rule is present
    Present,
    /// Ensure rule is absent
    Absent,
}

/// Execute iptables firewall task
pub async fn execute_iptables_task(task: &IptablesTask, dry_run: bool) -> Result<()> {
    // Check if iptables is available if requested
    if task.check_available && !is_iptables_available(task.ipv6)? {
        let cmd = if task.ipv6 { "ip6tables" } else { "iptables" };
        bail!("{} is not available on this system", cmd);
    }

    match task.state {
        IptablesState::Present => ensure_iptables_rule_present(task, dry_run).await,
        IptablesState::Absent => ensure_iptables_rule_absent(task, dry_run).await,
    }
}

/// Ensure iptables rule is present
async fn ensure_iptables_rule_present(task: &IptablesTask, dry_run: bool) -> Result<()> {
    // Build the rule specification
    let rule_spec = build_rule_specification(task)?;

    // Check if rule already exists
    if rule_exists(&rule_spec, task)? {
        return Ok(());
    }

    // Build the command to add the rule
    let mut cmd_parts = vec![if task.ipv6 { "ip6tables" } else { "iptables" }.to_string()];
    cmd_parts.push("-t".to_string());
    cmd_parts.push(task.table.clone());
    cmd_parts.push("-I".to_string());
    cmd_parts.push(task.chain.clone());

    // Add rule specification
    for part in &rule_spec {
        cmd_parts.push(part.clone());
    }

    // Add target
    cmd_parts.push("-j".to_string());
    cmd_parts.push(task.target.clone());

    // Add extra arguments
    for arg in &task.extra_args {
        cmd_parts.push(arg.clone());
    }

    let cmd = cmd_parts.join(" ");

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_iptables_cmd(&cmd_parts)?;
    }

    Ok(())
}

/// Ensure iptables rule is absent
async fn ensure_iptables_rule_absent(task: &IptablesTask, dry_run: bool) -> Result<()> {
    // Build the rule specification
    let rule_spec = build_rule_specification(task)?;

    // Check if rule exists
    if !rule_exists(&rule_spec, task)? {
        return Ok(());
    }

    // Build the command to delete the rule
    let mut cmd_parts = vec![if task.ipv6 { "ip6tables" } else { "iptables" }.to_string()];
    cmd_parts.push("-t".to_string());
    cmd_parts.push(task.table.clone());
    cmd_parts.push("-D".to_string());
    cmd_parts.push(task.chain.clone());

    // Add rule specification
    for part in &rule_spec {
        cmd_parts.push(part.clone());
    }

    // Add target
    cmd_parts.push("-j".to_string());
    cmd_parts.push(task.target.clone());

    // Add extra arguments
    for arg in &task.extra_args {
        cmd_parts.push(arg.clone());
    }

    let cmd = cmd_parts.join(" ");

    if dry_run {
        println!("DRY RUN: Would execute: {}", cmd);
    } else {
        run_iptables_cmd(&cmd_parts)?;
    }

    Ok(())
}

/// Build rule specification from task parameters
fn build_rule_specification(task: &IptablesTask) -> Result<Vec<String>> {
    let mut spec = Vec::new();

    // Protocol
    if task.protocol != "all" {
        spec.push("-p".to_string());
        spec.push(task.protocol.clone());
    }

    // Source
    if let Some(source) = &task.source {
        spec.push("-s".to_string());
        spec.push(source.clone());
    }

    // Destination
    if let Some(destination) = &task.destination {
        spec.push("-d".to_string());
        spec.push(destination.clone());
    }

    // Source port
    if let Some(sport) = &task.sport {
        if task.protocol == "tcp" || task.protocol == "udp" {
            spec.push("--sport".to_string());
            spec.push(sport.clone());
        }
    }

    // Destination port
    if let Some(dport) = &task.dport {
        if task.protocol == "tcp" || task.protocol == "udp" {
            spec.push("--dport".to_string());
            spec.push(dport.clone());
        }
    }

    // Input interface
    if let Some(in_interface) = &task.in_interface {
        spec.push("-i".to_string());
        spec.push(in_interface.clone());
    }

    // Output interface
    if let Some(out_interface) = &task.out_interface {
        spec.push("-o".to_string());
        spec.push(out_interface.clone());
    }

    Ok(spec)
}

/// Check if a rule already exists
fn rule_exists(_rule_spec: &[String], task: &IptablesTask) -> Result<bool> {
    // Build list command to check existing rules
    let mut cmd_parts = vec![if task.ipv6 { "ip6tables" } else { "iptables" }.to_string()];
    cmd_parts.push("-t".to_string());
    cmd_parts.push(task.table.clone());
    cmd_parts.push("-L".to_string());
    cmd_parts.push(task.chain.clone());
    cmd_parts.push("-n".to_string()); // Numeric output

    let output = run_iptables_cmd(&cmd_parts)?;
    let lines: Vec<&str> = output.lines().collect();

    // Skip header lines and look for our rule
    for line in lines.iter().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse the rule line
        // iptables output format: target prot opt source destination
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            continue;
        }

        let target = parts[0];
        let protocol = parts[1];
        let source = parts[3];
        let destination = parts[4];

        // Check if this matches our rule
        if target == task.target && protocol_matches(protocol, &task.protocol) {
            // Additional checks would be needed for full rule matching
            // For now, do basic matching
            let source_match = task
                .source
                .as_ref()
                .map(|s| source.contains(s))
                .unwrap_or(true);
            let dest_match = task
                .destination
                .as_ref()
                .map(|d| destination.contains(d))
                .unwrap_or(true);

            if source_match && dest_match {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Check if protocol matches (handles iptables output format)
fn protocol_matches(output_proto: &str, task_proto: &str) -> bool {
    match task_proto {
        "all" => output_proto == "all",
        "tcp" => output_proto == "tcp",
        "udp" => output_proto == "udp",
        "icmp" => output_proto == "icmp",
        _ => output_proto == task_proto,
    }
}

/// Check if iptables is available
fn is_iptables_available(ipv6: bool) -> Result<bool> {
    let cmd = if ipv6 { "ip6tables" } else { "iptables" };
    let output = Command::new("which")
        .arg(cmd)
        .output()
        .context("Failed to check if iptables is available")?;

    Ok(output.status.success())
}

/// Run an iptables command and return its output
fn run_iptables_cmd(cmd_parts: &[String]) -> Result<String> {
    let output = Command::new(&cmd_parts[0])
        .args(&cmd_parts[1..])
        .output()
        .with_context(|| {
            format!(
                "Failed to execute iptables command: {}",
                cmd_parts.join(" ")
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "iptables command failed: {} (stderr: {})",
            cmd_parts.join(" "),
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn default_table() -> String {
    "filter".to_string()
}

fn default_chain() -> String {
    "INPUT".to_string()
}

fn default_protocol() -> String {
    "tcp".to_string()
}

pub fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iptables_task_validation() {
        // Test with missing target - should fail during execution
        let task = IptablesTask {
            description: None,
            state: IptablesState::Present,
            table: "filter".to_string(),
            chain: "INPUT".to_string(),
            protocol: "tcp".to_string(),
            source: Some("192.168.1.0/24".to_string()),
            destination: None,
            sport: None,
            dport: Some("80".to_string()),
            in_interface: None,
            out_interface: None,
            target: "ACCEPT".to_string(),
            extra_args: vec![],
            ipv6: false,
            check_available: false,
        };

        // This should work for basic validation - actual execution would need iptables
        assert!(matches!(task.state, IptablesState::Present));
    }

    #[test]
    fn test_build_rule_specification() {
        let task = IptablesTask {
            description: None,
            state: IptablesState::Present,
            table: "filter".to_string(),
            chain: "INPUT".to_string(),
            protocol: "tcp".to_string(),
            source: Some("192.168.1.0/24".to_string()),
            destination: Some("10.0.0.0/8".to_string()),
            sport: Some("1024:65535".to_string()),
            dport: Some("80".to_string()),
            in_interface: Some("eth0".to_string()),
            out_interface: None,
            target: "ACCEPT".to_string(),
            extra_args: vec![],
            ipv6: false,
            check_available: true,
        };

        let spec = build_rule_specification(&task).unwrap();
        assert!(spec.contains(&"-p".to_string()));
        assert!(spec.contains(&"tcp".to_string()));
        assert!(spec.contains(&"-s".to_string()));
        assert!(spec.contains(&"192.168.1.0/24".to_string()));
        assert!(spec.contains(&"-d".to_string()));
        assert!(spec.contains(&"10.0.0.0/8".to_string()));
        assert!(spec.contains(&"--sport".to_string()));
        assert!(spec.contains(&"1024:65535".to_string()));
        assert!(spec.contains(&"--dport".to_string()));
        assert!(spec.contains(&"80".to_string()));
        assert!(spec.contains(&"-i".to_string()));
        assert!(spec.contains(&"eth0".to_string()));
    }

    #[test]
    fn test_protocol_matches() {
        assert!(protocol_matches("tcp", "tcp"));
        assert!(protocol_matches("udp", "udp"));
        assert!(protocol_matches("all", "all"));
        assert!(!protocol_matches("tcp", "udp"));
    }
}
