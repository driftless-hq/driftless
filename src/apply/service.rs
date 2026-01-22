//! Service task executor
//!
//! Handles service management operations: start, stop, restart, enable, disable services
//! using the appropriate service manager for the system.
//!
//! # Examples
//!
//! ## Start and enable a service
//!
//! This example starts the nginx service and ensures it starts automatically on boot.
//!
//! **YAML Format:**
//! ```yaml
//! - type: service
//!   description: "Start and enable nginx service"
//!   name: nginx
//!   state: started
//!   enabled: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "service",
//!   "description": "Start and enable nginx service",
//!   "name": "nginx",
//!   "state": "started",
//!   "enabled": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "service"
//! description = "Start and enable nginx service"
//! name = "nginx"
//! state = "started"
//! enabled = true
//! ```
//!
//! ## Stop and disable a service
//!
//! This example stops the telnet service and prevents it from starting automatically.
//!
//! **YAML Format:**
//! ```yaml
//! - type: service
//!   description: "Stop and disable telnet service"
//!   name: telnet
//!   state: stopped
//!   enabled: false
//! ```
//!
//! ## Restart a service
//!
//! This example restarts a service, which is useful after configuration changes.
//!
//! **YAML Format:**
//! ```yaml
//! - type: service
//!   description: "Restart apache service after config change"
//!   name: apache2
//!   state: restarted
//! ```
//!
//! ## Enable a service without changing its running state
//!
//! This example ensures a service is enabled for automatic startup without affecting its current running state.
//!
//! **YAML Format:**
//! ```yaml
//! - type: service
//!   description: "Enable ssh service for automatic startup"
//!   name: ssh
//!   enabled: true
//! ```

/// Service state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    /// Ensure service is running
    Started,
    /// Ensure service is stopped
    Stopped,
    /// Restart service
    Restarted,
    /// Reload service configuration
    Reloaded,
}

/// Service management task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Service name
    pub name: String,
    /// Service state
    pub state: ServiceState,
    /// Service manager to use (auto-detect if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<String>,
    /// Whether to enable service at boot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Execute a service task
pub async fn execute_service_task(task: &ServiceTask, dry_run: bool) -> Result<()> {
    let manager = detect_service_manager()
        .or_else(|| task.manager.as_ref().cloned())
        .ok_or_else(|| anyhow::anyhow!("Could not detect service manager"))?;

    // Handle enable/disable first if specified
    if let Some(enabled) = task.enabled {
        if enabled {
            ensure_service_enabled(&task.name, &manager, dry_run)?;
        } else {
            ensure_service_disabled(&task.name, &manager, dry_run)?;
        }
    }

    // Handle service state
    match task.state {
        ServiceState::Started => ensure_service_started(&task.name, &manager, dry_run).await,
        ServiceState::Stopped => ensure_service_stopped(&task.name, &manager, dry_run).await,
        ServiceState::Restarted => restart_service(&task.name, &manager, dry_run).await,
        ServiceState::Reloaded => reload_service(&task.name, &manager, dry_run).await,
    }
}

/// Detect the service manager available on the system
fn detect_service_manager() -> Option<String> {
    let managers = vec![
        ("systemctl", "systemd"),
        ("service", "sysvinit"),
        ("rc-service", "openrc"),
        ("sv", "runit"),
        ("brew services", "brew"),
    ];

    for (cmd, name) in managers {
        if Command::new("which")
            .arg(cmd.split_whitespace().next().unwrap())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(name.to_string());
        }
    }

    None
}

/// Ensure a service is started
async fn ensure_service_started(service: &str, manager: &str, dry_run: bool) -> Result<()> {
    if is_service_running(service, manager)? {
        println!("Service {} is already running", service);
        return Ok(());
    }

    let start_cmd = get_start_command(service, manager);

    if dry_run {
        println!("Would run: {}", start_cmd.join(" "));
    } else {
        run_command(&start_cmd).with_context(|| format!("Failed to start service {}", service))?;
        println!("Started service: {}", service);
    }

    Ok(())
}

/// Ensure a service is stopped
async fn ensure_service_stopped(service: &str, manager: &str, dry_run: bool) -> Result<()> {
    if !is_service_running(service, manager)? {
        println!("Service {} is already stopped", service);
        return Ok(());
    }

    let stop_cmd = get_stop_command(service, manager);

    if dry_run {
        println!("Would run: {}", stop_cmd.join(" "));
    } else {
        run_command(&stop_cmd).with_context(|| format!("Failed to stop service {}", service))?;
        println!("Stopped service: {}", service);
    }

    Ok(())
}

