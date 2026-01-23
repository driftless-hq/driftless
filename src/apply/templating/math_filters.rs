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

    // Type conversion filters
    TemplateRegistry::register_filter(
        registry,
        "float",
        "Convert a value to a floating-point number",
        "Math/Logic Operations",
        vec![(
            "default".to_string(),
            "number: Default value if conversion fails (optional)".to_string(),
        )],
        Arc::new(|value, args| {
            let default_str = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "0.0".to_string());
            let default: f64 = default_str.parse().unwrap_or(0.0);

            if let Some(num) = value.as_i64() {
                JinjaValue::from(num as f64)
            } else {
                let s = value.to_string();
                if let Ok(num) = s.parse::<f64>() {
                    JinjaValue::from(num)
                } else {
                    JinjaValue::from(default)
                }
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "int",
        "Convert a value to an integer",
        "Math/Logic Operations",
        vec![
            (
                "default".to_string(),
                "integer: Default value if conversion fails (optional, default: 0)".to_string(),
            ),
            (
                "base".to_string(),
                "integer: Base for string conversion (optional, default: 10)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let default = args.first().and_then(|v| v.as_i64()).unwrap_or(0);
            let base = args.get(1).and_then(|v| v.as_i64()).unwrap_or(10) as u32;
            if let Some(num) = value.as_i64() {
                JinjaValue::from(num)
            } else {
                let s = value.to_string();
                if let Ok(num) = i64::from_str_radix(&s, base) {
                    JinjaValue::from(num)
                } else if let Ok(num) = s.parse::<f64>() {
                    JinjaValue::from(num as i64)
                } else {
                    JinjaValue::from(default)
                }
            }
        }),
    );

    // Mathematical functions
    TemplateRegistry::register_filter(
        registry,
        "log",
        "Return the logarithm of a number",
        "Math/Logic Operations",
        vec![(
            "base".to_string(),
            "number: The base of the logarithm (optional, default: e)".to_string(),
        )],
        Arc::new(|value, args| {
            let base_str = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| std::f64::consts::E.to_string());
            let base: f64 = base_str.parse().unwrap_or(std::f64::consts::E);

            let num_str = value.to_string();
            if let Ok(num) = num_str.parse::<f64>() {
                if num > 0.0 {
                    let result = if (base - std::f64::consts::E).abs() < f64::EPSILON {
                        num.ln()
                    } else if (base - 10.0).abs() < f64::EPSILON {
                        num.log10()
                    } else if (base - 2.0).abs() < f64::EPSILON {
                        num.log2()
                    } else {
                        num.log(base)
                    };
                    JinjaValue::from(result)
                } else {
                    JinjaValue::from(f64::NAN)
                }
            } else {
                JinjaValue::from(f64::NAN)
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "pow",
        "Return a number raised to a power",
        "Math/Logic Operations",
        vec![("exp".to_string(), "number: The exponent".to_string())],
        Arc::new(|value, args| {
            let exp_str = args
                .first()
                .map(|v| v.to_string())
                .unwrap_or_else(|| "1.0".to_string());
            let exp: f64 = exp_str.parse().unwrap_or(1.0);

            let base_str = value.to_string();
            if let Ok(base) = base_str.parse::<f64>() {
                let result = base.powf(exp);
                JinjaValue::from(result)
            } else {
                JinjaValue::from(f64::NAN)
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "sqrt",
        "Return the square root of a number",
        "Math/Logic Operations",
        vec![],
        Arc::new(|value, _args| {
            let s = value.to_string();
            if let Ok(num) = s.parse::<f64>() {
                if num >= 0.0 {
                    let result = num.sqrt();
                    JinjaValue::from(result)
                } else {
                    JinjaValue::from(f64::NAN)
                }
            } else {
                JinjaValue::from(f64::NAN)
            }
        }),
    );

    // Range generation filter
    TemplateRegistry::register_filter(
        registry,
        "range",
        "Generate a list of numbers in a range",
        "Math/Logic Operations",
        vec![
            (
                "start".to_string(),
                "integer: Start of the range (optional, default: 0)".to_string(),
            ),
            (
                "end".to_string(),
                "integer: End of the range (required if start is provided)".to_string(),
            ),
            (
                "step".to_string(),
                "integer: Step size (optional, default: 1)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            // Handle different calling conventions:
            // range(end) -> [0, 1, ..., end-1]
            // range(start, end) -> [start, start+1, ..., end-1]
            // range(start, end, step) -> [start, start+step, ..., <end]

            let (start, end, step) = if args.is_empty() {
                // range(end) where end is the value
                let end = value.as_i64().unwrap_or(0);
                (0, end, 1)
            } else if args.len() == 1 {
                // range(start, end) where start is value, end is args[0]
                let start = value.as_i64().unwrap_or(0);
                let end = args[0].as_i64().unwrap_or(0);
                (start, end, 1)
            } else {
                // range(start, end, step) where start is value, end is args[0], step is args[1]
                let start = value.as_i64().unwrap_or(0);
                let end = args[0].as_i64().unwrap_or(0);
                let step = args[1].as_i64().unwrap_or(1);
                (start, end, step)
            };

            let mut result = Vec::new();
            if step > 0 {
                let mut current = start;
                while current < end {
                    result.push(JinjaValue::from(current));
                    current += step;
                }
            } else if step < 0 {
                let mut current = start;
                while current > end {
                    result.push(JinjaValue::from(current));
                    current += step;
                }
            }
            JinjaValue::from(result)
        }),
    );
}
