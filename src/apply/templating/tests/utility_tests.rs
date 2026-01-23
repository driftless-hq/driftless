//! Tests for utility functions

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
}
