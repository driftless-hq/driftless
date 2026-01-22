//! Shared templating utilities for minijinja setup and rendering

pub mod filters;
pub mod functions;

use minijinja::Environment;

/// Set up minijinja environment with custom filters and functions
pub fn setup_minijinja_env(env: &mut Environment) {
    filters::add_filters(env);
    functions::add_functions(env);
}

/// Render a template with the given context using minijinja
pub fn render_with_context(
    template: &str,
    context: minijinja::Value,
) -> Result<String, minijinja::Error> {
    let mut env = Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
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
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ ''|basename }}").unwrap();
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
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "/path/to");

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

        let tmpl = env.template_from_str("{{ 'h'|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "H");

        let tmpl = env.template_from_str("{{ ''|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        let tmpl = env.template_from_str("{{ '123abc'|capitalize }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "123abc");
    }

    #[test]
    fn test_truncate_filter_default() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world this is a long string'|truncate }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert_eq!(result, "hello world this is a long string"); // Should not truncate since < 255
    }

    #[test]
    fn test_truncate_filter_with_length() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world this is a long string'|truncate(20) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert!(result.len() <= 23); // 20 + 3 for "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_filter_with_custom_end() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world'|truncate(10, true, '***') }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "hello w***");
    }

    #[test]
    fn test_truncate_filter_with_killwords_false() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world this is a test'|truncate(15, false) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert!(result.len() <= 18); // 15 + 3 for "..."
        assert!(result.ends_with("..."));
        // Should break at word boundary
        assert!(!result.contains("test..."));
    }

    #[test]
    fn test_truncate_filter_with_killwords_true() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env
            .template_from_str("{{ 'hello world this is a test'|truncate(15, true) }}")
            .unwrap();
        let result = tmpl.render(&empty_context()).unwrap();
        assert!(result.len() <= 18); // 15 + 3 for "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_filter_edge_cases() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        // Short string
        let tmpl = env.template_from_str("{{ 'hi'|truncate(10) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "hi");

        // Empty string
        let tmpl = env.template_from_str("{{ ''|truncate(10) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "");

        // Length exactly matches
        let tmpl = env.template_from_str("{{ 'hello'|truncate(5) }}").unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "hello");

        // Length shorter than end string
        let tmpl = env
            .template_from_str("{{ 'hello world'|truncate(2) }}")
            .unwrap();
        assert_eq!(tmpl.render(&empty_context()).unwrap(), "...");
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
    fn test_lookup_function_insufficient_args() {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);

        let tmpl = env.template_from_str("{{ lookup('env') }}").unwrap();
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
    fn test_render_with_context_basic() {
        let mut context = HashMap::new();
        context.insert("name".to_string(), Value::from("world"));
        let context = Value::from(context);

        let result = render_with_context("Hello {{ name }}!", context).unwrap();
        assert_eq!(result, "Hello world!");
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
        let result = render_with_context("Hello world", context).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_render_with_context_undefined_variable() {
        let context = empty_context();
        assert!(render_with_context("{{ undefined_var }}", context).is_err());
    }

    #[test]
    fn test_render_with_context_invalid_template() {
        let context = empty_context();
        assert!(render_with_context("{{ unclosed", context).is_err());
    }

    #[test]
    fn test_render_with_context_complex_expressions() {
        let mut context = HashMap::new();
        context.insert("items".to_string(), Value::from(vec![1, 2, 3, 4, 5]));
        let context = Value::from(context);

        let result = render_with_context("Count: {{ items|length }}", context).unwrap();
        assert_eq!(result, "Count: 5");
    }

    #[test]
    fn test_render_with_context_with_filters() {
        let mut context = HashMap::new();
        context.insert("text".to_string(), Value::from("hello world"));
        let context = Value::from(context);

        let result = render_with_context("{{ text|upper }}", context).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_render_with_context_with_functions() {
        let context = empty_context();
        let result = render_with_context("{{ length('test') }}", context).unwrap();
        assert_eq!(result, "4");
    }
}
