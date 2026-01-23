//! Tests for math and logic filter operations

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
    fn test_abs_filter() {
        let env = create_test_env();

        // Test positive integer
        let template = env.template_from_str("{{ 5 | abs }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");

        // Test negative integer
        let template = env.template_from_str("{{ -5 | abs }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");

        // Test positive float
        let template = env.template_from_str("{{ 3.14 | abs }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.14");

        // Test negative float
        let template = env.template_from_str("{{ -3.14 | abs }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.14");

        // Test zero
        let template = env.template_from_str("{{ 0 | abs }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0");

        // Test string (should return 0)
        let template = env.template_from_str("{{ 'hello' | abs }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn test_round_filter() {
        let env = create_test_env();

        // Test default rounding (0 decimal places)
        let template = env.template_from_str("{{ 3.14159 | round }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");

        // Test rounding to 2 decimal places
        let template = env.template_from_str("{{ 3.14159 | round(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.14");

        // Test rounding to 4 decimal places
        let template = env.template_from_str("{{ 3.14159 | round(4) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.1416");

        // Test integer (no change)
        let template = env.template_from_str("{{ 5 | round(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");

        // Test negative precision
        let template = env.template_from_str("{{ 123.456 | round(-1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "120");

        // Test string (should return 0)
        let template = env.template_from_str("{{ 'hello' | round }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn test_random_filter() {
        let env = create_test_env();

        // Test random number with end range
        let template = env.template_from_str("{{ 10 | random }}").unwrap();
        let result: i64 = template.render(minijinja::context!()).unwrap().parse().unwrap();
        assert!(result >= 0 && result <= 10);

        // Test random number with start and end range
        let template = env.template_from_str("{{ 5 | random(10) }}").unwrap();
        let result: i64 = template.render(minijinja::context!()).unwrap().parse().unwrap();
        assert!(result >= 5 && result <= 10);

        // Test random from string
        let template = env.template_from_str("{{ 'abc' | random }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!("abc".contains(&result));

        // Test random from list
        let template = env.template_from_str("{{ [1, 2, 3] | random }}").unwrap();
        let result: i64 = template.render(minijinja::context!()).unwrap().parse().unwrap();
        assert!(result >= 1 && result <= 3);

        // Test default random (0-100)
        let template = env.template_from_str("{{ none | random }}").unwrap();
        let result: i64 = template.render(minijinja::context!()).unwrap().parse().unwrap();
        assert!(result >= 0 && result <= 100);
    }

    #[test]
    fn test_bool_filter() {
        let env = create_test_env();

        // Test truthy values
        let template = env.template_from_str("{{ 1 | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "true");

        let template = env.template_from_str("{{ 'hello' | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "true");

        let template = env.template_from_str("{{ [1, 2] | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "true");

        // Test falsy values
        let template = env.template_from_str("{{ 0 | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");

        let template = env.template_from_str("{{ '' | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");

        let template = env.template_from_str("{{ [] | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");

        let template = env.template_from_str("{{ none | bool }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");
    }

    #[test]
    fn test_ternary_filter() {
        let env = create_test_env();

        // Test true condition
        let template = env.template_from_str("{{ true | ternary('yes', 'no') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "yes");

        // Test false condition
        let template = env.template_from_str("{{ false | ternary('yes', 'no') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "no");

        // Test with numbers
        let template = env.template_from_str("{{ 1 | ternary(10, 20) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "10");

        let template = env.template_from_str("{{ 0 | ternary(10, 20) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "20");

        // Test with default values when args not provided
        let template = env.template_from_str("{{ true | ternary }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "true");

        let template = env.template_from_str("{{ false | ternary }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "false");
    }
}