//! Plugin Interface Definition
//!
//! This module defines the interface that plugins must implement to register
//! custom task types and template extensions with Driftless.
//!
//! Plugins are WebAssembly modules that export specific functions for
//! registration and execution.

/// Task types that plugins can register
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    Apply,
    Facts,
    Logs,
}

/// Definition of a custom task provided by a plugin
#[derive(Debug, Clone)]
pub struct TaskDefinition {
    pub name: String,
    pub task_type: TaskType,
    pub config_schema: serde_json::Value, // JSON Schema for validation
}

/// Template filter or function provided by a plugin
#[derive(Debug, Clone)]
pub struct TemplateExtension {
    pub name: String,
    pub extension_type: TemplateExtensionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateExtensionType {
    Filter,
    Function,
}

/// Plugin interface functions that WASM modules must export
///
/// All functions use JSON strings for data exchange to enable cross-language
/// compatibility and avoid direct type coupling.
pub mod plugin_exports {
    /// Returns a JSON array of task definitions
    ///
    /// Format: [{"name": "string", "type": "apply|facts|logs", "config_schema": {...}}]
    pub const GET_TASK_DEFINITIONS: &str = "get_task_definitions";

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
}

/// Host interface functions that plugins can import
///
/// These allow plugins to interact with the host environment in a controlled way.
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
