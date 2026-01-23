//! Include role functionality
//!
//! This module provides functionality to include roles into the current
//! configuration. Roles are reusable collections of tasks with their
//! own variables and defaults.
//!
//! # Examples
//!
//! ## Include a role
//!
//! This example includes a role for web server setup.
//!
//! **YAML Format:**
//! ```yaml
//! - type: include_role
//!   description: "Setup web server"
//!   name: webserver
//!   when: "webserver_required"
//!   vars:
//!     port: 8080
//!   defaults:
//!     document_root: /var/www/html
//! ```
//!
//! **JSON Format:**
//! ```json
//! [
//!   {
//!     "type": "include_role",
//!     "description": "Setup web server",
//!     "name": "webserver",
//!     "when": "webserver_required",
//!     "vars": {
//!       "port": 8080
//!     },
//!     "defaults": {
//!       "document_root": "/var/www/html"
//!     }
//!   }
//! ]
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "include_role"
//! description = "Setup web server"
//! name = "webserver"
//! when = "webserver_required"
//!
//! [tasks.vars]
//! port = 8080
//!
//! [tasks.defaults]
//! document_root = "/var/www/html"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Include role task for reusable configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeRoleTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Role name
    ///
    /// Name of the role to include.
    pub name: String,

    /// Variable overrides
    ///
    /// Variables to pass to the role.
    #[serde(default)]
    pub vars: HashMap<String, serde_json::Value>,

    /// Default variables
    ///
    /// Default variables for the role.
    #[serde(default)]
    pub defaults: HashMap<String, serde_json::Value>,
}
