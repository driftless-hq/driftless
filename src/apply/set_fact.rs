//! Set fact task for variable management
//!
//! This module provides functionality to set facts (variables) that
//! can be used throughout the configuration. Facts can be cached
//! between runs for performance.
//!
//! # Examples
//!
//! ## Set a fact
//!
//! This example sets a fact that can be used by other tasks.
//!
//! **YAML Format:**
//! ```yaml
//! - type: set_fact
//!   description: "Set application version"
//!   key: app_version
//!   value: "1.2.3"
//!   cacheable: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "set_fact",
//!     "description": "Set application version",
//!     "key": "app_version",
//!     "value": "1.2.3",
//!     "cacheable": true
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "set_fact"
//! description = "Set application version"
//! key = "app_version"
//! value = "1.2.3"
//! cacheable = true
//! ```

use serde::{Deserialize, Serialize};

/// Set fact task for variable management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFactTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Variable name
    ///
    /// Name of the fact/variable to set.
    pub key: String,

    /// Variable value
    ///
    /// Value to assign to the variable.
    pub value: serde_yaml::Value,

    /// Cacheable flag
    ///
    /// Whether this fact can be cached between runs.
    #[serde(default)]
    pub cacheable: bool,
}
