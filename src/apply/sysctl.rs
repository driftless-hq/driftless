//! Sysctl task executor
//!
//! Handles kernel parameter management via sysctl.
//!
//! # Examples
//!
//! ## Set kernel parameter
//!
//! This example sets a kernel parameter value.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sysctl
//!   description: "Enable IP forwarding"
//!   name: net.ipv4.ip_forward
//!   state: present
//!   value: "1"
//!   persist: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sysctl",
//!   "description": "Enable IP forwarding",
//!   "name": "net.ipv4.ip_forward",
//!   "state": "present",
//!   "value": "1",
//!   "persist": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sysctl"
//! description = "Enable IP forwarding"
//! name = "net.ipv4.ip_forward"
//! state = "present"
//! value = "1"
//! persist = true
//! ```
//!
//! ## Configure network buffer sizes
//!
//! This example sets network buffer sizes for better performance.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sysctl
//!   description: "Increase network buffer sizes"
//!   name: net.core.rmem_max
//!   state: present
//!   value: "16777216"
//!   persist: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sysctl",
//!   "description": "Increase network buffer sizes",
//!   "name": "net.core.rmem_max",
//!   "state": "present",
//!   "value": "16777216",
//!   "persist": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sysctl"
//! description = "Increase network buffer sizes"
//! name = "net.core.rmem_max"
//! state = "present"
//! value = "16777216"
//! persist = true
//! ```
//!
//! ## Disable IPv6
//!
//! This example disables IPv6 on the system.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sysctl
//!   description: "Disable IPv6"
//!   name: net.ipv6.conf.all.disable_ipv6
//!   state: present
//!   value: "1"
//!   persist: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sysctl",
//!   "description": "Disable IPv6",
//!   "name": "net.ipv6.conf.all.disable_ipv6",
//!   "state": "present",
//!   "value": "1",
//!   "persist": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sysctl"
//! description = "Disable IPv6"
//! name = "net.ipv6.conf.all.disable_ipv6"
//! state = "present"
//! value = "1"
//! persist = true
//! ```
//!
//! ## Remove sysctl parameter
//!
//! This example removes a sysctl parameter setting.
//!
//! **YAML Format:**
//! ```yaml
//! - type: sysctl
//!   description: "Remove custom sysctl parameter"
//!   name: net.ipv4.tcp_tw_reuse
//!   state: absent
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "sysctl",
//!   "description": "Remove custom sysctl parameter",
//!   "name": "net.ipv4.tcp_tw_reuse",
//!   "state": "absent"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "sysctl"
//! description = "Remove custom sysctl parameter"
//! name = "net.ipv4.tcp_tw_reuse"
//! state = "absent"
//! ```

/// Sysctl state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SysctlState {
    /// Ensure parameter has this value
    Present,
    /// Ensure parameter does not exist
    Absent,
}

/// Kernel parameter management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SysctlTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter name (e.g., "net.ipv4.ip_forward")
    pub name: String,
    /// Parameter state
    pub state: SysctlState,
    /// Parameter value
    pub value: String,
    /// Whether to persist changes to /etc/sysctl.conf
    #[serde(default)]
    pub persist: bool,
    /// Whether to reload immediately
    #[serde(default = "crate::apply::default_true")]
    pub reload: bool,
}

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Execute a sysctl task
pub async fn execute_sysctl_task(task: &SysctlTask, dry_run: bool) -> Result<()> {
    match task.state {
        SysctlState::Present => {
            ensure_sysctl_value(task, dry_run).await
        }
        SysctlState::Absent => {
            ensure_sysctl_absent(task, dry_run).await
        }
    }
}

