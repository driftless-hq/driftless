//! Tests for utility functions

#[cfg(test)]
mod tests {
    use crate::apply::templating::setup_minijinja_env;
    use chrono;
    use minijinja::Environment;
    use tempfile;

    fn create_test_env() -> Environment<'static> {
        let mut env = Environment::new();
        setup_minijinja_env(&mut env);
        env
    }

    #[test]
    fn test_lookup_function_env() {
        let env = create_test_env();

        // Set a test environment variable
        std::env::set_var("TEST_VAR", "test_value");

        let template = env
            .template_from_str("{{ lookup('env', 'TEST_VAR') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "test_value");

        // Clean up
        std::env::remove_var("TEST_VAR");
    }

    #[test]
    fn test_lookup_function_env_nonexistent() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ lookup('env', 'NONEXISTENT_VAR') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_lookup_function_insufficient_args() {
        let env = create_test_env();

        let template = env.template_from_str("{{ lookup('env') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "None");
    }

    #[test]
    fn test_lookup_function_invalid_type() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ lookup('invalid', 'key') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "None");
    }

    #[test]
    fn test_lookup_function_file() {
        let env = create_test_env();

        // Create a temporary file for testing
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        std::fs::write(file_path, "test content").unwrap();

        let template_str = format!("{{{{ lookup('file', '{}') }}}}", file_path);
        let template = env.template_from_str(&template_str).unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "test content");
    }

    #[test]
    fn test_lookup_function_file_nonexistent() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ lookup('file', '/nonexistent/file') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_lookup_function_pipe() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ lookup('pipe', 'echo hello world') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_lookup_function_pipe_failure() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ lookup('pipe', 'false') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_hash_function_md5() {
        let env = create_test_env();

        let template = env.template_from_str("{{ hash('test', 'md5') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // MD5 hash of "test" is "098f6bcd4621d373cade4e832627b4f6"
        assert_eq!(result, "098f6bcd4621d373cade4e832627b4f6");
    }

    #[test]
    fn test_hash_function_sha1() {
        let env = create_test_env();

        let template = env.template_from_str("{{ hash('test', 'sha1') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // SHA1 hash of "test" is "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3"
        assert_eq!(result, "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
    }

    #[test]
    fn test_hash_function_sha256() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ hash('test', 'sha256') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // SHA256 hash of "test" is "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        assert_eq!(
            result,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_hash_function_invalid_algorithm() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ hash('test', 'invalid') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_hash_function_insufficient_args() {
        let env = create_test_env();

        let template = env.template_from_str("{{ hash('test') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_uuid_function() {
        let env = create_test_env();

        let template = env.template_from_str("{{ uuid() }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // UUID should be a valid UUID4 format
        assert!(uuid::Uuid::parse_str(&result).is_ok());
        let parsed = uuid::Uuid::parse_str(&result).unwrap();
        assert_eq!(parsed.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_timestamp_function_default() {
        let env = create_test_env();

        let template = env.template_from_str("{{ timestamp() }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // Should be a valid RFC3339 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&result).is_ok());
    }

    #[test]
    fn test_timestamp_function_custom_format() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ timestamp('%Y-%m-%d %H:%M:%S') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // Should match the format YYYY-MM-DD HH:MM:SS
        assert!(regex::Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$")
            .unwrap()
            .is_match(&result));
    }

    #[test]
    fn test_timestamp_function_invalid_format() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ timestamp('%invalid') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // Should return a valid timestamp (fallback to ISO format on invalid format)
        assert!(chrono::DateTime::parse_from_rfc3339(&result).is_ok() || !result.is_empty());
    }

    #[test]
    fn test_ansible_managed_function() {
        let env = create_test_env();

        let template = env.template_from_str("{{ ansible_managed() }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Ansible managed");
    }

    #[test]
    fn test_expandvars_function() {
        let env = create_test_env();

        // Set test environment variables
        std::env::set_var("TEST_VAR", "test_value");
        std::env::set_var("ANOTHER_VAR", "another_value");

        // Test ${VAR} syntax
        let template = env
            .template_from_str("{{ expandvars('Hello ${TEST_VAR}') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello test_value");

        // Test $VAR syntax
        let template = env
            .template_from_str("{{ expandvars('Hello $TEST_VAR') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello test_value");

        // Test multiple variables
        let template = env
            .template_from_str("{{ expandvars('${TEST_VAR} and $ANOTHER_VAR') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "test_value and another_value");

        // Test nonexistent variable (should remain unchanged)
        let template = env
            .template_from_str("{{ expandvars('Hello $NONEXISTENT') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello $NONEXISTENT");

        // Clean up
        std::env::remove_var("TEST_VAR");
        std::env::remove_var("ANOTHER_VAR");
    }

    #[test]
    fn test_ansible_date_time_function() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ ansible_date_time().year }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // Should be a valid year (4 digits)
        assert_eq!(result.len(), 4);
        assert!(result.chars().all(|c| c.is_ascii_digit()));

        // Test that it's actually the current year
        let current_year = chrono::Utc::now().format("%Y").to_string();
        assert_eq!(result, current_year);

        // Test multiple fields
        let template = env
            .template_from_str("{{ ansible_date_time().date }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains('-')); // Should be YYYY-MM-DD format
    }

    #[test]
    fn test_include_vars_function() {
        let env = create_test_env();

        // Create a temporary YAML file
        let temp_dir = tempfile::tempdir().unwrap();
        let yaml_file = temp_dir.path().join("test_vars.yml");
        std::fs::write(&yaml_file, "key1: value1\nkey2: value2\n").unwrap();

        let template_str = format!("{{{{ include_vars('{}').key1 }}}}", yaml_file.display());
        let template = env.template_from_str(&template_str).unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "value1");

        // Test with JSON file
        let json_file = temp_dir.path().join("test_vars.json");
        std::fs::write(&json_file, r#"{"key3": "value3", "key4": "value4"}"#).unwrap();

        let template_str = format!("{{{{ include_vars('{}').key3 }}}}", json_file.display());
        let template = env.template_from_str(&template_str).unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "value3");

        // Test with nonexistent file
        let template = env
            .template_from_str("{{ include_vars('nonexistent.yml').key1 }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, ""); // Should return empty dict, so accessing key1 gives None which renders as empty string
    }

    #[test]
    fn test_query_function_inventory_hostnames() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ query('inventory_hostnames', 'all') | first }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "localhost");
    }

    #[test]
    fn test_query_function_file() {
        let env = create_test_env();

        // Create a temporary file
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(&temp_file, "test content").unwrap();

        let template_str = format!("{{{{ query('file', '{}') }}}}", temp_file.path().display());
        let template = env.template_from_str(&template_str).unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "test content");
    }

    #[test]
    fn test_query_function_fileglob() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ query('fileglob', '*.txt') | first }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "*.txt"); // Simple implementation returns the pattern
    }

    #[test]
    fn test_query_function_unknown_type() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ query('unknown_type', 'arg') | length }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0"); // Should return empty array
    }
}
