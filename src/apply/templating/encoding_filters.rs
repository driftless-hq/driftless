//! Encoding and decoding filters for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use base64::Engine;
use minijinja::Value as JinjaValue;
use regex::Regex;
use std::sync::Arc;
use urlencoding;

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

    // mandatory filter
    TemplateRegistry::register_filter(
        registry,
        "mandatory",
        "Fail if the value is undefined, None, or empty. Otherwise return the value.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if value.is_undefined() || value.is_none() {
                // Fail template rendering for undefined/none values
                minijinja::Value::UNDEFINED
            } else if let Some(s) = value.as_str() {
                if s.is_empty() {
                    minijinja::Value::UNDEFINED
                } else {
                    value
                }
            } else if let Ok(seq) = value.try_iter() {
                if seq.count() == 0 {
                    minijinja::Value::UNDEFINED
                } else {
                    value
                }
            } else {
                value
            }
        }),
    );

    // regex_escape filter
    TemplateRegistry::register_filter(
        registry,
        "regex_escape",
        "Escape special regex characters in a string.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                JinjaValue::from(regex::escape(s))
            } else {
                value
            }
        }),
    );

    // regex_findall filter
    TemplateRegistry::register_filter(
        registry,
        "regex_findall",
        "Find all matches of a regex pattern in a string.",
        "Encoding/Decoding",
        vec![(
            "pattern".to_string(),
            "string: The regex pattern to search for".to_string(),
        )],
        Arc::new(|value, args| {
            let pattern = args.first().and_then(|v| v.as_str()).unwrap_or("");
            if let Some(text) = value.as_str() {
                match Regex::new(pattern) {
                    Ok(re) => {
                        let matches: Vec<JinjaValue> = re
                            .find_iter(text)
                            .map(|m| JinjaValue::from(m.as_str()))
                            .collect();
                        JinjaValue::from(matches)
                    }
                    Err(_) => JinjaValue::from(Vec::<JinjaValue>::new()),
                }
            } else {
                JinjaValue::from(Vec::<JinjaValue>::new())
            }
        }),
    );

    // regex_replace filter
    TemplateRegistry::register_filter(
        registry,
        "regex_replace",
        "Replace matches of a regex pattern in a string.",
        "Encoding/Decoding",
        vec![
            (
                "pattern".to_string(),
                "string: The regex pattern to search for".to_string(),
            ),
            (
                "replacement".to_string(),
                "string: The replacement string".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let pattern = args.first().and_then(|v| v.as_str()).unwrap_or("");
            let replacement = args.get(1).and_then(|v| v.as_str()).unwrap_or("");
            if let Some(text) = value.as_str() {
                match Regex::new(pattern) {
                    Ok(re) => {
                        let result = re.replace_all(text, replacement);
                        JinjaValue::from(result.to_string())
                    }
                    Err(_) => JinjaValue::from(text.to_string()),
                }
            } else {
                value
            }
        }),
    );

    // regex_search filter
    TemplateRegistry::register_filter(
        registry,
        "regex_search",
        "Search for a regex pattern in a string and return the first match.",
        "Encoding/Decoding",
        vec![(
            "pattern".to_string(),
            "string: The regex pattern to search for".to_string(),
        )],
        Arc::new(|value, args| {
            let pattern = args.first().and_then(|v| v.as_str()).unwrap_or("");
            if let Some(text) = value.as_str() {
                match Regex::new(pattern) {
                    Ok(re) => {
                        if let Some(mat) = re.find(text) {
                            JinjaValue::from(mat.as_str())
                        } else {
                            JinjaValue::from(false)
                        }
                    }
                    Err(_) => JinjaValue::from(false),
                }
            } else {
                JinjaValue::from(false)
            }
        }),
    );

    // to_nice_json filter
    TemplateRegistry::register_filter(
        registry,
        "to_nice_json",
        "Convert a value to a nicely formatted JSON string.",
        "Encoding/Decoding",
        vec![(
            "indent".to_string(),
            "integer: Number of spaces for indentation (optional, default: 2)".to_string(),
        )],
        Arc::new(|value, args| {
            let indent = args.first().and_then(|v| v.as_i64()).unwrap_or(2) as usize;
            match serde_json::to_string_pretty(&value) {
                Ok(json) => {
                    // Apply custom indentation if different from default
                    if indent != 2 {
                        let lines: Vec<&str> = json.lines().collect();
                        let indented: Vec<String> = lines
                            .iter()
                            .enumerate()
                            .map(|(i, line)| {
                                if i == 0 {
                                    line.to_string()
                                } else {
                                    format!("{}{}", " ".repeat(indent), line.trim_start())
                                }
                            })
                            .collect();
                        JinjaValue::from(indented.join("\n"))
                    } else {
                        JinjaValue::from(json)
                    }
                }
                Err(_) => JinjaValue::from("{}"),
            }
        }),
    );

    // to_nice_yaml filter
    TemplateRegistry::register_filter(
        registry,
        "to_nice_yaml",
        "Convert a value to a nicely formatted YAML string.",
        "Encoding/Decoding",
        vec![(
            "indent".to_string(),
            "integer: Number of spaces for indentation (optional, default: 2)".to_string(),
        )],
        Arc::new(|value, args| {
            let _indent = args.first().and_then(|v| v.as_i64()).unwrap_or(2);
            match serde_yaml::to_string(&value) {
                Ok(yaml) => JinjaValue::from(yaml),
                Err(_) => JinjaValue::from(""),
            }
        }),
    );

    // urlencode filter
    TemplateRegistry::register_filter(
        registry,
        "urlencode",
        "URL encode a string.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                JinjaValue::from(urlencoding::encode(s))
            } else {
                let s = value.to_string();
                JinjaValue::from(urlencoding::encode(&s))
            }
        }),
    );

    // urldecode filter
    TemplateRegistry::register_filter(
        registry,
        "urldecode",
        "URL decode a string.",
        "Encoding/Decoding",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                match urlencoding::decode(s) {
                    Ok(decoded) => JinjaValue::from(decoded.to_string()),
                    Err(_) => JinjaValue::from(s.to_string()),
                }
            } else {
                value
            }
        }),
    );
}
