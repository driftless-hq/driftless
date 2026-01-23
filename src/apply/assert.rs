//! Assert task for validating conditions
//!
//! This module provides assertion functionality to validate conditions
//! during task execution. Assertions can be used to verify that certain
//! conditions are met before proceeding with subsequent tasks.
//!
//! # Examples
//!
//! ## Assert a condition
//!
//! This example checks that a variable is set to a specific value.
//!
//! **YAML Format:**
//! ```yaml
//! - type: assert
//!   description: "Verify nginx is installed"
//!   that: "'nginx' in installed_packages"
//!   success_msg: "Nginx is properly installed"
//!   fail_msg: "Nginx installation failed"
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "assert",
//!     "description": "Verify nginx is installed",
//!     "that": "'nginx' in installed_packages",
//!     "success_msg": "Nginx is properly installed",
//!     "fail_msg": "Nginx installation failed"
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "assert"
//! description = "Verify nginx is installed"
//! that = "'nginx' in installed_packages"
//! success_msg = "Nginx is properly installed"
//! fail_msg = "Nginx installation failed"
//! ```

use serde::{Deserialize, Serialize};

/// Assert task for validating conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Condition to assert
    ///
    /// Boolean expression that must evaluate to true.
    pub that: String,

    /// Success message
    ///
    /// Message to display when assertion passes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_msg: Option<String>,

    /// Failure message
    ///
    /// Message to display when assertion fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_msg: Option<String>,

    /// Quiet mode
    ///
    /// Don't show success messages.
    #[serde(default)]
    pub quiet: bool,
}
