//! Example Custom Task Plugin for Driftless
//!
//! This plugin demonstrates how to create custom tasks that can be executed
//! by the Driftless agent. It provides a simple "echo" task that logs messages
//! and a "file_touch" task that creates empty files.

use serde_json::Value;
use wasm_bindgen::prelude::*;

// Export required plugin interface functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Helper macro for logging from WASM
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn get_task_definitions() -> String {
    let definitions = vec![
        serde_json::json!({
            "name": "echo",
            "type": "apply",
            "config_schema": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to log"
                    },
                    "level": {
                        "type": "string",
                        "enum": ["info", "warn", "error"],
                        "default": "info",
                        "description": "Log level"
                    }
                },
                "required": ["message"]
            }
        }),
        serde_json::json!({
            "name": "file_touch",
            "type": "apply",
            "config_schema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to create"
                    },
                    "mode": {
                        "type": "string",
                        "pattern": "^[0-7]{3,4}$",
                        "default": "644",
                        "description": "File permissions (octal)"
                    }
                },
                "required": ["path"]
            }
        }),
    ];

    serde_json::to_string(&definitions).unwrap()
}

#[wasm_bindgen]
pub fn execute_task(name: &str, config_json: &str) -> String {
    match name {
        "echo" => execute_echo_task(config_json),
        "file_touch" => execute_file_touch_task(config_json),
        _ => serde_json::json!({
            "status": "error",
            "message": format!("Unknown task: {}", name)
        })
        .to_string(),
    }
}

fn execute_echo_task(config_json: &str) -> String {
    match serde_json::from_str::<Value>(config_json) {
        Ok(config) => {
            let message = config["message"].as_str().unwrap_or("No message");
            let level = config["level"].as_str().unwrap_or("info");

            match level {
                "info" => console_log!("ECHO: {}", message),
                "warn" => console_log!("ECHO WARNING: {}", message),
                "error" => console_log!("ECHO ERROR: {}", message),
                _ => console_log!("ECHO [{}]: {}", level, message),
            }

            serde_json::json!({
                "status": "success",
                "message": format!("Logged message: {}", message),
                "level": level
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "status": "error",
            "message": format!("Invalid configuration: {}", e)
        })
        .to_string(),
    }
}

fn execute_file_touch_task(config_json: &str) -> String {
    match serde_json::from_str::<Value>(config_json) {
        Ok(config) => {
            let path = config["path"].as_str().unwrap_or("");
            let mode = config["mode"].as_str().unwrap_or("644");

            if path.is_empty() {
                return serde_json::json!({
                    "status": "error",
                    "message": "Path cannot be empty"
                })
                .to_string();
            }

            // Note: In a real plugin, we would use host functions to create files
            // For this example, we just log the intention
            console_log!("Would create file: {} with mode: {}", path, mode);

            serde_json::json!({
                "status": "success",
                "message": format!("File '{}' would be created with mode {}", path, mode),
                "path": path,
                "mode": mode
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "status": "error",
            "message": format!("Invalid configuration: {}", e)
        })
        .to_string(),
    }
}

// Other required plugin functions (return empty arrays for this example)
#[wasm_bindgen]
pub fn get_facts_collectors() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn get_template_extensions() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn get_log_sources() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn get_log_parsers() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn get_log_filters() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn get_log_outputs() -> String {
    "[]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_task_definitions() {
        let definitions: Vec<Value> = serde_json::from_str(&get_task_definitions()).unwrap();
        assert_eq!(definitions.len(), 2);
        assert_eq!(definitions[0]["name"], "echo");
        assert_eq!(definitions[1]["name"], "file_touch");
    }

    #[test]
    fn test_execute_echo_task() {
        let config = r#"{"message": "Hello, World!", "level": "info"}"#;
        let result: Value = serde_json::from_str(&execute_task("echo", config)).unwrap();
        assert_eq!(result["status"], "success");
        assert!(result["message"]
            .as_str()
            .unwrap()
            .contains("Hello, World!"));
    }

    #[test]
    fn test_execute_unknown_task() {
        let result: Value = serde_json::from_str(&execute_task("unknown", "{}")).unwrap();
        assert_eq!(result["status"], "error");
        assert!(result["message"].as_str().unwrap().contains("Unknown task"));
    }

    #[test]
    fn test_execute_file_touch_task() {
        let config = r#"{"path": "/tmp/test.txt", "mode": "755"}"#;
        let result: Value = serde_json::from_str(&execute_task("file_touch", config)).unwrap();
        assert_eq!(result["status"], "success");
        assert!(result["message"].as_str().unwrap().contains("test.txt"));
    }
}
