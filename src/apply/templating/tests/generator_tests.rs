//! Tests for generator functions

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
    fn test_range_function_single_arg() {
        let env = create_test_env();

        // Test range(5) - should generate [0, 1, 2, 3, 4]
        let template = env.template_from_str("{{ range(5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[0, 1, 2, 3, 4]");

        // Test range(0) - should generate []
        let template = env.template_from_str("{{ range(0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[]");

        // Test range(3) - should generate [0, 1, 2]
        let template = env.template_from_str("{{ range(3) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[0, 1, 2]");
    }

    #[test]
    fn test_range_function_two_args() {
        let env = create_test_env();

        // Test range(1, 6) - should generate [1, 2, 3, 4, 5]
        let template = env.template_from_str("{{ range(1, 6) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[1, 2, 3, 4, 5]");

        // Test range(5, 5) - should generate []
        let template = env.template_from_str("{{ range(5, 5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[]");

        // Test range(3, 1) - should generate [] (start >= end)
        let template = env.template_from_str("{{ range(3, 1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[]");
    }

    #[test]
    fn test_range_function_three_args() {
        let env = create_test_env();

        // Test range(1, 10, 2) - should generate [1, 3, 5, 7, 9]
        let template = env.template_from_str("{{ range(1, 10, 2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[1, 3, 5, 7, 9]");

        // Test range(10, 1, -1) - should generate [10, 9, 8, 7, 6, 5, 4, 3, 2]
        let template = env.template_from_str("{{ range(10, 1, -1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[10, 9, 8, 7, 6, 5, 4, 3, 2]");

        // Test range(0, 10, 3) - should generate [0, 3, 6, 9]
        let template = env.template_from_str("{{ range(0, 10, 3) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[0, 3, 6, 9]");
    }

    #[test]
    fn test_range_function_edge_cases() {
        let env = create_test_env();

        // Test range() with no args - should generate []
        let template = env.template_from_str("{{ range() }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[]");

        // Test range with step = 0 - should generate []
        let template = env.template_from_str("{{ range(1, 5, 0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[]");

        // Test range with negative step
        let template = env.template_from_str("{{ range(5, 0, -1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "[5, 4, 3, 2, 1]");
    }

    #[test]
    fn test_random_function_no_args() {
        let env = create_test_env();

        // Test random() - should return a float between 0.0 and 1.0
        let template = env.template_from_str("{{ random() }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // Parse the result as a float and check it's in range
        let random_value: f64 = result.trim().parse().unwrap();
        assert!((0.0..1.0).contains(&random_value));
    }

    #[test]
    fn test_random_function_single_arg() {
        let env = create_test_env();

        // Test random(10) - should return an integer from 0 to 9
        let template = env.template_from_str("{{ random(10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        let random_value: i64 = result.trim().parse().unwrap();
        assert!((0..10).contains(&random_value));

        // Test random(1) - should return 0
        let template = env.template_from_str("{{ random(1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "0");

        // Test random(0) - should return 0
        let template = env.template_from_str("{{ random(0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "0");

        // Test random(-5) - should return 0
        let template = env.template_from_str("{{ random(-5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "0");
    }

    #[test]
    fn test_random_function_two_args() {
        let env = create_test_env();

        // Test random(5, 10) - should return an integer from 5 to 9
        let template = env.template_from_str("{{ random(5, 10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        let random_value: i64 = result.trim().parse().unwrap();
        assert!((5..10).contains(&random_value));

        // Test random(10, 10) - should return 10
        let template = env.template_from_str("{{ random(10, 10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "10");

        // Test random(10, 5) - should return 10 (min >= max)
        let template = env.template_from_str("{{ random(10, 5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "10");
    }

    #[test]
    fn test_random_function_invalid_args() {
        let env = create_test_env();

        // Test random with string arg - should return 0
        let template = env.template_from_str("{{ random('invalid') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "0");

        // Test random with two invalid args - should return 0
        let template = env.template_from_str("{{ random('a', 'b') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result.trim(), "0");
    }

    #[test]
    fn test_range_random_combination() {
        let env = create_test_env();

        // Test using range and random together
        let template = env
            .template_from_str("{{ range(3) | map('add', random(5)) | list }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        // The result should be a list of 3 elements, each between 0 and 4
        // We can't predict exact values, but we can check the structure
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));

        // Parse the result and check each element is in range
        let content = &result[1..result.len() - 1]; // Remove brackets
        if !content.is_empty() {
            for num_str in content.split(',') {
                let num: i64 = num_str.trim().parse().unwrap();
                assert!((0..5).contains(&num));
            }
        }
    }
}
