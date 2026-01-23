//! Generator functions for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use rand::Rng;
use std::sync::Arc;

/// Register generator functions
pub fn register_generator_functions(
    registry: &mut std::collections::HashMap<
        String,
        crate::apply::templating::TemplateFunctionEntry,
    >,
) {
    TemplateRegistry::register_function(
        registry,
        "range",
        "Generate a sequence of numbers.",
        "Generator Functions",
        vec![
            (
                "end_or_start".to_string(),
                "int: The end value (exclusive) for single arg, or start value for multiple args"
                    .to_string(),
            ),
            (
                "end".to_string(),
                "int: The end value (exclusive)".to_string(),
            ),
            (
                "step".to_string(),
                "int: The step value (optional, defaults to 1)".to_string(),
            ),
        ],
        Arc::new(|args| {
            if args.is_empty() {
                return JinjaValue::from(Vec::<JinjaValue>::new());
            }

            let (start, end, step) = match args.len() {
                1 => {
                    // range(end) - from 0 to end-1
                    let end = match args[0].as_i64() {
                        Some(v) => v,
                        None => return JinjaValue::from(Vec::<JinjaValue>::new()),
                    };
                    (0, end, 1)
                }
                2 => {
                    // range(start, end) - from start to end-1
                    let start = match args[0].as_i64() {
                        Some(v) => v,
                        None => return JinjaValue::from(Vec::<JinjaValue>::new()),
                    };
                    let end = match args[1].as_i64() {
                        Some(v) => v,
                        None => return JinjaValue::from(Vec::<JinjaValue>::new()),
                    };
                    (start, end, 1)
                }
                _ => {
                    // range(start, end, step) - from start to end-1 with step
                    let start = match args[0].as_i64() {
                        Some(v) => v,
                        None => return JinjaValue::from(Vec::<JinjaValue>::new()),
                    };
                    let end = match args[1].as_i64() {
                        Some(v) => v,
                        None => return JinjaValue::from(Vec::<JinjaValue>::new()),
                    };
                    let step = args.get(2).and_then(|v| v.as_i64()).unwrap_or(1);
                    (start, end, step)
                }
            };

            if step == 0 {
                return JinjaValue::from(Vec::<JinjaValue>::new());
            }

            let mut result = Vec::new();
            if step > 0 {
                let mut current = start;
                while current < end {
                    result.push(JinjaValue::from(current));
                    current += step;
                }
            } else {
                let mut current = start;
                while current > end {
                    result.push(JinjaValue::from(current));
                    current += step;
                }
            }

            JinjaValue::from(result)
        }),
    );

    TemplateRegistry::register_function(
        registry,
        "random",
        "Generate random numbers.",
        "Generator Functions",
        vec![
            (
                "max".to_string(),
                "int: The maximum value (exclusive) or minimum value if second arg provided"
                    .to_string(),
            ),
            (
                "max".to_string(),
                "int: The maximum value (exclusive)".to_string(),
            ),
        ],
        Arc::new(|args| {
            let mut rng = rand::rng();

            match args.len() {
                0 => {
                    // random() - return float between 0.0 and 1.0
                    let random_float: f64 = rng.random();
                    JinjaValue::from(random_float)
                }
                1 => {
                    // random(max) - return int from 0 to max-1
                    if let Some(max) = args[0].as_i64() {
                        if max <= 0 {
                            return JinjaValue::from(0);
                        }
                        let random_int = rng.random_range(0..max);
                        JinjaValue::from(random_int)
                    } else {
                        JinjaValue::from(0)
                    }
                }
                2.. => {
                    // random(min, max) - return int from min to max-1
                    if let (Some(min), Some(max)) = (args[0].as_i64(), args[1].as_i64()) {
                        if min >= max {
                            return JinjaValue::from(min);
                        }
                        let random_int = rng.random_range(min..max);
                        JinjaValue::from(random_int)
                    } else {
                        JinjaValue::from(0)
                    }
                }
            }
        }),
    );
}
