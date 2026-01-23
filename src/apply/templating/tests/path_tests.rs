//! Tests for path operation filters and functions

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
    fn test_basename_filter() {
        let env = create_test_env();

        // Test basic basename
        let template = env
            .template_from_str("{{ '/path/to/file.txt' | basename }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "file.txt");

        // Test basename with no directory
        let template = env
            .template_from_str("{{ 'file.txt' | basename }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "file.txt");

        // Test basename with trailing slash (should be empty)
        let template = env
            .template_from_str("{{ '/path/to/dir/' | basename }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test root path
        let template = env.template_from_str("{{ '/' | basename }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_dirname_filter() {
        let env = create_test_env();

        // Test basic dirname
        let template = env
            .template_from_str("{{ '/path/to/file.txt' | dirname }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "/path/to");

        // Test dirname with no directory
        let template = env.template_from_str("{{ 'file.txt' | dirname }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test dirname with trailing slash
        let template = env
            .template_from_str("{{ '/path/to/dir/' | dirname }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "/path/to/dir");

        // Test root path
        let template = env.template_from_str("{{ '/' | dirname }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_basename_function() {
        let env = create_test_env();

        // Test basic basename
        let template = env
            .template_from_str("{{ basename('/path/to/file.txt') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "file.txt");

        // Test basename with no directory
        let template = env.template_from_str("{{ basename('file.txt') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "file.txt");

        // Test basename with trailing slash (should be empty)
        let template = env
            .template_from_str("{{ basename('/path/to/dir/') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test root path
        let template = env.template_from_str("{{ basename('/') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_dirname_function() {
        let env = create_test_env();

        // Test basic dirname
        let template = env
            .template_from_str("{{ dirname('/path/to/file.txt') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "/path/to");

        // Test dirname with no directory
        let template = env.template_from_str("{{ dirname('file.txt') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test dirname with trailing slash
        let template = env
            .template_from_str("{{ dirname('/path/to/dir/') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "/path/to/dir");

        // Test root path
        let template = env.template_from_str("{{ dirname('/') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }
}