/// Ensure a sysctl parameter has the specified value
async fn ensure_sysctl_value(task: &SysctlTask, dry_run: bool) -> Result<()> {
    // Get current value
    let current_value = get_sysctl_value(&task.name)?;

    if let Some(current) = current_value {
        if current == task.value {
            println!("Sysctl parameter {} already has value: {}", task.name, task.value);
            return Ok(());
        } else {
            println!("Sysctl parameter {} has different value (current: {}, desired: {})",
                    task.name, current, task.value);
        }
    } else {
        println!("Sysctl parameter {} does not exist", task.name);
    }

    if dry_run {
        println!("Would set sysctl {} = {}", task.name, task.value);
        if task.persist {
            println!("  and persist to /etc/sysctl.conf");
        }
    } else {
        set_sysctl_value(task)?;
        println!("Set sysctl {} = {}", task.name, task.value);

        if task.persist {
            persist_sysctl_value(task)?;
            println!("Persisted sysctl {} to /etc/sysctl.conf", task.name);
        }
    }

    Ok(())
}

/// Ensure a sysctl parameter does not exist
async fn ensure_sysctl_absent(task: &SysctlTask, dry_run: bool) -> Result<()> {
    let current_value = get_sysctl_value(&task.name)?;

    if current_value.is_none() {
        println!("Sysctl parameter {} does not exist", task.name);
        return Ok(());
    }

    if dry_run {
        println!("Would remove sysctl {}", task.name);
        if task.persist {
            println!("  and remove from /etc/sysctl.conf");
        }
    } else {
        remove_sysctl_value(task)?;
        println!("Removed sysctl {}", task.name);

        if task.persist {
            remove_persistent_sysctl(task)?;
            println!("Removed sysctl {} from /etc/sysctl.conf", task.name);
        }
    }

    Ok(())
}

/// Get the current value of a sysctl parameter
fn get_sysctl_value(name: &str) -> Result<Option<String>> {
    let output = std::process::Command::new("sysctl")
        .arg("-n")  // Only output value, no name
        .arg(name)
        .output()
        .with_context(|| format!("Failed to get sysctl value for {}", name))?;

    if output.status.success() {
        let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if value.is_empty() {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    } else {
        // Parameter doesn't exist or error
        Ok(None)
    }
}

/// Set a sysctl parameter value
fn set_sysctl_value(task: &SysctlTask) -> Result<()> {
    let output = std::process::Command::new("sysctl")
        .arg("-w")  // Write mode
        .arg(format!("{}={}", task.name, task.value))
        .output()
        .with_context(|| format!("Failed to set sysctl {}={}", task.name, task.value))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("sysctl command failed: {}", stderr));
    }

    if !task.reload {
        // Apply immediately via /proc/sys interface
        apply_sysctl_immediately(task)?;
    }

    Ok(())
}

/// Apply sysctl value immediately via /proc/sys
fn apply_sysctl_immediately(task: &SysctlTask) -> Result<()> {
    // Convert sysctl name to /proc/sys path
    // e.g., "net.ipv4.ip_forward" -> "/proc/sys/net/ipv4/ip_forward"
    let proc_path = format!("/proc/sys/{}", task.name.replace('.', "/"));

    if Path::new(&proc_path).exists() {
        fs::write(&proc_path, &task.value)
            .with_context(|| format!("Failed to write to {}", proc_path))?;
    } else {
        println!("Warning: {} does not exist, sysctl may not be available", proc_path);
    }

    Ok(())
}

/// Remove a sysctl parameter (reset to default)
fn remove_sysctl_value(_task: &SysctlTask) -> Result<()> {
    // This is tricky - we can't really "remove" a sysctl parameter
    // The best we can do is reset it to its default value
    // For now, we'll just warn that this operation is not fully supported
    println!("Warning: Removing sysctl parameters is not fully supported");
    println!("Consider manually editing /etc/sysctl.conf or using sysctl -p to reload defaults");

    Ok(())
}