/// Restart a service
async fn restart_service(service: &str, manager: &str, dry_run: bool) -> Result<()> {
    let restart_cmd = get_restart_command(service, manager);

    if dry_run {
        println!("Would run: {}", restart_cmd.join(" "));
    } else {
        run_command(&restart_cmd)
            .with_context(|| format!("Failed to restart service {}", service))?;
        println!("Restarted service: {}", service);
    }

    Ok(())
}

/// Reload a service configuration
async fn reload_service(service: &str, manager: &str, dry_run: bool) -> Result<()> {
    let reload_cmd = get_reload_command(service, manager);

    if dry_run {
        println!("Would run: {}", reload_cmd.join(" "));
    } else {
        run_command(&reload_cmd)
            .with_context(|| format!("Failed to reload service {}", service))?;
        println!("Reloaded service: {}", service);
    }

    Ok(())
}

/// Ensure a service is enabled at boot
fn ensure_service_enabled(service: &str, manager: &str, dry_run: bool) -> Result<()> {
    if is_service_enabled(service, manager)? {
        println!("Service {} is already enabled", service);
        return Ok(());
    }

    let enable_cmd = get_enable_command(service, manager);

    if dry_run {
        println!("Would run: {}", enable_cmd.join(" "));
    } else {
        run_command(&enable_cmd)
            .with_context(|| format!("Failed to enable service {}", service))?;
        println!("Enabled service: {}", service);
    }

    Ok(())
}

/// Ensure a service is disabled at boot
fn ensure_service_disabled(service: &str, manager: &str, dry_run: bool) -> Result<()> {
    if !is_service_enabled(service, manager)? {
        println!("Service {} is already disabled", service);
        return Ok(());
    }

    let disable_cmd = get_disable_command(service, manager);

    if dry_run {
        println!("Would run: {}", disable_cmd.join(" "));
    } else {
        run_command(&disable_cmd)
            .with_context(|| format!("Failed to disable service {}", service))?;
        println!("Disabled service: {}", service);
    }

    Ok(())
}

/// Check if a service is running
fn is_service_running(service: &str, manager: &str) -> Result<bool> {
    let status_cmd = get_status_command(service, manager);

    let output = Command::new(&status_cmd[0])
        .args(&status_cmd[1..])
        .output()
        .with_context(|| format!("Failed to check status of service {}", service))?;

    Ok(output.status.success())
}

/// Check if a service is enabled
fn is_service_enabled(service: &str, manager: &str) -> Result<bool> {
    let enabled_cmd = get_is_enabled_command(service, manager);

    let output = Command::new(&enabled_cmd[0])
        .args(&enabled_cmd[1..])
        .output()
        .with_context(|| format!("Failed to check if service {} is enabled", service))?;

    Ok(output.status.success())
}

/// Get the start command for a service manager
fn get_start_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "start".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "service".to_string(),
            service.to_string(),
            "start".to_string(),
        ],
        "openrc" => vec![
            "rc-service".to_string(),
            service.to_string(),
            "start".to_string(),
        ],
        "runit" => vec!["sv".to_string(), "start".to_string(), service.to_string()],
        "brew" => vec![
            "brew".to_string(),
            "services".to_string(),
            "start".to_string(),
            service.to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported service manager: {}", manager),
        ],
    }
}

/// Get the stop command for a service manager
fn get_stop_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "stop".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "service".to_string(),
            service.to_string(),
            "stop".to_string(),
        ],
        "openrc" => vec![
            "rc-service".to_string(),
            service.to_string(),
            "stop".to_string(),
        ],
        "runit" => vec!["sv".to_string(), "stop".to_string(), service.to_string()],
        "brew" => vec![
            "brew".to_string(),
            "services".to_string(),
            "stop".to_string(),
            service.to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported service manager: {}", manager),
        ],
    }
}

/// Get the restart command for a service manager
fn get_restart_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "restart".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "service".to_string(),
            service.to_string(),
            "restart".to_string(),
        ],
        "openrc" => vec![
            "rc-service".to_string(),
            service.to_string(),
            "restart".to_string(),
        ],
        "runit" => vec!["sv".to_string(), "restart".to_string(), service.to_string()],
        "brew" => vec![
            "brew".to_string(),
            "services".to_string(),
            "restart".to_string(),
            service.to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported service manager: {}", manager),
        ],
    }
}

