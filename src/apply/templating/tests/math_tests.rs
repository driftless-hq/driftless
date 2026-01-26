//! Tests for math and logic filter operations

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
        let result: i64 = template
            .render(minijinja::context!())
            .unwrap()
            .parse()
            .unwrap();
        assert!((0..=10).contains(&result));

        // Test random number with start and end range
        let template = env.template_from_str("{{ 5 | random(10) }}").unwrap();
        let result: i64 = template
            .render(minijinja::context!())
            .unwrap()
            .parse()
            .unwrap();
        assert!((5..=10).contains(&result));

        // Test random from string
        let template = env.template_from_str("{{ 'abc' | random }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!("abc".contains(&result));

        // Test random from list
        let template = env.template_from_str("{{ [1, 2, 3] | random }}").unwrap();
        let result: i64 = template
            .render(minijinja::context!())
            .unwrap()
            .parse()
            .unwrap();
        assert!((1..=3).contains(&result));

        // Test undefined/none values (should fail)
        let template = env.template_from_str("{{ none | random }}").unwrap();
        let result = template.render(minijinja::context!(none => ()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("Error: {}", err);
        assert!(err.to_string().contains("undefined") || err.to_string().contains("random filter"));
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

        // Test undefined/none values (should fail)
        let template = env.template_from_str("{{ none | bool }}").unwrap();
        let result = template.render(minijinja::context!(none => ()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        println!("Error: {}", err);
        assert!(err.to_string().contains("undefined") || err.to_string().contains("bool filter"));
    }

    #[test]
    fn test_ternary_filter() {
        let env = create_test_env();

        // Test true condition
        let template = env
            .template_from_str("{{ true | ternary('yes', 'no') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "yes");

        // Test false condition
        let template = env
            .template_from_str("{{ false | ternary('yes', 'no') }}")
            .unwrap();
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

    #[test]
    fn test_float_filter() {
        let env = create_test_env();

        // Test integer to float
        let template = env.template_from_str("{{ 5 | float }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5.0");

        // Test float to float
        let template = env.template_from_str("{{ 3.14 | float }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.14");

        // Test string to float
        let template = env.template_from_str("{{ '2.5' | float }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "2.5");

        // Test invalid string with default
        let template = env
            .template_from_str("{{ 'invalid' | float(42.0) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "42.0");

        // Test invalid string without default
        let template = env.template_from_str("{{ 'invalid' | float }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0.0");
    }

    #[test]
    fn test_int_filter() {
        let env = create_test_env();

        // Test float to int
        let template = env.template_from_str("{{ 3.7 | int }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");

        // Test int to int
        let template = env.template_from_str("{{ 42 | int }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "42");

        // Test string to int
        let template = env.template_from_str("{{ '123' | int }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "123");

        // Test hex string to int
        let template = env.template_from_str("{{ 'FF' | int(0, 16) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "255");

        // Test invalid string with default
        let template = env.template_from_str("{{ 'invalid' | int(99) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "99");

        // Test invalid string without default
        let template = env.template_from_str("{{ 'invalid' | int }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn test_log_filter() {
        let env = create_test_env();

        // Test natural log
        let template = env.template_from_str("{{ 2.718 | log }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.parse::<f64>().unwrap() > 0.99 && result.parse::<f64>().unwrap() < 1.01);

        // Test log base 10
        let template = env.template_from_str("{{ 100 | log(10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "2.0");

        // Test log base 2
        let template = env.template_from_str("{{ 8 | log(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.0");

        // Test negative number (should return NaN)
        let template = env.template_from_str("{{ -1 | log }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "NaN");
    }

    #[test]
    fn test_pow_filter() {
        let env = create_test_env();

        // Test power
        let template = env.template_from_str("{{ 2 | pow(3) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "8.0");

        // Test square
        let template = env.template_from_str("{{ 3 | pow(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "9.0");

        // Test fractional power
        let template = env.template_from_str("{{ 4 | pow(0.5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "2.0");

        // Test invalid input
        let template = env.template_from_str("{{ 'invalid' | pow(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "NaN");
    }

    #[test]
    fn test_sqrt_filter() {
        let env = create_test_env();

        // Test perfect square
        let template = env.template_from_str("{{ 9 | sqrt }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3.0");

        // Test non-perfect square
        let template = env.template_from_str("{{ 2 | sqrt }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.parse::<f64>().unwrap() > 1.41 && result.parse::<f64>().unwrap() < 1.42);

        // Test zero
        let template = env.template_from_str("{{ 0 | sqrt }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0.0");

        // Test negative number (should return NaN)
        let template = env.template_from_str("{{ -1 | sqrt }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "NaN");
    }

    #[test]
    fn test_range_filter() {
        let env = create_test_env();

        // Test range(end)
        let template = env.template_from_str("{{ 5 | range }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[0, 1, 2, 3, 4]");

        // Test range(start, end)
        let template = env.template_from_str("{{ 2 | range(6) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[2, 3, 4, 5]");

        // Test range(start, end, step)
        let template = env.template_from_str("{{ 1 | range(10, 2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 3, 5, 7, 9]");

        // Test negative step
        let template = env.template_from_str("{{ 10 | range(0, -1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]");

        // Test empty range
        let template = env.template_from_str("{{ 5 | range(5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");
    }
}