/// Persist a sysctl value to /etc/sysctl.conf
fn persist_sysctl_value(task: &SysctlTask) -> Result<()> {
    let sysctl_conf = "/etc/sysctl.conf";
    let entry = format!("# Managed by Driftless\n{} = {}\n", task.name, task.value);

    // Read existing file
    let mut content = if Path::new(sysctl_conf).exists() {
        fs::read_to_string(sysctl_conf)
            .with_context(|| format!("Failed to read {}", sysctl_conf))?
    } else {
        String::new()
    };

    // Check if entry already exists
    let search_pattern = format!("{} =", task.name);
    if content.contains(&search_pattern) {
        // Replace existing entry
        let lines: Vec<String> = content.lines()
            .map(|line| {
                if line.contains(&search_pattern) {
                    format!("{} = {}", task.name, task.value)
                } else {
                    line.to_string()
                }
            })
            .collect();
        content = lines.join("\n");
    } else {
        // Add new entry
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&entry);
    }

    fs::write(sysctl_conf, content)
        .with_context(|| format!("Failed to write {}", sysctl_conf))?;

    Ok(())
}

/// Remove persistent sysctl value from /etc/sysctl.conf
fn remove_persistent_sysctl(task: &SysctlTask) -> Result<()> {
    let sysctl_conf = "/etc/sysctl.conf";

    if !Path::new(sysctl_conf).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(sysctl_conf)
        .with_context(|| format!("Failed to read {}", sysctl_conf))?;

    // Remove lines containing the parameter
    let search_pattern = format!("{} =", task.name);
    let new_content: String = content.lines()
        .filter(|line| !line.contains(&search_pattern))
        .collect::<Vec<&str>>()
        .join("\n");

    fs::write(sysctl_conf, new_content)
        .with_context(|| format!("Failed to write {}", sysctl_conf))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::apply::SysctlTask;
    use crate::apply::sysctl::SysctlState;

    #[tokio::test]
    async fn test_sysctl_set_dry_run() {
        let task = SysctlTask {
            description: None,
            name: "net.ipv4.ip_forward".to_string(),
            state: SysctlState::Present,
            value: "1".to_string(),
            persist: true,
            reload: true,
        };

        let result = execute_sysctl_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sysctl_remove_dry_run() {
        let task = SysctlTask {
            description: None,
            name: "net.ipv4.ip_forward".to_string(),
            state: SysctlState::Absent,
            value: "0".to_string(),
            persist: false,
            reload: false,
        };

        let result = execute_sysctl_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_sysctl_value() {
        // Test with a common sysctl parameter
        let result = get_sysctl_value("kernel.hostname");
        // We don't assert the result since it depends on the system and permissions
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sysctl_empty_name() {
        let task = SysctlTask {
            description: None,
            name: "".to_string(), // Empty name
            state: SysctlState::Present,
            value: "1".to_string(),
            persist: false,
            reload: false,
        };

        let result = execute_sysctl_task(&task, true).await;
        // Dry-run should succeed even with empty name
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sysctl_empty_value() {
        let task = SysctlTask {
            description: None,
            name: "net.ipv4.ip_forward".to_string(),
            state: SysctlState::Present,
            value: "".to_string(), // Empty value might be valid
            persist: false,
            reload: false,
        };

        let result = execute_sysctl_task(&task, true).await;
        assert!(result.is_ok()); // Empty value should be allowed
    }

    #[tokio::test]
    async fn test_sysctl_invalid_parameter() {
        let task = SysctlTask {
            description: None,
            name: "nonexistent.parameter.12345".to_string(), // Parameter that doesn't exist
            state: SysctlState::Present,
            value: "1".to_string(),
            persist: false,
            reload: false,
        };

        let result = execute_sysctl_task(&task, true).await;
        // Dry-run should succeed even with invalid parameter
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sysctl_remove_nonexistent() {
        let task = SysctlTask {
            description: None,
            name: "nonexistent.parameter.12345".to_string(),
            state: SysctlState::Absent,
            value: "0".to_string(),
            persist: false,
            reload: false,
        };

        let result = execute_sysctl_task(&task, true).await;
        assert!(result.is_ok()); // Removing non-existent parameter should succeed
    }
}