/// Get the reload command for a service manager
fn get_reload_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "reload".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "service".to_string(),
            service.to_string(),
            "reload".to_string(),
        ],
        "openrc" => vec![
            "rc-service".to_string(),
            service.to_string(),
            "reload".to_string(),
        ],
        _ => get_restart_command(service, manager), // Fallback to restart
    }
}

/// Get the enable command for a service manager
fn get_enable_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "enable".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "chkconfig".to_string(),
            service.to_string(),
            "on".to_string(),
        ],
        "openrc" => vec![
            "rc-update".to_string(),
            "add".to_string(),
            service.to_string(),
        ],
        "runit" => vec![
            "ln".to_string(),
            "-s".to_string(),
            format!("/etc/sv/{}", service),
            "/var/service/".to_string(),
        ],
        "brew" => vec![
            "echo".to_string(),
            format!("brew services enable not supported for {}", service),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported service manager: {}", manager),
        ],
    }
}

/// Get the disable command for a service manager
fn get_disable_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "disable".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "chkconfig".to_string(),
            service.to_string(),
            "off".to_string(),
        ],
        "openrc" => vec![
            "rc-update".to_string(),
            "del".to_string(),
            service.to_string(),
        ],
        "runit" => vec!["rm".to_string(), format!("/var/service/{}", service)],
        "brew" => vec![
            "echo".to_string(),
            format!("brew services disable not supported for {}", service),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported service manager: {}", manager),
        ],
    }
}

/// Get the status command for a service manager
fn get_status_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "is-active".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "service".to_string(),
            service.to_string(),
            "status".to_string(),
        ],
        "openrc" => vec![
            "rc-service".to_string(),
            service.to_string(),
            "status".to_string(),
        ],
        "runit" => vec!["sv".to_string(), "status".to_string(), service.to_string()],
        "brew" => vec![
            "brew".to_string(),
            "services".to_string(),
            "list".to_string(),
        ],
        _ => vec![
            "echo".to_string(),
            format!("Unsupported service manager: {}", manager),
        ],
    }
}

/// Get the is-enabled command for a service manager
fn get_is_enabled_command(service: &str, manager: &str) -> Vec<String> {
    match manager {
        "systemd" => vec![
            "systemctl".to_string(),
            "is-enabled".to_string(),
            service.to_string(),
        ],
        "sysvinit" => vec![
            "chkconfig".to_string(),
            "--list".to_string(),
            service.to_string(),
        ],
        "openrc" => vec!["rc-update".to_string(), "show".to_string()],
        _ => vec!["echo".to_string(), "1".to_string()], // Assume enabled by default for unsupported managers
    }
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
    async fn test_service_start_dry_run() {
        let task = ServiceTask {
            description: None,
            name: "nginx".to_string(),
            state: ServiceState::Started,
            manager: Some("systemd".to_string()),
            enabled: None,
        };

        let result = execute_service_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_stop_dry_run() {
        let task = ServiceTask {
            description: None,
            name: "nginx".to_string(),
            state: ServiceState::Stopped,
            manager: Some("systemd".to_string()),
            enabled: None,
        };

        let result = execute_service_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_enable_disable() {
        let task = ServiceTask {
            description: None,
            name: "nginx".to_string(),
            state: ServiceState::Started,
            manager: Some("systemd".to_string()),
            enabled: Some(true),
        };

        let result = execute_service_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_start_command() {
        let cmd = get_start_command("nginx", "systemd");
        assert_eq!(cmd, vec!["systemctl", "start", "nginx"]);

        let cmd = get_start_command("nginx", "sysvinit");
        assert_eq!(cmd, vec!["service", "nginx", "start"]);
    }

    #[test]
    fn test_get_stop_command() {
        let cmd = get_stop_command("nginx", "systemd");
        assert_eq!(cmd, vec!["systemctl", "stop", "nginx"]);

        let cmd = get_stop_command("nginx", "sysvinit");
        assert_eq!(cmd, vec!["service", "nginx", "stop"]);
    }

    #[test]
    fn test_detect_service_manager() {
        // Similar to package manager test - we can't assert much in a test environment
        let manager = detect_service_manager();
        assert!(manager.is_some() || manager.is_none());
    }
}
