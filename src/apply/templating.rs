//! Shared templating utilities for minijinja setup and rendering

use minijinja::{Environment, Value as JinjaValue};
use std::path::Path;

/// Set up minijinja environment with custom filters and functions
pub fn setup_minijinja_env(env: &mut Environment) {
    // Add custom filters
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
        Path::new(value.as_str().unwrap_or(""))
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    });

    env.add_filter("dirname", |value: JinjaValue| {
        Path::new(value.as_str().unwrap_or(""))
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
            } else {
                if killwords {
                    // Simple truncate at character boundary
                    let mut result: String =
                        s.chars().take(length.saturating_sub(end.len())).collect();
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
            }
        },
    );

    // Add custom functions
    env.add_function("length", |value: JinjaValue| {
        JinjaValue::from(value.len().unwrap_or(0) as i64)
    });

    env.add_function("basename", |value: JinjaValue| {
        JinjaValue::from(
            Path::new(value.as_str().unwrap_or(""))
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
        )
    });

    env.add_function("dirname", |value: JinjaValue| {
        JinjaValue::from(
            Path::new(value.as_str().unwrap_or(""))
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string(),
        )
    });

    env.add_function(
        "lookup",
        |type_str: String, key: Option<String>| -> JinjaValue {
            if type_str == "env" {
                if let Some(key) = key {
                    JinjaValue::from(std::env::var(key).unwrap_or_default())
                } else {
                    JinjaValue::from(String::new())
                }
            } else {
                JinjaValue::from(String::new())
            }
        },
    );
}

