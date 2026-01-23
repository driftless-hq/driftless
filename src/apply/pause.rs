//! Pause task to halt execution
//!
//! This module provides functionality to pause task execution for
//! a specified duration. Useful for waiting between operations or
//! allowing manual intervention.
//!
//! # Examples
//!
//! ## Pause execution
//!
//! This example pauses execution for 30 seconds.
//!
//! **YAML Format:**
//! ```yaml
//! - type: pause
//!   description: "Wait for services to start"
//!   prompt: "Waiting for services to initialize..."
//!   seconds: 30
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "pause",
//!     "description": "Wait for services to start",
//!     "prompt": "Waiting for services to initialize...",
//!     "seconds": 30
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "pause"
//! description = "Wait for services to start"
//! prompt = "Waiting for services to initialize..."
//! seconds = 30
//! ```

use serde::{Deserialize, Serialize};

/// Default pause message
fn default_pause_message() -> String {
    "Press enter to continue...".to_string()
}

/// Pause task to halt execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PauseTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Message to display during pause
    ///
    /// Message shown to user during pause.
    #[serde(default = "default_pause_message")]
    pub prompt: String,

    /// Seconds to pause
    ///
    /// Duration to pause execution in seconds.
    #[serde(default)]
    pub seconds: u64,

    /// Minutes to pause
    ///
    /// Duration to pause execution in minutes.
    #[serde(default)]
    pub minutes: u64,
}
