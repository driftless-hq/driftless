//! String manipulation filters for Jinja2 templating

use crate::apply::templating::TemplateRegistry;
use minijinja::Value as JinjaValue;
use std::sync::Arc;

/// Register string manipulation filters
pub fn register_string_filters(
    registry: &mut std::collections::HashMap<String, crate::apply::templating::TemplateFilterEntry>,
) {
    // String case filters
    TemplateRegistry::register_filter(
        registry,
        "upper",
        "Convert a string to uppercase",
        "String Operations",
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").to_uppercase())),
    );

    TemplateRegistry::register_filter(
        registry,
        "lower",
        "Convert a string to lowercase",
        "String Operations",
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").to_lowercase())),
    );

    TemplateRegistry::register_filter(
        registry,
        "capitalize",
        "Capitalize the first character of a string",
        "String Operations",
        Arc::new(|value, _args| {
            let s = value.as_str().unwrap_or("");
            if s.is_empty() {
                JinjaValue::from(String::new())
            } else {
                let mut chars = s.chars();
                let first = chars.next().unwrap().to_uppercase().collect::<String>();
                let rest = chars.as_str().to_lowercase();
                JinjaValue::from(format!("{}{}", first, rest))
            }
        }),
    );

    // String length filter
    TemplateRegistry::register_filter(
        registry,
        "length",
        "Return the length of a string, list, or object",
        "String/List Operations",
        Arc::new(|value, _args| JinjaValue::from(value.len().unwrap_or(0) as i64)),
    );

    // Complex string filters
    TemplateRegistry::register_filter(
        registry,
        "truncate",
        "Truncate a string to a specified length",
        "String Operations",
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let length = args.first().and_then(|v| v.as_i64()).unwrap_or(255) as usize;

            // Handle variable arguments: truncate(length), truncate(length, end), truncate(length, killwords, end)
            let (killwords, end) = if args.len() >= 3 {
                // truncate(length, killwords, end)
                let killwords = args
                    .get(1)
                    .map(|v| v.is_true() || v.as_str() == Some("true"))
                    .unwrap_or(false);
                let end = args
                    .get(2)
                    .and_then(|v| v.as_str())
                    .unwrap_or("...")
                    .to_string();
                (killwords, end)
            } else if args.len() == 2 {
                // truncate(length, end) - assume killwords = false
                let end = args
                    .get(1)
                    .and_then(|v| v.as_str())
                    .unwrap_or("...")
                    .to_string();
                (false, end)
            } else {
                // truncate(length) - default values
                (false, "...".to_string())
            };

            if s.len() <= length {
                JinjaValue::from(s.to_string())
            } else if killwords {
                // Simple truncate at character boundary
                let mut result: String = s.chars().take(length.saturating_sub(end.len())).collect();
                result.push_str(&end);
                JinjaValue::from(result)
            } else {
                // Try to truncate at word boundary
                let words: Vec<&str> = s.split_whitespace().collect();
                let mut result = String::new();
                let mut char_count = 0;

                for word in &words {
                    let word_with_space = if result.is_empty() {
                        word.len()
                    } else {
                        word.len() + 1
                    };
                    if char_count + word_with_space + end.len() > length {
                        break;
                    }
                    if !result.is_empty() {
                        result.push(' ');
                        char_count += 1;
                    }
                    result.push_str(word);
                    char_count += word.len();
                }

                // If we couldn't fit any words, fall back to character truncation
                if result.is_empty() && !words.is_empty() {
                    result = s.chars().take(length.saturating_sub(end.len())).collect();
                }

                result.push_str(&end);
                JinjaValue::from(result)
            }
        }),
    );
}