/// Render a template with the given context using minijinja
pub fn render_with_context(
    template: &str,
    context: minijinja::Value,
) -> Result<String, minijinja::Error> {
    let mut env = Environment::new();
    setup_minijinja_env(&mut env);

    let tmpl = env.template_from_str(template)?;
    tmpl.render(&context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::Value;
    use std::collections::HashMap;

    fn empty_context() -> Value {
        Value::from(HashMap::<String, Value>::new())
    }

    #[test]
    fn test_length_filter() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Test with string
        let tmpl = env.template_from_str("{{ 'hello'|length }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "5");

        // Test with array
        let tmpl = env.template_from_str("{{ [1,2,3,4]|length }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "4");

        // Test with empty string
        let tmpl = env.template_from_str("{{ ''|length }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "0");

        // Test with empty array
        let tmpl = env.template_from_str("{{ []|length }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "0");
    }

    #[test]
    fn test_upper_filter() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env.template_from_str("{{ 'hello world'|upper }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "HELLO WORLD");

        let tmpl = env.template_from_str("{{ 'Hello'|upper }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "HELLO");

        let tmpl = env.template_from_str("{{ ''|upper }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ '123'|upper }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "123");
    }

    #[test]
    fn test_lower_filter() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env.template_from_str("{{ 'HELLO WORLD'|lower }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "hello world");

        let tmpl = env.template_from_str("{{ 'Hello'|lower }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "hello");

        let tmpl = env.template_from_str("{{ ''|lower }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ '123'|lower }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "123");
    }

    #[test]
    fn test_basename_filter() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ '/path/to/file.txt'|basename }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "file.txt");

        let tmpl = env.template_from_str("{{ 'file.txt'|basename }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "file.txt");

        let tmpl = env.template_from_str("{{ '/path/to/'|basename }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "to");

        let tmpl = env.template_from_str("{{ ''|basename }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ '/'|basename }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_dirname_filter() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ '/path/to/file.txt'|dirname }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "/path/to");

        let tmpl = env.template_from_str("{{ 'file.txt'|dirname }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ '/path/to/'|dirname }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "/path");

        let tmpl = env.template_from_str("{{ '/'|dirname }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ ''|dirname }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_capitalize_filter() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world'|capitalize }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "Hello world");

        let tmpl = env
            .template_from_str("{{ 'HELLO WORLD'|capitalize }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "Hello world");

        let tmpl = env.template_from_str("{{ 'hELLO'|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "Hello");

        let tmpl = env.template_from_str("{{ ''|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ 'a'|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "A");

        let tmpl = env.template_from_str("{{ '123test'|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "123test");
    }

    #[test]
    fn test_truncate_filter_default() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Test default truncation (255 chars)
        let long_text = "a".repeat(300);
        let template_str = format!("{{{{ '{}' | truncate }}}}", long_text);
        let tmpl = env.template_from_str(&template_str).unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result.len(), 255);
        assert!(result.ends_with("..."));

        // Test short text (no truncation)
        let tmpl = env.template_from_str("{{ 'short'|truncate }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "short");
    }

    #[test]
    fn test_truncate_filter_with_length() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world this is a long text'|truncate(10) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "hello...");

        let tmpl = env.template_from_str("{{ 'short'|truncate(10) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "short");
    }

    #[test]
    fn test_truncate_filter_with_killwords_false() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Should truncate at word boundary
        let tmpl = env
            .template_from_str("{{ 'hello world this is a test'|truncate(15, false) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "hello world...");

        // Should truncate at word boundary
        let tmpl = env
            .template_from_str("{{ 'hello world this is a very long test'|truncate(20, false) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "hello world this...");
    }

    #[test]
    fn test_truncate_filter_with_killwords_true() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Should truncate at character boundary
        let tmpl = env
            .template_from_str("{{ 'hello world this is a test'|truncate(15, true) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "hello world ...");
    }

    #[test]
    fn test_truncate_filter_with_custom_end() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world'|truncate(8, true, '***') }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "hello***");
    }

    #[test]
    fn test_truncate_filter_edge_cases() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Empty string
        let tmpl = env.template_from_str("{{ ''|truncate }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        // Length exactly matches
        let tmpl = env.template_from_str("{{ 'hello'|truncate(5) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "hello");

        // Length shorter than end marker
        let tmpl = env.template_from_str("{{ 'hello'|truncate(2) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "...");

        // Single word longer than limit
        let tmpl = env
            .template_from_str("{{ 'supercalifragilisticexpialidocious'|truncate(10, false) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "superca..."); // Falls back to character truncation
    }

    #[test]
    fn test_length_function() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env.template_from_str("{{ length('hello') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "5");

        let tmpl = env.template_from_str("{{ length([1,2,3]) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "3");

        let tmpl = env.template_from_str("{{ length('') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "0");
    }

    #[test]
    fn test_basename_function() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ basename('/path/to/file.txt') }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "file.txt");

        let tmpl = env.template_from_str("{{ basename('file.txt') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "file.txt");

        let tmpl = env.template_from_str("{{ basename('') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_dirname_function() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ dirname('/path/to/file.txt') }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "/path/to");

        let tmpl = env.template_from_str("{{ dirname('file.txt') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ dirname('') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_lookup_function_env() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Set a test environment variable
        std::env::set_var("TEST_VAR", "test_value");

        let tmpl = env
            .template_from_str("{{ lookup('env', 'TEST_VAR') }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "test_value");

        // Clean up
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_lookup_function_env_nonexistent() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ lookup('env', 'NONEXISTENT_VAR') }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_lookup_function_invalid_type() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ lookup('invalid', 'key') }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_lookup_function_insufficient_args() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env.template_from_str("{{ lookup('env') }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");
    }

    #[test]
    fn test_render_with_context_basic() {
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::from("world"));
        context.insert("count".to_string(), Value::from(42));

        let result =
            render_with_context("Hello {{ name }}! Count: {{ count }}", Value::from(context))
                .unwrap();
        assert_eq!(result, "Hello world! Count: 42");
    }

    #[test]
    fn test_render_with_context_with_filters() {
        let mut context = HashMap::new();
        context.insert("text".to_string(), Value::from("hello world"));

        let result =
            render_with_context("{{ text | upper | capitalize }}", Value::from(context)).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_render_with_context_with_functions() {
        let mut context = HashMap::new();
        context.insert("path".to_string(), Value::from("/home/user/file.txt"));

        let result = render_with_context(
            "{{ basename(path) }} in {{ dirname(path) }}",
            Value::from(context),
        )
        .unwrap();
        assert_eq!(result, "file.txt in /home/user");
    }

    #[test]
    fn test_render_with_context_invalid_template() {
        let context = empty_context();
        let result = render_with_context("{{ unclosed", context);
        assert!(result.is_err());
    }

    #[test]
    fn test_render_with_context_undefined_variable() {
        let context = empty_context();
        let result = render_with_context("Hello {{ undefined_var }}", context).unwrap();
        assert_eq!(result, "Hello ");
    }

    #[test]
    fn test_render_with_context_complex_expressions() {
        let mut context = HashMap::new();
        context.insert("items".to_string(), Value::from(vec!["a", "b", "c"]));
        context.insert("text".to_string(), Value::from("HELLO WORLD"));

        let result = render_with_context(
            "Items: {{ items | length }}, Text: {{ text | lower | capitalize }}",
            Value::from(context),
        )
        .unwrap();
        assert_eq!(result, "Items: 3, Text: Hello world");
    }

    #[test]
    fn test_render_with_context_empty_template() {
        let context = empty_context();
        let result = render_with_context("", context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_with_context_no_variables() {
        let context = empty_context();
        let result = render_with_context("Plain text", context).unwrap();
        assert_eq!(result, "Plain text");
    }
}
