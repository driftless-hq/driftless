//! Tests for additional path operation filters

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
    fn test_expanduser_filter() {
        let env = create_test_env();

        // Test basic tilde expansion
        let template = env.template_from_str("{{ '~' | expanduser }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        if let Some(home_dir) = dirs::home_dir() {
            if let Some(home_str) = home_dir.to_str() {
                assert_eq!(result, home_str);
            } else {
                // If home dir has invalid UTF-8, should return original
                assert_eq!(result, "~");
            }
        } else {
            // If no home dir, should return original
            assert_eq!(result, "~");
        }

        // Test tilde with path
        let template = env
            .template_from_str("{{ '~/test' | expanduser }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        if let Some(home_dir) = dirs::home_dir() {
            if let Some(home_str) = home_dir.to_str() {
                let expected = format!("{}/test", home_str);
                assert_eq!(result, expected);
            } else {
                // If home dir has invalid UTF-8, should return original
                assert_eq!(result, "~/test");
            }
        } else {
            // If no home dir, should return original
            assert_eq!(result, "~/test");
        }

        // Test path without tilde (should remain unchanged)
        let template = env
            .template_from_str("{{ '/absolute/path' | expanduser }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "/absolute/path");

        // Test relative path without tilde (should remain unchanged)
        let template = env
            .template_from_str("{{ 'relative/path' | expanduser }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "relative/path");

        // Test ~user syntax (should remain unchanged for now)
        let template = env
            .template_from_str("{{ '~user/path' | expanduser }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "~user/path");

        // Test empty string
        let template = env.template_from_str("{{ '' | expanduser }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test non-string input (should return as-is)
        let template = env.template_from_str("{{ 42 | expanduser }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_realpath_filter() {
        let env = create_test_env();

        // Test with current directory
        let template = env.template_from_str("{{ '.' | realpath }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        match std::fs::canonicalize(".") {
            Ok(canonical) => {
                if let Some(canonical_str) = canonical.to_str() {
                    assert_eq!(result, canonical_str);
                } else {
                    // If canonical path has invalid UTF-8, should return original
                    assert_eq!(result, ".");
                }
            }
            Err(_) => {
                // If canonicalization fails, should return original
                assert_eq!(result, ".");
            }
        }

        // Test with absolute path
        let template = env.template_from_str("{{ '/' | realpath }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        match std::fs::canonicalize("/") {
            Ok(canonical) => {
                if let Some(canonical_str) = canonical.to_str() {
                    assert_eq!(result, canonical_str);
                } else {
                    assert_eq!(result, "/");
                }
            }
            Err(_) => {
                assert_eq!(result, "/");
            }
        }

        // Test with non-existent path (should return original)
        let template = env
            .template_from_str("{{ '/non/existent/path' | realpath }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "/non/existent/path");

        // Test with relative path components
        let template = env
            .template_from_str("{{ '././test/../.' | realpath }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        match std::fs::canonicalize("././test/../.") {
            Ok(canonical) => {
                if let Some(canonical_str) = canonical.to_str() {
                    assert_eq!(result, canonical_str);
                } else {
                    assert_eq!(result, "././test/../.");
                }
            }
            Err(_) => {
                assert_eq!(result, "././test/../.");
            }
        }

        // Test with empty string
        let template = env.template_from_str("{{ '' | realpath }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test non-string input (should return as-is)
        let template = env.template_from_str("{{ 42 | realpath }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_expanduser_realpath_combination() {
        let env = create_test_env();

        // Test combining expanduser and realpath
        let template = env
            .template_from_str("{{ '~' | expanduser | realpath }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();

        if let Some(home_dir) = dirs::home_dir() {
            match std::fs::canonicalize(&home_dir) {
                Ok(canonical) => {
                    if let Some(canonical_str) = canonical.to_str() {
                        assert_eq!(result, canonical_str);
                    } else {
                        // If canonical path has invalid UTF-8, should return expanded home
                        if let Some(home_str) = home_dir.to_str() {
                            assert_eq!(result, home_str);
                        }
                    }
                }
                Err(_) => {
                    // If canonicalization fails, should return expanded home
                    if let Some(home_str) = home_dir.to_str() {
                        assert_eq!(result, home_str);
                    }
                }
            }
        }
    }
}
