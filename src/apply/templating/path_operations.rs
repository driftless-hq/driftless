//! Path and filesystem operation filters and functions for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use std::{path::Path, sync::Arc};

/// Register path-related filters
pub fn register_path_filters(
    registry: &mut std::collections::HashMap<String, crate::apply::templating::TemplateFilterEntry>,
) {
    TemplateRegistry::register_filter(
        registry,
        "basename",
        "Return the basename of a path",
        "Path Operations",
        vec![],
        Arc::new(|value, _args| {
            let path_str = value.as_str().unwrap_or("");
            if path_str.ends_with('/') && path_str != "/" {
                // For paths ending with / (except root), basename is empty
                JinjaValue::from(String::new())
            } else {
                JinjaValue::from(
                    Path::new(path_str)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string(),
                )
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "dirname",
        "Return the directory name of a path",
        "Path Operations",
        vec![],
        Arc::new(|value, _args| {
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
        }),
    );
}

/// Register path-related functions
pub fn register_path_functions(
    registry: &mut std::collections::HashMap<
        String,
        crate::apply::templating::TemplateFunctionEntry,
    >,
) {
    TemplateRegistry::register_function(
        registry,
        "basename",
        "Return the basename of a path",
        "Path Operations",
        vec![(
            "path".to_string(),
            "string: The path to extract the basename from".to_string(),
        )],
        Arc::new(|args| {
            args.first()
                .and_then(|v| v.as_str())
                .map(|path_str| {
                    if path_str.ends_with('/') && path_str != "/" {
                        // For paths ending with / (except root), basename is empty
                        JinjaValue::from(String::new())
                    } else {
                        JinjaValue::from(
                            Path::new(path_str)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string(),
                        )
                    }
                })
                .unwrap_or(JinjaValue::from(String::new()))
        }),
    );

    TemplateRegistry::register_function(
        registry,
        "dirname",
        "Return the directory name of a path",
        "Path Operations",
        vec![(
            "path".to_string(),
            "string: The path to extract the directory name from".to_string(),
        )],
        Arc::new(|args| {
            args.first()
                .and_then(|v| v.as_str())
                .map(|path_str| {
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
                })
                .unwrap_or(JinjaValue::from(String::new()))
        }),
    );
}
