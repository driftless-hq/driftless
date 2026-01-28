//! Additional path and filesystem operation filters for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use std::{path::Path, sync::Arc};

/// Register additional path-related filters
pub fn register_path_filters(
    registry: &mut std::collections::HashMap<String, crate::apply::templating::TemplateFilterEntry>,
) {
    // expanduser filter
    TemplateRegistry::register_filter(
        registry,
        "expanduser",
        "Expand a path containing a tilde (~) to the user's home directory.",
        "Path Operations",
        vec![],
        Arc::new(|value, _args| {
            if value.is_undefined() || value.is_none() {
                JinjaValue::from(minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "expanduser filter received undefined or none value",
                ))
            } else if let Some(path_str) = value.as_str() {
                if path_str.starts_with('~') {
                    if path_str == "~" || path_str.starts_with("~/") {
                        // Expand ~ or ~/ to home directory
                        if let Some(home_dir) = dirs::home_dir() {
                            let rest = if path_str == "~" {
                                ""
                            } else {
                                &path_str[2..] // Remove "~/"
                            };
                            if rest.is_empty() {
                                // For just "~", return the home directory as-is (no trailing slash)
                                if let Some(home_str) = home_dir.to_str() {
                                    return JinjaValue::from(home_str);
                                }
                            } else {
                                // For "~/path", join and return
                                let expanded = home_dir.join(rest);
                                if let Some(expanded_str) = expanded.to_str() {
                                    return JinjaValue::from(expanded_str);
                                }
                            }
                        }
                        // If home directory can't be determined, return original
                        JinjaValue::from(path_str)
                    } else {
                        // Handle ~user syntax - for now, just return as-is since we don't have user lookup
                        // In a full implementation, this would look up the user's home directory
                        JinjaValue::from(path_str)
                    }
                } else {
                    // No tilde, return as-is
                    JinjaValue::from(path_str)
                }
            } else {
                // Non-string input, return as-is
                value
            }
        }),
    );

    // realpath filter
    TemplateRegistry::register_filter(
        registry,
        "realpath",
        "Return the canonical absolute path, resolving symlinks and relative components.",
        "Path Operations",
        vec![],
        Arc::new(|value, _args| {
            if value.is_undefined() || value.is_none() {
                JinjaValue::from(minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "realpath filter received undefined or none value",
                ))
            } else if let Some(path_str) = value.as_str() {
                match std::fs::canonicalize(Path::new(path_str)) {
                    Ok(canonical_path) => {
                        if let Some(canonical_str) = canonical_path.to_str() {
                            JinjaValue::from(canonical_str)
                        } else {
                            // Path contains invalid UTF-8, return original
                            JinjaValue::from(path_str)
                        }
                    }
                    Err(_) => {
                        // Path doesn't exist or can't be canonicalized, return original
                        JinjaValue::from(path_str)
                    }
                }
            } else {
                // Non-string input, return as-is
                value
            }
        }),
    );
}
