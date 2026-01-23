//! Encoding and decoding filters for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use base64::Engine;
use minijinja::Value as JinjaValue;
use std::sync::Arc;

/// Register encoding and decoding filters
pub fn register_encoding_filters(
    registry: &mut std::collections::HashMap<String, crate::apply::templating::TemplateFilterEntry>,
) {
    // b64encode filter
    TemplateRegistry::register_filter(
        registry,
        "b64encode",
        "Encode a string using base64 encoding.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                let encoded = base64::engine::general_purpose::STANDARD.encode(s);
                JinjaValue::from(encoded)
            } else {
                // For non-string values, convert to string first then encode
                let s = value.to_string();
                let encoded = base64::engine::general_purpose::STANDARD.encode(s);
                JinjaValue::from(encoded)
            }
        }),
    );

    // b64decode filter
    TemplateRegistry::register_filter(
        registry,
        "b64decode",
        "Decode a base64 encoded string.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                match base64::engine::general_purpose::STANDARD.decode(s) {
                    Ok(bytes) => match String::from_utf8(bytes) {
                        Ok(decoded) => JinjaValue::from(decoded),
                        Err(_) => JinjaValue::from(false), // Return false on invalid UTF-8
                    },
                    Err(_) => JinjaValue::from(false), // Return false on invalid base64
                }
            } else {
                JinjaValue::from(false) // Return false for non-string inputs
            }
        }),
    );

    // to_json filter
    TemplateRegistry::register_filter(
        registry,
        "to_json",
        "Serialize a value to JSON string.",
        "Encoding/Decoding",
        vec![(
            "indent".to_string(),
            "Number of spaces for indentation (optional)".to_string(),
        )],
        Arc::new(|value, args| {
            let indent = if !args.is_empty() {
                args[0].as_i64().unwrap_or(0) as usize
            } else {
                0
            };

            match serde_json::to_value(&value) {
                Ok(json_value) => {
                    if indent > 0 {
                        match serde_json::to_string_pretty(&json_value) {
                            Ok(s) => JinjaValue::from(s),
                            Err(_) => JinjaValue::from(false),
                        }
                    } else {
                        match serde_json::to_string(&json_value) {
                            Ok(s) => JinjaValue::from(s),
                            Err(_) => JinjaValue::from(false),
                        }
                    }
                }
                Err(_) => JinjaValue::from(false),
            }
        }),
    );

    // from_json filter
    TemplateRegistry::register_filter(
        registry,
        "from_json",
        "Parse a JSON string into a value.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                match serde_json::from_str::<serde_json::Value>(s) {
                    Ok(json_value) => JinjaValue::from_serialize(&json_value),
                    Err(_) => JinjaValue::from(false),
                }
            } else {
                JinjaValue::from(false)
            }
        }),
    );

    // to_yaml filter
    TemplateRegistry::register_filter(
        registry,
        "to_yaml",
        "Serialize a value to YAML string.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| match serde_yaml::to_string(&value) {
            Ok(yaml_str) => JinjaValue::from(yaml_str),
            Err(_) => JinjaValue::from(false),
        }),
    );

    // from_yaml filter
    TemplateRegistry::register_filter(
        registry,
        "from_yaml",
        "Parse a YAML string into a value.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                match serde_yaml::from_str::<serde_yaml::Value>(s) {
                    Ok(yaml_value) => JinjaValue::from_serialize(&yaml_value),
                    Err(_) => JinjaValue::from(false),
                }
            } else {
                JinjaValue::from(false)
            }
        }),
    );
}
