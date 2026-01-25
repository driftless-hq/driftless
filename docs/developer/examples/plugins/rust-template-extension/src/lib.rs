//! Example Template Extension Plugin for Driftless
//!
//! This plugin demonstrates how to create custom Jinja2 filters and functions
//! for use in Driftless templates. It provides filters for text transformation
//! and utility functions.

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
pub fn get_template_extensions() -> String {
    let extensions = vec![
        serde_json::json!({
            "name": "base64_encode",
            "type": "filter",
            "config_schema": {
                "type": "object",
                "properties": {}
            },
            "description": "Base64 encode a string",
            "category": "encoding",
            "arguments": [
                ["input", "String to encode"]
            ]
        }),
        serde_json::json!({
            "name": "base64_decode",
            "type": "filter",
            "config_schema": {
                "type": "object",
                "properties": {}
            },
            "description": "Base64 decode a string",
            "category": "encoding",
            "arguments": [
                ["input", "String to decode"]
            ]
        }),
        serde_json::json!({
            "name": "slugify",
            "type": "filter",
            "config_schema": {
                "type": "object",
                "properties": {}
            },
            "description": "Convert string to URL-friendly slug",
            "category": "text",
            "arguments": [
                ["input", "String to slugify"]
            ]
        }),
        serde_json::json!({
            "name": "random_string",
            "type": "function",
            "config_schema": {
                "type": "object",
                "properties": {
                    "length": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 16
                    }
                }
            },
            "description": "Generate a random string",
            "category": "utility",
            "arguments": [
                ["length", "Length of the string (optional, default 16)"]
            ]
        }),
    ];

    serde_json::to_string(&extensions).unwrap()
}

#[wasm_bindgen]
pub fn get_plugin_metadata() -> String {
    serde_json::json!({
        "version": "1.0.0",
        "description": "Example template extension plugin demonstrating custom filters"
    }).to_string()
}

#[wasm_bindgen]
pub fn execute_template_filter(
    name: &str,
    _config_json: &str,
    value_json: &str,
    args_json: &str,
) -> String {
    match name {
        "base64_encode" => execute_base64_encode_filter(value_json),
        "base64_decode" => execute_base64_decode_filter(value_json),
        "slugify" => execute_slugify_filter(value_json),
        _ => serde_json::json!({
            "error": format!("Unknown filter: {}", name)
        })
        .to_string(),
    }
}

#[wasm_bindgen]
pub fn execute_template_function(name: &str, config_json: &str, args_json: &str) -> String {
    match name {
        "random_string" => execute_random_string_function(config_json, args_json),
        _ => serde_json::json!({
            "error": format!("Unknown function: {}", name)
        })
        .to_string(),
    }
}

fn execute_base64_encode_filter(value_json: &str) -> String {
    match serde_json::from_str::<Value>(value_json) {
        Ok(value) => {
            if let Some(s) = value.as_str() {
                let encoded = base64::encode(s);
                serde_json::to_string(&encoded).unwrap()
            } else {
                serde_json::json!({"error": "Filter input must be a string"}).to_string()
            }
        }
        Err(e) => serde_json::json!({"error": format!("Invalid input: {}", e)}).to_string(),
    }
}

fn execute_base64_decode_filter(value_json: &str) -> String {
    match serde_json::from_str::<Value>(value_json) {
        Ok(value) => {
            if let Some(s) = value.as_str() {
                match base64::decode(s) {
                    Ok(bytes) => match String::from_utf8(bytes) {
                        Ok(decoded) => serde_json::to_string(&decoded).unwrap(),
                        Err(e) => serde_json::json!({"error": format!("Invalid UTF-8: {}", e)})
                            .to_string(),
                    },
                    Err(e) => {
                        serde_json::json!({"error": format!("Invalid base64: {}", e)}).to_string()
                    }
                }
            } else {
                serde_json::json!({"error": "Filter input must be a string"}).to_string()
            }
        }
        Err(e) => serde_json::json!({"error": format!("Invalid input: {}", e)}).to_string(),
    }
}

fn execute_slugify_filter(value_json: &str) -> String {
    match serde_json::from_str::<Value>(value_json) {
        Ok(value) => {
            if let Some(s) = value.as_str() {
                let slug = s
                    .to_lowercase()
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == ' ' || c == '-' {
                            c
                        } else {
                            '-'
                        }
                    })
                    .collect::<String>()
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join("-")
                    .trim_matches('-')
                    .to_string();

                serde_json::to_string(&slug).unwrap()
            } else {
                serde_json::json!({"error": "Filter input must be a string"}).to_string()
            }
        }
        Err(e) => serde_json::json!({"error": format!("Invalid input: {}", e)}).to_string(),
    }
}

fn execute_random_string_function(config_json: &str, args_json: &str) -> String {
    let length = if let Ok(args) = serde_json::from_str::<Vec<Value>>(args_json) {
        if let Some(len_val) = args.get(0) {
            len_val.as_u64().unwrap_or(16) as usize
        } else {
            16
        }
    } else {
        16
    };

    // Generate a simple random string (in real implementation, use proper RNG)
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut result = String::with_capacity(length);

    for _ in 0..length {
        let idx = (js_sys::Math::random() * charset.len() as f64) as usize;
        result.push(charset.chars().nth(idx).unwrap_or('a'));
    }

    serde_json::to_string(&result).unwrap()
}

// Other required plugin functions (return empty arrays for this example)
#[wasm_bindgen]
pub fn get_task_definitions() -> String {
    "[]".to_string()
}

#[wasm_bindgen]
pub fn get_facts_collectors() -> String {
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
    fn test_get_template_extensions() {
        let extensions: Vec<Value> = serde_json::from_str(&get_template_extensions()).unwrap();
        assert_eq!(extensions.len(), 4);

        let names: Vec<&str> = extensions
            .iter()
            .map(|e| e["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"base64_encode"));
        assert!(names.contains(&"base64_decode"));
        assert!(names.contains(&"slugify"));
        assert!(names.contains(&"random_string"));
    }

    #[test]
    fn test_base64_encode_filter() {
        let input = serde_json::to_string("Hello, World!").unwrap();
        let result: Value = serde_json::from_str(&execute_template_filter(
            "base64_encode",
            "{}",
            &input,
            "[]",
        ))
        .unwrap();
        assert_eq!(result.as_str().unwrap(), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn test_slugify_filter() {
        let input = serde_json::to_string("Hello, World! This is a Test.").unwrap();
        let result: Value =
            serde_json::from_str(&execute_template_filter("slugify", "{}", &input, "[]")).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello-world-this-is-a-test");
    }

    #[test]
    fn test_random_string_function() {
        let result: Value =
            serde_json::from_str(&execute_template_function("random_string", "{}", "[8]")).unwrap();
        let s = result.as_str().unwrap();
        assert_eq!(s.len(), 8);
        assert!(s.chars().all(|c| c.is_alphanumeric()));
    }
}
