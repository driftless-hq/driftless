//! Debug task for displaying information
//!
//! This module provides debugging functionality to display messages
//! and variable values during task execution. Useful for troubleshooting
//! and monitoring task progress.
//!
//! # Examples
//!
//! ## Display a debug message
//!
//! This example displays a debug message.
//!
//! **YAML Format:**
//! ```yaml
//! - type: debug
//!   description: "Show current configuration"
//!   msg: "Current web_root: {{ web_root }}"
//!   verbosity: normal
//! ```

use serde::{Deserialize, Serialize};

/// Debug verbosity levels
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum DebugVerbosity {
    /// Show in normal output
    #[default]
    Normal,
    /// Show only in verbose mode
    Verbose,
}

/// Debug task for displaying information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Message to display
    ///
    /// The message to print. Can be a string or variable reference.
    pub msg: String,

    /// Variable to debug
    ///
    /// Variable name to display value of. Alternative to msg.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub var: Option<String>,

    /// Verbosity level
    ///
    /// Control when this debug message is shown (normal/verbose).
    #[serde(default)]
    pub verbosity: DebugVerbosity,
}