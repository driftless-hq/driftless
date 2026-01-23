//! Math and logic filters for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use std::sync::Arc;

/// Register math and logic filters
pub fn register_math_filters(
    registry: &mut std::collections::HashMap<String, crate::apply::templating::TemplateFilterEntry>,
) {
    // Math/Logic Operations
    TemplateRegistry::register_filter(
        registry,
        "abs",
        "Return the absolute value of a number",
        "Math/Logic Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Some(num) = value.as_i64() {
                JinjaValue::from(num.abs())
            } else {
                // Try to parse as float from string representation
                let s = value.to_string();
                if let Ok(num) = s.parse::<f64>() {
                    JinjaValue::from(num.abs())
                } else {
                    JinjaValue::from(0)
                }
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "round",
        "Round a number to a given precision (default 0 decimal places)",
        "Math/Logic Operations",
        vec![(
            "precision".to_string(),
            "integer: The number of decimal places to round to (optional, default: 0)".to_string(),
        )],
        Arc::new(|value, args| {
            let precision = args.first().and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            if let Some(num) = value.as_i64() {
                JinjaValue::from(num)
            } else {
                // Try to parse as float from string representation
                let s = value.to_string();
                if let Ok(num) = s.parse::<f64>() {
                    let multiplier = 10f64.powi(precision);
                    let rounded = (num * multiplier).round() / multiplier;
                    if rounded.fract() == 0.0 {
                        JinjaValue::from(rounded as i64)
                    } else {
                        JinjaValue::from(rounded)
                    }
                } else {
                    JinjaValue::from(0)
                }
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "random",
        "Return a random number, optionally within a specified range",
        "Math/Logic Operations",
        vec![
            (
                "start".to_string(),
                "integer: The starting value of the range (optional)".to_string(),
            ),
            (
                "end".to_string(),
                "integer: The ending value of the range (optional)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            use rand::Rng;
            let mut rng = rand::rng();

            if let (Some(start), Some(end)) = (
                args.first().and_then(|v| v.as_i64()),
                args.get(1).and_then(|v| v.as_i64()),
            ) {
                JinjaValue::from(rng.random_range(start..=end))
            } else if let Some(end) = args.first().and_then(|v| v.as_i64()) {
                // One argument: use value as start, argument as end
                if let Some(start) = value.as_i64() {
                    JinjaValue::from(rng.random_range(start..=end))
                } else {
                    JinjaValue::from(rng.random_range(0..=end))
                }
            } else if let Some(end) = value.as_i64() {
                // No args: use value as end
                JinjaValue::from(rng.random_range(0..=end))
            } else if let Some(seq) = value.as_str() {
                // Random character from string
                if seq.is_empty() {
                    JinjaValue::from("")
                } else {
                    let chars: Vec<char> = seq.chars().collect();
                    let idx = rng.random_range(0..chars.len());
                    JinjaValue::from(chars[idx].to_string())
                }
            } else if let Ok(iter) = value.try_iter() {
                let items: Vec<JinjaValue> = iter.collect();
                if items.is_empty() {
                    // For empty sequences, return default random
                    JinjaValue::from(rng.random_range(0..=100))
                } else {
                    let idx = rng.random_range(0..items.len());
                    items[idx].clone()
                }
            } else {
                // Default random number 0-100
                JinjaValue::from(rng.random_range(0..=100))
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "bool",
        "Convert value to boolean",
        "Math/Logic Operations",
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.is_true())),
    );

    TemplateRegistry::register_filter(
        registry,
        "ternary",
        "Return one of two values based on condition (true_val if condition is true, false_val if false)",
        "Math/Logic Operations",
        vec![
            ("true_val".to_string(), "any: The value to return if the condition is true".to_string()),
            ("false_val".to_string(), "any: The value to return if the condition is false".to_string()),
        ],
        Arc::new(|value, args| {
            if value.is_true() {
                args.first().cloned().unwrap_or(JinjaValue::from(true))
            } else {
                args.get(1).cloned().unwrap_or(JinjaValue::from(false))
            }
        }),
    );
}
