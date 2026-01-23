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
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").to_uppercase())),
    );

    TemplateRegistry::register_filter(
        registry,
        "lower",
        "Convert a string to lowercase",
        "String Operations",
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").to_lowercase())),
    );

    TemplateRegistry::register_filter(
        registry,
        "capitalize",
        "Capitalize the first character of a string",
        "String Operations",
        vec![],
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
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.len().unwrap_or(0) as i64)),
    );

    // Complex string filters
    TemplateRegistry::register_filter(
        registry,
        "truncate",
        "Truncate a string to a specified length",
        "String Operations",
        vec![
            ("length".to_string(), "integer: Maximum length of the resulting string".to_string()),
            ("killwords".to_string(), "boolean: If true, truncate at character boundary; if false, try to truncate at word boundary (optional, default: false)".to_string()),
            ("end".to_string(), "string: String to append when truncation occurs (optional, default: \"...\")".to_string()),
        ],
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

    // String justification filters
    TemplateRegistry::register_filter(
        registry,
        "center",
        "Center a string in a field of given width",
        "String Operations",
        vec![
            (
                "width".to_string(),
                "integer: Width of the field".to_string(),
            ),
            (
                "fillchar".to_string(),
                "string: Character to fill with (optional, default: space)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let width = args.first().and_then(|v| v.as_i64()).unwrap_or(0) as usize;
            let fillchar = args
                .get(1)
                .and_then(|v| v.as_str())
                .and_then(|s| s.chars().next())
                .unwrap_or(' ');

            if s.len() >= width {
                JinjaValue::from(s.to_string())
            } else {
                let padding = width - s.len();
                let left_pad = padding / 2;
                let right_pad = padding - left_pad;
                let result = format!(
                    "{}{}{}",
                    fillchar.to_string().repeat(left_pad),
                    s,
                    fillchar.to_string().repeat(right_pad)
                );
                JinjaValue::from(result)
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "ljust",
        "Left-justify a string in a field of given width",
        "String Operations",
        vec![
            (
                "width".to_string(),
                "integer: Width of the field".to_string(),
            ),
            (
                "fillchar".to_string(),
                "string: Character to fill with (optional, default: space)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let width = args.first().and_then(|v| v.as_i64()).unwrap_or(0) as usize;
            let fillchar = args
                .get(1)
                .and_then(|v| v.as_str())
                .and_then(|s| s.chars().next())
                .unwrap_or(' ');

            if s.len() >= width {
                JinjaValue::from(s.to_string())
            } else {
                let padding = width - s.len();
                let result = format!("{}{}", s, fillchar.to_string().repeat(padding));
                JinjaValue::from(result)
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "rjust",
        "Right-justify a string in a field of given width",
        "String Operations",
        vec![
            (
                "width".to_string(),
                "integer: Width of the field".to_string(),
            ),
            (
                "fillchar".to_string(),
                "string: Character to fill with (optional, default: space)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let width = args.first().and_then(|v| v.as_i64()).unwrap_or(0) as usize;
            let fillchar = args
                .get(1)
                .and_then(|v| v.as_str())
                .and_then(|s| s.chars().next())
                .unwrap_or(' ');

            if s.len() >= width {
                JinjaValue::from(s.to_string())
            } else {
                let padding = width - s.len();
                let result = format!("{}{}", fillchar.to_string().repeat(padding), s);
                JinjaValue::from(result)
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "indent",
        "Indent each line of a string",
        "String Operations",
        vec![
            (
                "width".to_string(),
                "integer: Number of spaces to indent (optional, default: 0)".to_string(),
            ),
            (
                "indentfirst".to_string(),
                "boolean: Whether to indent the first line (optional, default: false)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let width = args.first().and_then(|v| v.as_i64()).unwrap_or(0) as usize;
            let indentfirst = args
                .get(1)
                .map(|v| v.is_true() || v.as_str() == Some("true"))
                .unwrap_or(false);

            if width == 0 {
                return JinjaValue::from(s.to_string());
            }

            let indent_str = " ".repeat(width);
            let mut result = String::new();
            let has_trailing_newline = s.ends_with('\n');

            // Split by lines but preserve empty lines
            let lines: Vec<&str> = s.split('\n').collect();

            for (i, line) in lines.iter().enumerate() {
                if i == 0 && !indentfirst {
                    result.push_str(line);
                } else if line.is_empty() {
                    // Don't indent empty lines
                    // Do nothing, just preserve the line break
                } else {
                    result.push_str(&indent_str);
                    result.push_str(line);
                }

                // Add newline if not the last line, or if it's an empty line in the middle
                if i < lines.len() - 1 {
                    result.push('\n');
                }
            }

            // If original string had trailing newline, add it back
            if has_trailing_newline && !result.ends_with('\n') {
                result.push('\n');
            }

            JinjaValue::from(result)
        }),
    );

    // String trimming filters
    TemplateRegistry::register_filter(
        registry,
        "lstrip",
        "Remove leading whitespace from a string",
        "String Operations",
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").trim_start())),
    );

    TemplateRegistry::register_filter(
        registry,
        "rstrip",
        "Remove trailing whitespace from a string",
        "String Operations",
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").trim_end())),
    );

    TemplateRegistry::register_filter(
        registry,
        "strip",
        "Remove leading and trailing whitespace from a string",
        "String Operations",
        vec![],
        Arc::new(|value, _args| JinjaValue::from(value.as_str().unwrap_or("").trim())),
    );

    // String transformation filters
    TemplateRegistry::register_filter(
        registry,
        "title",
        "Convert a string to title case",
        "String Operations",
        vec![],
        Arc::new(|value, _args| {
            let s = value.as_str().unwrap_or("");
            let mut result = String::new();
            let mut capitalize_next = true;

            for ch in s.chars() {
                if ch.is_whitespace() {
                    result.push(ch);
                    capitalize_next = true;
                } else if capitalize_next {
                    result.extend(ch.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.extend(ch.to_lowercase());
                }
            }

            JinjaValue::from(result)
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "splitlines",
        "Split a string into a list of lines",
        "String Operations",
        vec![],
        Arc::new(|value, _args| {
            let s = value.as_str().unwrap_or("");
            let lines: Vec<JinjaValue> = s.lines().map(JinjaValue::from).collect();
            JinjaValue::from(lines)
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "wordcount",
        "Count the number of words in a string",
        "String Operations",
        vec![],
        Arc::new(|value, _args| {
            let s = value.as_str().unwrap_or("");
            let count = s.split_whitespace().filter(|word| !word.is_empty()).count() as i64;
            JinjaValue::from(count)
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "comment",
        "Wrap a string in comment markers",
        "String Operations",
        vec![(
            "style".to_string(),
            "string: Comment style (optional, default: #)".to_string(),
        )],
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let style = args.first().and_then(|v| v.as_str()).unwrap_or("#");
            let lines: Vec<&str> = s.lines().collect();
            if lines.is_empty() {
                JinjaValue::from(format!("{} ", style))
            } else {
                let commented: Vec<String> = lines
                    .iter()
                    .map(|line| format!("{} {}", style, line))
                    .collect();
                JinjaValue::from(commented.join("\n"))
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "format",
        "Format a string with placeholders",
        "String Operations",
        vec![(
            "args".to_string(),
            "variadic: Arguments to format into the string".to_string(),
        )],
        Arc::new(|value, args| {
            let template = value.as_str().unwrap_or("");
            // Simple implementation: replace {} with args in order
            let mut result = template.to_string();
            for arg in args {
                if let Some(arg_str) = arg.as_str() {
                    if let Some(pos) = result.find("{}") {
                        result.replace_range(pos..pos + 2, arg_str);
                    }
                }
            }
            JinjaValue::from(result)
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "wordwrap",
        "Wrap a string to a specified width",
        "String Operations",
        vec![(
            "width".to_string(),
            "integer: Maximum width of each line (optional, default: 79)".to_string(),
        )],
        Arc::new(|value, args| {
            let s = value.as_str().unwrap_or("");
            let width = args.first().and_then(|v| v.as_i64()).unwrap_or(79) as usize;
            let mut result = String::new();
            let mut current_line = String::new();

            for word in s.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + word.len() < width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str(&current_line);
                    current_line = word.to_string();
                }
            }
            if !current_line.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&current_line);
            }
            JinjaValue::from(result)
        }),
    );

    // List operation filters
    TemplateRegistry::register_filter(
        registry,
        "first",
        "Get the first item from a list",
        "List Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Some(_s) = value.as_str() {
                // For strings, return empty
                JinjaValue::from("")
            } else if let Ok(mut iter) = value.try_iter() {
                if let Some(first) = iter.next() {
                    first
                } else {
                    JinjaValue::from("")
                }
            } else {
                JinjaValue::from("")
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "last",
        "Get the last item from a list",
        "List Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Some(_s) = value.as_str() {
                // For strings, return empty
                JinjaValue::from("")
            } else if let Ok(iter) = value.try_iter() {
                let mut last_item = None;
                for item in iter {
                    last_item = Some(item);
                }
                last_item.unwrap_or(JinjaValue::from(""))
            } else {
                JinjaValue::from("")
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "join",
        "Join a list of strings with a separator",
        "List Operations",
        vec![(
            "separator".to_string(),
            "string: String to join with (optional, default: empty string)".to_string(),
        )],
        Arc::new(|value, args| {
            let separator = args.first().and_then(|v| v.as_str()).unwrap_or("");

            if let Ok(iter) = value.try_iter() {
                let strings: Vec<String> = iter
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                JinjaValue::from(strings.join(separator))
            } else {
                JinjaValue::from(value.as_str().unwrap_or(""))
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "reverse",
        "Reverse the order of items in a list",
        "List Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Some(s) = value.as_str() {
                // For strings, reverse the characters
                let reversed: String = s.chars().rev().collect();
                JinjaValue::from(reversed)
            } else if let Ok(iter) = value.try_iter() {
                let mut reversed: Vec<JinjaValue> = iter.collect();
                reversed.reverse();
                JinjaValue::from(reversed)
            } else {
                value.clone()
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "sort",
        "Sort items in a list",
        "List Operations",
        vec![
            (
                "reverse".to_string(),
                "boolean: Sort in reverse order (optional, default: false)".to_string(),
            ),
            (
                "case_sensitive".to_string(),
                "boolean: Case sensitive sorting for strings (optional, default: true)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let reverse = args
                .first()
                .map(|v| v.is_true() || v.as_str() == Some("true"))
                .unwrap_or(false);
            let case_sensitive = args
                .get(1)
                .map(|v| v.is_true() || v.as_str() == Some("true"))
                .unwrap_or(true);

            if let Some(s) = value.as_str() {
                // For strings, sort the characters
                let mut chars: Vec<char> = s.chars().collect();
                chars.sort();
                if reverse {
                    chars.reverse();
                }
                let sorted: String = chars.into_iter().collect();
                JinjaValue::from(sorted)
            } else if let Ok(iter) = value.try_iter() {
                let mut sorted: Vec<JinjaValue> = iter.collect();

                sorted.sort_by(|a, b| {
                    match (a.as_str(), b.as_str()) {
                        (Some(a_str), Some(b_str)) => {
                            if case_sensitive {
                                a_str.cmp(b_str)
                            } else {
                                a_str.to_lowercase().cmp(&b_str.to_lowercase())
                            }
                        }
                        _ => {
                            // For non-string values, convert to string for comparison
                            let a_str = a.to_string();
                            let b_str = b.to_string();
                            if case_sensitive {
                                a_str.cmp(&b_str)
                            } else {
                                a_str.to_lowercase().cmp(&b_str.to_lowercase())
                            }
                        }
                    }
                });

                if reverse {
                    sorted.reverse();
                }

                JinjaValue::from(sorted)
            } else {
                value.clone()
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "unique",
        "Remove duplicate items from a list",
        "List Operations",
        vec![],
        Arc::new(|value, _args| {
            if let Some(_s) = value.as_str() {
                // For strings, return as-is
                value.clone()
            } else if let Ok(iter) = value.try_iter() {
                let mut seen = std::collections::HashSet::new();
                let mut unique: Vec<JinjaValue> = Vec::new();

                for item in iter {
                    // Use string representation for uniqueness check
                    let key = item.to_string();
                    if seen.insert(key) {
                        unique.push(item);
                    }
                }

                JinjaValue::from(unique)
            } else {
                value.clone()
            }
        }),
    );

    TemplateRegistry::register_filter(
        registry,
        "batch",
        "Batch items in a list into groups of a specified size",
        "List Operations",
        vec![
            (
                "size".to_string(),
                "integer: Size of each batch".to_string(),
            ),
            (
                "fill_with".to_string(),
                "any: Value to fill incomplete batches (optional)".to_string(),
            ),
        ],
        Arc::new(|value, args| {
            let size = args.first().and_then(|v| v.as_i64()).unwrap_or(1) as usize;
            let fill_with = args.get(1).cloned();

            if let Some(_s) = value.as_str() {
                // For strings, return empty list
                JinjaValue::from(Vec::<JinjaValue>::new())
            } else if let Ok(iter) = value.try_iter() {
                let mut batches: Vec<JinjaValue> = Vec::new();
                let mut current_batch: Vec<JinjaValue> = Vec::new();

                for item in iter {
                    current_batch.push(item);
                    if current_batch.len() == size {
                        batches.push(JinjaValue::from(current_batch.clone()));
                        current_batch.clear();
                    }
                }

                // Handle remaining items
                if !current_batch.is_empty() {
                    if let Some(fill) = &fill_with {
                        while current_batch.len() < size {
                            current_batch.push(fill.clone());
                        }
                    }
                    batches.push(JinjaValue::from(current_batch));
                }

                JinjaValue::from(batches)
            } else {
                // For non-sequence values, return empty list
                JinjaValue::from(Vec::<JinjaValue>::new())
            }
        }),
    );
}
