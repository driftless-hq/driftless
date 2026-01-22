//! Custom Jinja2 filters for templating

use minijinja::{Environment, Value as JinjaValue};
use std::path::Path;

/// Add all custom filters to the minijinja environment
pub fn add_filters(env: &mut Environment) {
    env.add_filter("length", |value: JinjaValue| {
        value.len().unwrap_or(0) as i64
    });

    env.add_filter("upper", |value: JinjaValue| {
        value.as_str().unwrap_or("").to_uppercase()
    });

    env.add_filter("lower", |value: JinjaValue| {
        value.as_str().unwrap_or("").to_lowercase()
    });

    env.add_filter("basename", |value: JinjaValue| {
        let path_str = value.as_str().unwrap_or("");
        if path_str.ends_with('/') && path_str != "/" {
            // For paths ending with / (except root), basename is empty
            String::new()
        } else {
            Path::new(path_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
        }
    });

    env.add_filter("dirname", |value: JinjaValue| {
        let path_str = value.as_str().unwrap_or("");
        if path_str.is_empty() {
            return String::new();
        }
        // For paths ending with /, dirname is the path without the trailing /
        if path_str.ends_with('/') {
            return path_str.trim_end_matches('/').to_string();
        }
        // Otherwise, use Path::parent()
        Path::new(path_str)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string()
    });

    env.add_filter("capitalize", |value: JinjaValue| {
        let s = value.as_str().unwrap_or("");
        if s.is_empty() {
            String::new()
        } else {
            let mut chars = s.chars();
            let first = chars.next().unwrap().to_uppercase().collect::<String>();
            let rest = chars.as_str().to_lowercase();
            format!("{}{}", first, rest)
        }
    });

    env.add_filter(
        "truncate",
        |value: JinjaValue, length: Option<i64>, killwords: Option<bool>, end: Option<String>| {
            let s = value.as_str().unwrap_or("");
            let length = length.unwrap_or(255) as usize;
            let killwords = killwords.unwrap_or(false);
            let end = end.unwrap_or("...".to_string());

            if s.len() <= length {
                s.to_string()
            } else if killwords {
                // Simple truncate at character boundary
                let mut result: String = s.chars().take(length.saturating_sub(end.len())).collect();
                result.push_str(&end);
                result
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
                result
            }
        },
    );

    // Add new filters here (e.g., center, indent, etc.)
    // Group by category as per DESIGN.md for clarity
}
