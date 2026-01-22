//! Custom Jinja2 functions for templating

use minijinja::{Environment, Value as JinjaValue};
use std::path::Path;

/// Add all custom functions to the minijinja environment
pub fn add_functions(env: &mut Environment) {
    env.add_function("length", |value: JinjaValue| {
        JinjaValue::from(value.len().unwrap_or(0) as i64)
    });

    env.add_function("basename", |value: JinjaValue| {
        JinjaValue::from(
            Path::new(value.as_str().unwrap_or(""))
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
        )
    });

    env.add_function("dirname", |value: JinjaValue| {
        let path_str = value.as_str().unwrap_or("");
        if path_str.is_empty() {
            return JinjaValue::from(String::new());
        }
        // For paths ending with /, dirname is the path without the trailing /
        if path_str.ends_with('/') {
            return JinjaValue::from(path_str.trim_end_matches('/').to_string());
        }
        // Otherwise, use Path::parent()
        JinjaValue::from(
            Path::new(path_str)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string(),
        )
    });

    env.add_function(
        "lookup",
        |type_str: String, key: Option<String>| -> JinjaValue {
            if type_str == "env" {
                if let Some(key) = key {
                    JinjaValue::from(std::env::var(key).unwrap_or_default())
                } else {
                    JinjaValue::from(String::new())
                }
            } else {
                JinjaValue::from(String::new())
            }
        },
    );

    // Add new functions here (e.g., range, random, etc.)
}
