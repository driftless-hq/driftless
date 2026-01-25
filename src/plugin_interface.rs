//! Plugin Interface Definition
//!
//! This module defines the interface that plugins must implement to register
//! custom task types and template extensions with Driftless.
//!
//! Plugins are WebAssembly modules that export specific functions for
//! registration and execution.

use serde::{Deserialize, Serialize};

/// Task types that plugins can register
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Apply,
    Facts,
    Logs,
}

/// Definition of a custom task provided by a plugin
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TaskDefinition {
    pub name: String,
    pub task_type: TaskType,
    pub config_schema: serde_json::Value, // JSON Schema for validation
}

/// Template filter or function provided by a plugin
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateExtension {
    /// Extension name
    pub name: String,
    /// Type of extension
    pub extension_type: TemplateExtensionType,
    /// JSON Schema for configuration validation
    #[serde(rename = "config_schema")]
    pub config_schema: serde_json::Value,
    /// Human-readable description
    pub description: String,
    /// Category for organization
    pub category: String,
    /// Arguments specification as (name, description) pairs
    pub arguments: Vec<(String, String)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateExtensionType {
    Filter,
    Function,
}

/// Plugin interface functions that WASM modules must export
///
/// All functions use JSON strings for data exchange to enable cross-language
/// compatibility and avoid direct type coupling.
#[allow(dead_code)]
pub mod plugin_exports {
    /// Returns a JSON array of task definitions
    ///
    /// Format: [{"name": "string", "type": "apply|facts|logs", "config_schema": {...}}]
    pub const GET_TASK_DEFINITIONS: &str = "get_task_definitions";

    /// Returns a JSON array of facts collector definitions
    ///
    /// Format: [{"name": "string", "config_schema": {...}}]
    pub const GET_FACTS_COLLECTORS: &str = "get_facts_collectors";

    /// Executes a registered facts collector
    ///
    /// Args:
    /// - name: collector name as string
    /// - config: JSON string of collector configuration
    ///
    /// Returns: JSON string of collected facts or error
    pub const EXECUTE_FACTS_COLLECTOR: &str = "execute_facts_collector";

    /// Executes a registered task
    ///
    /// Args:
    /// - name: task name as string
    /// - config: JSON string of task configuration
    ///
    /// Returns: JSON string of execution result or error
    pub const EXECUTE_TASK: &str = "execute_task";

    /// Returns a JSON array of template extensions
    ///
    /// Format: [{"name": "string", "type": "filter|function"}]
    pub const GET_TEMPLATE_EXTENSIONS: &str = "get_template_extensions";

    /// Returns a JSON array of log source definitions
    ///
    /// Format: [{"name": "string", "config_schema": {...}}]
    pub const GET_LOG_SOURCES: &str = "get_log_sources";

    /// Returns a JSON array of log parser definitions
    ///
    /// Format: [{"name": "string", "config_schema": {...}}]
    pub const GET_LOG_PARSERS: &str = "get_log_parsers";

    /// Returns a JSON array of log filter definitions
    ///
    /// Format: [{"name": "string", "config_schema": {...}}]
    pub const GET_LOG_FILTERS: &str = "get_log_filters";

    /// Returns a JSON array of log output definitions
    ///
    /// Format: [{"name": "string", "config_schema": {...}}]
    pub const GET_LOG_OUTPUTS: &str = "get_log_outputs";

    /// Executes a template filter
    ///
    /// Args:
    /// - name: filter name
    /// - input: JSON string of input value
    /// - args: JSON array of additional arguments
    ///
    /// Returns: JSON string of filtered result
    pub const EXECUTE_FILTER: &str = "execute_filter";

    /// Executes a template function
    ///
    /// Args:
    /// - name: function name
    /// - args: JSON array of arguments
    ///
    /// Returns: JSON string of function result
    pub const EXECUTE_FUNCTION: &str = "execute_function";

    /// Executes a registered log source
    ///
    /// Args:
    /// - name: source name as string
    /// - config: JSON string of source configuration
    ///
    /// Returns: JSON string of log data or error
    pub const EXECUTE_LOG_SOURCE: &str = "execute_log_source";

    /// Executes a registered log parser
    ///
    /// Args:
    /// - name: parser name as string
    /// - config: JSON string of parser configuration
    /// - input: raw log line as string
    ///
    /// Returns: JSON string of parsed log entry or error
    pub const EXECUTE_LOG_PARSER: &str = "execute_log_parser";

    /// Executes a registered log filter
    ///
    /// Args:
    /// - name: filter name as string
    /// - config: JSON string of filter configuration
    /// - entry: JSON string of log entry
    ///
    /// Returns: JSON boolean indicating if entry passes filter
    pub const EXECUTE_LOG_FILTER: &str = "execute_log_filter";

    /// Executes a registered log output
    ///
    /// Args:
    /// - name: output name as string
    /// - config: JSON string of output configuration
    /// - entry: JSON string of log entry
    ///
    /// Returns: JSON string of execution result or error
    pub const EXECUTE_LOG_OUTPUT: &str = "execute_log_output";

    /// Executes a registered template filter
    ///
    /// Args:
    /// - name: filter name as string
    /// - config: JSON string of filter configuration
    /// - value: JSON string of value to filter
    /// - args: JSON array of additional arguments
    ///
    /// Returns: JSON string of filtered result
    pub const EXECUTE_TEMPLATE_FILTER: &str = "execute_template_filter";

    /// Executes a registered template function
    ///
    /// Args:
    /// - name: function name as string
    /// - config: JSON string of function configuration
    /// - args: JSON array of function arguments
    ///
    /// Returns: JSON string of function result
    pub const EXECUTE_TEMPLATE_FUNCTION: &str = "execute_template_function";
}

/// Host interface functions that plugins can import
///
/// These allow plugins to interact with the host environment in a controlled way.
///
/// **Note:** These host functions are currently documented but not yet wired into
/// the WASM Linker. Plugins attempting to import these functions will encounter
/// missing-import errors until the host provides these functions via the Linker.
#[allow(dead_code)]
pub mod host_imports {
    /// Log a message from the plugin
    ///
    /// Args:
    /// - level: "error" | "warn" | "info" | "debug"
    /// - message: string
    pub const LOG: &str = "host_log";

    /// Get current timestamp
    ///
    /// Returns: Unix timestamp as u64
    pub const GET_TIMESTAMP: &str = "host_get_timestamp";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_definition_creation() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            }
        });

        let def = TaskDefinition {
            name: "custom_copy".to_string(),
            task_type: TaskType::Apply,
            config_schema: schema,
        };

        assert_eq!(def.name, "custom_copy");
        assert_eq!(def.task_type, TaskType::Apply);
    }
}
