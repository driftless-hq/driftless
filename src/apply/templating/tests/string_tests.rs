//! Tests for string operation filters and functions

use crate::apply::templating::setup_minijinja_env;
use minijinja::Environment;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_env() -> Environment<'static> {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);
        env
    }

    #[test]
    fn test_upper_filter() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ 'hello world' | upper }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_lower_filter() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ 'HELLO WORLD' | lower }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_capitalize_filter() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ 'hello world' | capitalize }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_truncate_filter() {
        let env = create_test_env();

        // Test basic truncation
        let template = env
            .template_from_str("{{ 'hello world' | truncate(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "he...");

        // Test truncation with custom suffix
        let template = env
            .template_from_str("{{ 'hello world' | truncate(5, '...') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "he...");

        // Test truncation without suffix when length is exact
        let template = env
            .template_from_str("{{ 'hello' | truncate(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test truncation with longer suffix
        let template = env
            .template_from_str("{{ 'hello world' | truncate(5, ' [truncated]') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, " [truncated]");
    }

    #[test]
    fn test_length_filter() {
        let env = create_test_env();
        let template = env.template_from_str("{{ 'hello' | length }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_length_function() {
        let env = create_test_env();
        let template = env.template_from_str("{{ length('hello') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_length_function_array() {
        let env = create_test_env();
        let template = env.template_from_str("{{ length([1, 2, 3]) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_length_function_object() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ length({'a': 1, 'b': 2}) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "2");
    }
}
