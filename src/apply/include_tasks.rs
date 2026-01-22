//! Include tasks functionality
//!
//! This module provides functionality to include external task files
//! into the current configuration. Allows for modular configuration
//! and code reuse.
//!
//! # Examples
//!
//! ## Include a task file
//!
//! This example includes tasks from an external file.
//!
//! **YAML Format:**
//! ```yaml
//! - type: include_tasks
//!   description: "Include common setup tasks"
//!   file: common/setup.yml
//!   when: "setup_required"
//!   vars:
//!     app_name: myapp
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Include tasks task for modular configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeTasksTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// File to include
    ///
    /// Path to the task file to include.
    pub file: String,

    /// Variable overrides
    ///
    /// Variables to pass to the included tasks.
    #[serde(default)]
    pub vars: HashMap<String, serde_json::Value>,
}