//! Fail task for forcing execution failure
//!
//! This module provides functionality to force task execution failure
//! with custom error messages. Useful for validation and error handling.
//!
//! # Examples
//!
//! ## Fail with a message
//!
//! This example fails execution with a custom message.
//!
//! **YAML Format:**
//! ```yaml
//! - type: fail
//!   description: "Stop execution if requirements not met"
//!   msg: "System requirements not satisfied"
//!   when: "not requirements_met"
//! ```

use serde::{Deserialize, Serialize};

/// Fail task for forcing execution failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Failure message
    ///
    /// Message to display when failing.
    pub msg: String,

    /// When condition
    ///
    /// Only fail when this condition is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}