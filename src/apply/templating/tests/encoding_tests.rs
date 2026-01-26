//! Tests for encoding and decoding filters

#[cfg(test)]
mod tests {
    use crate::apply::templating::setup_minijinja_env;
    use minijinja::Environment;

    fn create_test_env() -> Environment<'static> {
        let mut env = Environment::new();
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
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

    #[test]
    fn test_mandatory_filter() {
        let env = create_test_env();

        // Test with defined value
        let template = env.template_from_str("{{ 'hello' | mandatory }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test with empty string (should fail)
        let template = env.template_from_str("{{ '' | mandatory }}").unwrap();
        let result = template.render(minijinja::context!());
        assert!(result.is_err());

        // Test with empty list (should fail)
        let template = env.template_from_str("{{ [] | mandatory }}").unwrap();
        let result = template.render(minijinja::context!());
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_escape_filter() {
        let env = create_test_env();

        // Test basic escaping
        let template = env
            .template_from_str("{{ 'hello.world' | regex_escape }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello\\.world");

        // Test with special characters
        let template = env
            .template_from_str("{{ '[a-z]+' | regex_escape }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "\\[a\\-z\\]\\+");

        // Test empty string
        let template = env.template_from_str("{{ '' | regex_escape }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_regex_findall_filter() {
        let env = create_test_env();

        // Test basic findall
        let template = env
            .template_from_str("{{ 'hello world test' | regex_findall(\"\\\\w+\") }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"hello\", \"world\", \"test\"]");

        // Test with no matches
        let template = env
            .template_from_str("{{ 'hello' | regex_findall(\"\\\\d+\") }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test with invalid regex
        let template = env
            .template_from_str("{{ 'hello' | regex_findall('[invalid') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_regex_replace_filter() {
        let env = create_test_env();

        // Test basic replace
        let template = env
            .template_from_str("{{ 'hello world' | regex_replace('world', 'universe') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello universe");

        // Test replace all
        let template = env
            .template_from_str("{{ 'test 123 test 456' | regex_replace(\"\\\\d+\", 'NUM') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "test NUM test NUM");

        // Test with invalid regex
        let template = env
            .template_from_str("{{ 'hello' | regex_replace('[invalid', 'X') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_regex_search_filter() {
        let env = create_test_env();

        // Test basic search
        let template = env
            .template_from_str("{{ 'hello world' | regex_search('world') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "world");

        // Test no match
        let template = env
            .template_from_str("{{ 'hello' | regex_search('world') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");

        // Test with invalid regex
        let template = env
            .template_from_str("{{ 'hello' | regex_search('[invalid') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_to_nice_json_filter() {
        let env = create_test_env();

        // Test basic formatting
        let template = env
            .template_from_str("{{ {'name': 'test', 'items': [1, 2, 3]} | to_nice_json }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\"name\""));
        assert!(result.contains("\"test\""));
        assert!(result.contains("\"items\""));
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
        // Should have proper indentation
        assert!(result.contains("\n  "));
    }

    #[test]
    fn test_to_nice_yaml_filter() {
        let env = create_test_env();

        // Test basic formatting
        let template = env
            .template_from_str("{{ {'name': 'test', 'items': [1, 2, 3]} | to_nice_yaml }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("name:"));
        assert!(result.contains("test"));
        assert!(result.contains("items:"));
        assert!(result.contains("- 1"));
        assert!(result.contains("- 2"));
        assert!(result.contains("- 3"));
    }

    #[test]
    fn test_urlencode_filter() {
        let env = create_test_env();

        // Test basic encoding
        let template = env
            .template_from_str("{{ 'hello world' | urlencode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello%20world");

        // Test with special characters
        let template = env
            .template_from_str("{{ 'hello/world?query=value' | urlencode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello%2Fworld%3Fquery%3Dvalue");

        // Test empty string
        let template = env.template_from_str("{{ '' | urlencode }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_urldecode_filter() {
        let env = create_test_env();

        // Test basic decoding
        let template = env
            .template_from_str("{{ 'hello%20world' | urldecode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");

        // Test with special characters
        let template = env
            .template_from_str("{{ 'hello%2Fworld%3Fquery%3Dvalue' | urldecode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello/world?query=value");

        // Test invalid encoding
        let template = env
            .template_from_str("{{ 'hello%XX' | urldecode }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello%XX"); // Should return original on error

        // Test empty string
        let template = env.template_from_str("{{ '' | urldecode }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }
}
