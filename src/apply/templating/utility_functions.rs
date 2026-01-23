//! Utility functions for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use std::sync::Arc;

/// Register utility functions
pub fn register_utility_functions(
    registry: &mut std::collections::HashMap<
        String,
        crate::apply::templating::TemplateFunctionEntry,
    >,
) {
    TemplateRegistry::register_function(
        registry,
        "length",
        "Return the length of a string, array, or object",
        "Utility Functions",
        vec![(
            "value".to_string(),
            "any: The value to get the length of (string, array, or object)".to_string(),
        )],
        Arc::new(|args: &[JinjaValue]| {
            args.first()
                .map(|v| JinjaValue::from(v.len().unwrap_or(0) as i64))
                .unwrap_or(JinjaValue::from(0))
        }),
    );

    TemplateRegistry::register_function(
        registry,
        "lookup",
        "Look up values from various sources (env, file, etc.)",
        "Lookup Functions",
        vec![
            (
                "type".to_string(),
                "string: The lookup type (env, file, template, pipe)".to_string(),
            ),
            (
                "key".to_string(),
                "string: The key/path/command to look up".to_string(),
            ),
        ],
        Arc::new(|args| {
            if args.len() >= 2 {
                if let (Some(type_str), Some(key)) = (args[0].as_str(), args[1].as_str()) {
                    match type_str {
                        "env" => {
                            return JinjaValue::from(std::env::var(key).unwrap_or_default());
                        }
                        "file" => {
                            // Read content from file
                            match std::fs::read_to_string(key) {
                                Ok(content) => return JinjaValue::from(content),
                                Err(_) => return JinjaValue::from(""), // Return empty string on error
                            }
                        }
                        "template" => {
                            // For template lookups, we need access to the template engine
                            // This is more complex and would require passing the environment
                            // For now, just read the file as-is
                            match std::fs::read_to_string(key) {
                                Ok(content) => return JinjaValue::from(content),
                                Err(_) => return JinjaValue::from(""),
                            }
                        }
                        "pipe" => {
                            // Execute command and return output
                            match std::process::Command::new("sh").arg("-c").arg(key).output() {
                                Ok(output) => {
                                    if output.status.success() {
                                        let stdout = String::from_utf8_lossy(&output.stdout);
                                        return JinjaValue::from(stdout.trim());
                                    } else {
                                        return JinjaValue::from(""); // Return empty on command failure
                                    }
                                }
                                Err(_) => return JinjaValue::from(""),
                            }
                        }
                        _ => {} // Unknown type, fall through to "None"
                    }
                }
            }
            JinjaValue::from("None")
        }),
    );

    TemplateRegistry::register_function(
        registry,
        "hash",
        "Return the hash of a string using the specified algorithm",
        "Utility Functions",
        vec![
            (
                "value".to_string(),
                "string: The string to hash".to_string(),
            ),
            (
                "algorithm".to_string(),
                "string: The hash algorithm (md5, sha1, sha256, sha384, sha512)".to_string(),
            ),
        ],
        Arc::new(|args| {
            if args.len() >= 2 {
                if let (Some(value), Some(algorithm)) = (args[0].as_str(), args[1].as_str()) {
                    return match algorithm {
                        "md5" => {
                            let hash = md5::compute(value.as_bytes());
                            JinjaValue::from(format!("{:x}", hash))
                        }
                        "sha1" => {
                            use sha1::Digest;
                            let hash = sha1::Sha1::digest(value.as_bytes());
                            JinjaValue::from(format!("{:x}", hash))
                        }
                        "sha256" => {
                            use sha2::Digest;
                            let hash = sha2::Sha256::digest(value.as_bytes());
                            JinjaValue::from(format!("{:x}", hash))
                        }
                        "sha384" => {
                            use sha2::Digest;
                            let hash = sha2::Sha384::digest(value.as_bytes());
                            JinjaValue::from(format!("{:x}", hash))
                        }
                        "sha512" => {
                            use sha2::Digest;
                            let hash = sha2::Sha512::digest(value.as_bytes());
                            JinjaValue::from(format!("{:x}", hash))
                        }
                        _ => JinjaValue::from(false), // Invalid algorithm
                    };
                }
            }
            JinjaValue::from(false) // Invalid arguments
        }),
    );

    TemplateRegistry::register_function(
        registry,
        "uuid",
        "Generate a random UUID4",
        "Utility Functions",
        vec![],
        Arc::new(|_args| {
            let id = uuid::Uuid::new_v4();
            JinjaValue::from(id.to_string())
        }),
    );

    TemplateRegistry::register_function(
        registry,
        "timestamp",
        "Return the current timestamp",
        "Utility Functions",
        vec![(
            "format".to_string(),
            "string: Optional strftime format string (default: ISO 8601)".to_string(),
        )],
        Arc::new(|args| {
            let now = chrono::Utc::now();
            if let Some(format_str) = args.first().and_then(|v| v.as_str()) {
                // Try to format with the given string, fall back to ISO 8601 on error
                let formatted = now.format(format_str);
                // Use a safe conversion that won't panic
                match std::panic::catch_unwind(|| formatted.to_string()) {
                    Ok(result) => JinjaValue::from(result),
                    Err(_) => JinjaValue::from(now.to_rfc3339()), // Fallback on invalid format
                }
            } else {
                // Default ISO 8601 format
                JinjaValue::from(now.to_rfc3339())
            }
        }),
    );
}
