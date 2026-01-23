//! Tests for encoding and decoding filters

#[cfg(test)]
mod tests {
    use crate::apply::templating::setup_minijinja_env;
    use minijinja::Environment;

    fn create_test_env() -> Environment<'static> {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);
        env
    }

    #[test]
    fn test_b64encode_filter() {
        let env = create_test_env();

        // Test basic string encoding
        let template = env
            .template_from_str("{{ 'hello world' | b64encode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "aGVsbG8gd29ybGQ=");

        // Test empty string
        let template = env.template_from_str("{{ '' | b64encode }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "");

        // Test non-string value (should convert to string first)
        let template = env.template_from_str("{{ 42 | b64encode }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "NDI=");
    }

    #[test]
    fn test_b64decode_filter() {
        let env = create_test_env();

        // Test basic decoding
        let template = env
            .template_from_str("{{ 'aGVsbG8gd29ybGQ=' | b64decode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "hello world");

        // Test empty string
        let template = env.template_from_str("{{ '' | b64decode }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "");

        // Test invalid base64 (should return false)
        let template = env
            .template_from_str("{{ 'invalid!' | b64decode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "false");

        // Test non-string input (should return false)
        let template = env.template_from_str("{{ 42 | b64decode }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "false");
    }

    #[test]
    fn test_b64encode_b64decode_roundtrip() {
        let env = create_test_env();

        // Test roundtrip
        let template = env
            .template_from_str("{{ 'test string with spaces' | b64encode | b64decode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "test string with spaces");

        // Test with special characters
        let template = env
            .template_from_str("{{ 'café & naïve résumé' | b64encode | b64decode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "café & naïve résumé");
    }

    #[test]
    fn test_to_json_filter() {
        let env = create_test_env();

        // Test basic object
        let template = env
            .template_from_str("{{ {'name': 'test', 'value': 42} | to_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // JSON output order may vary, so check key components
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"value\""));
        assert!(result.contains("42"));

        // Test array
        let template = env.template_from_str("{{ [1, 2, 3] | to_json }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[1,2,3]");

        // Test string
        let template = env.template_from_str("{{ 'hello' | to_json }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "\"hello\"");

        // Test pretty printing with indent
        let template = env
            .template_from_str("{{ {'a': 1, 'b': 2} | to_json(2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\n"));
        assert!(result.contains("  \""));
    }

    #[test]
    fn test_from_json_filter() {
        let env = create_test_env();

        // Test parsing object
        let template = env
            .template_from_str("{{ '{\"name\": \"test\", \"value\": 42}' | from_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"value\""));
        assert!(result.contains("42"));

        // Test parsing array
        let template = env
            .template_from_str("{{ '[1, 2, 3]' | from_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[1, 2, 3]");

        // Test parsing string
        let template = env
            .template_from_str("{{ '\"hello\"' | from_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "hello");

        // Test invalid JSON (should return false)
        let template = env
            .template_from_str("{{ 'invalid json' | from_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "false");

        // Test non-string input (should return false)
        let template = env.template_from_str("{{ 42 | from_json }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "false");
    }

    #[test]
    fn test_to_json_from_json_roundtrip() {
        let env = create_test_env();

        // Test roundtrip with object
        let template = env
            .template_from_str("{{ {'name': 'test', 'items': [1, 2, 3]} | to_json | from_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"items\""));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_to_yaml_filter() {
        let env = create_test_env();

        // Test basic object
        let template = env
            .template_from_str("{{ {'name': 'test', 'value': 42} | to_yaml }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("name: test"));
        assert!(result.contains("value: 42"));

        // Test array
        let template = env.template_from_str("{{ [1, 2, 3] | to_yaml }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("- 1"));
        assert!(result.contains("- 2"));
        assert!(result.contains("- 3"));
    }

    #[test]
    fn test_from_yaml_filter() {
        let env = create_test_env();

        // Test parsing object
        let template = env
            .template_from_str("{{ 'name: test\\nvalue: 42' | from_yaml }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"value\""));
        assert!(result.contains("42"));

        // Test parsing array
        let template = env
            .template_from_str("{{ '- 1\\n- 2\\n- 3' | from_yaml }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));

        // Test invalid YAML (should return false)
        let template = env
            .template_from_str("{{ 'invalid: yaml: content:' | from_yaml }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "false");

        // Test non-string input (should return false)
        let template = env.template_from_str("{{ 42 | from_yaml }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "false");
    }

    #[test]
    fn test_to_yaml_from_yaml_roundtrip() {
        let env = create_test_env();

        // Test roundtrip with object
        let template = env
            .template_from_str("{{ {'name': 'test', 'items': [1, 2, 3]} | to_yaml | from_yaml }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"items\""));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }
}
