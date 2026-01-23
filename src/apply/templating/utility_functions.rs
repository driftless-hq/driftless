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
        vec!["value: any - The value to get the length of (string, array, or object)".to_string()],
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
            "type: string - The lookup type (currently only 'env' is supported)".to_string(),
            "key: string - The key to look up".to_string(),
        ],
        Arc::new(|args| {
            if args.len() >= 2 {
                if let (Some(type_str), Some(key)) = (args[0].as_str(), args[1].as_str()) {
                    if type_str == "env" {
                        return JinjaValue::from(std::env::var(key).unwrap_or_default());
                    }
                }
            }
            JinjaValue::from("None")
        }),
    );
}
