//! Wait for task for synchronization
//!
//! This module provides functionality to wait for various conditions
//! such as network connectivity, file existence, or service availability.
//! Useful for ensuring dependencies are ready before proceeding.
//!
//! # Examples
//!
//! ## Wait for port connectivity
//!
//! This example waits for a service to become available on port 80.
//!
//! **YAML Format:**
//! ```yaml
//! - type: wait_for
//!   description: "Wait for web server to start"
//!   host: localhost
//!   port: 80
//!   timeout: 60
//!   delay: 5
//! ```

use serde::{Deserialize, Serialize};

/// Default wait timeout in seconds
fn default_wait_timeout() -> u64 {
    300
}

/// Default delay between checks in seconds
fn default_wait_delay() -> u64 {
    1
}

/// Connection state for wait conditions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ConnectionState {
    /// Wait for connection to be established
    #[default]
    Started,
    /// Wait for connection to be stopped
    Stopped,
}

/// Wait for task for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitForTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Host to wait for connectivity
    ///
    /// Hostname or IP address to check connectivity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Port to check
    ///
    /// Port number to check for connectivity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,

    /// Path to file to wait for
    ///
    /// File path to wait for existence or non-existence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Timeout in seconds
    ///
    /// Maximum time to wait for condition.
    #[serde(default = "default_wait_timeout")]
    pub timeout: u64,

    /// Delay between checks
    ///
    /// Time to wait between connectivity checks.
    #[serde(default = "default_wait_delay")]
    pub delay: u64,

    /// Connection state to wait for
    ///
    /// Whether to wait for connection to be started or stopped.
    #[serde(default)]
    pub state: ConnectionState,

    /// Active connection check
    ///
    /// Perform active connection attempt instead of just port scan.
    #[serde(default)]
    pub active_connection: bool,
}