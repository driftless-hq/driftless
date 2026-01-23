//! Tests for list and dictionary operation filters

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
    fn test_combine_filter() {
        let env = create_test_env();

        // Test basic combine
        let template = env
            .template_from_str("{{ {'a': 1, 'b': 2} | combine({'b': 3, 'c': 4}) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(
            result.contains("\"a\": 1")
                && result.contains("\"b\": 3")
                && result.contains("\"c\": 4")
        );

        // Test multiple combines
        let template = env
            .template_from_str("{{ {'a': 1} | combine({'b': 2}, {'c': 3}) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(
            result.contains("\"a\": 1")
                && result.contains("\"b\": 2")
                && result.contains("\"c\": 3")
        );
    }

    #[test]
    fn test_dict2items_filter() {
        let env = create_test_env();

        let template = env
            .template_from_str("{{ {'a': 1, 'b': 2} | dict2items }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // The order might vary, so check that it contains the items
        assert!(result.contains("\"key\": \"a\"") && result.contains("\"value\": 1"));
        assert!(result.contains("\"key\": \"b\"") && result.contains("\"value\": 2"));
    }

    #[test]
    fn test_items2dict_filter() {
        let env = create_test_env();

        let template = env
            .template_from_str(
                "{{ [{'key': 'a', 'value': 1}, {'key': 'b', 'value': 2}] | items2dict }}",
            )
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(result.contains("\"a\": 1") && result.contains("\"b\": 2"));
    }

    #[test]
    fn test_flatten_filter() {
        let env = create_test_env();

        // Test basic flatten
        let template = env
            .template_from_str("{{ [1, [2, 3], [4, [5, 6]]] | flatten }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2, 3, 4, 5, 6]");

        // Test already flat
        let template = env.template_from_str("{{ [1, 2, 3] | flatten }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_map_filter() {
        let env = create_test_env();

        // Test map with attribute
        let template = env
            .template_from_str(
                "{{ [{'name': 'alice', 'age': 25}, {'name': 'bob', 'age': 30}] | map('name') }}",
            )
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"alice\", \"bob\"]");

        // Test map with non-existent attribute
        let template = env
            .template_from_str("{{ [{'name': 'alice'}, {'name': 'bob'}] | map('missing') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[none, none]");
    }

    #[test]
    fn test_select_filter() {
        let env = create_test_env();

        // Test select with truthy
        let template = env
            .template_from_str("{{ [0, 1, false, true, '', 'hello'] | select('truthy') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, true, \"hello\"]");

        // Test select with defined
        let template = env
            .template_from_str("{{ [None, 1, None, 2] | select('defined') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2]");

        // Test select with undefined
        let template = env
            .template_from_str("{{ [None, 1, None, 2] | select('undefined') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test select with none
        let template = env
            .template_from_str("{{ [None, 1, None, 2] | select('none') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[none, none]");

        // Test select with falsy
        let template = env
            .template_from_str("{{ [0, 1, false, true, '', 'hello'] | select('falsy') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[0, false, \"\"]");

        // Test select with equalto
        let template = env
            .template_from_str("{{ [1, 2, 3, 2, 1] | select('equalto', 2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[2, 2]");

        // Test select with match
        let template = env
            .template_from_str("{{ ['abc', 'def', 'ghi'] | select('match', '^a') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"abc\"]");

        // Test select with search
        let template = env
            .template_from_str("{{ ['abc', 'def', 'ghi'] | select('search', 'e') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"def\"]");

        // Test select with version_compare
        let template = env
            .template_from_str(
                "{{ ['1.0.0', '1.1.0', '2.0.0'] | select('version_compare', '1.1.0') }}",
            )
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"1.1.0\"]");
    }

    #[test]
    fn test_reject_filter() {
        let env = create_test_env();

        // Test reject with truthy
        let template = env
            .template_from_str("{{ [0, 1, false, true, '', 'hello'] | reject('truthy') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[0, false, \"\"]");

        // Test reject with defined
        let template = env
            .template_from_str("{{ [None, 1, None, 2] | reject('defined') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[none, none]");

        // Test reject with undefined
        let template = env
            .template_from_str("{{ [None, 1, None, 2] | reject('undefined') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[none, 1, none, 2]");

        // Test reject with none
        let template = env
            .template_from_str("{{ [None, 1, None, 2] | reject('none') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2]");

        // Test reject with falsy
        let template = env
            .template_from_str("{{ [0, 1, false, true, '', 'hello'] | reject('falsy') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, true, \"hello\"]");

        // Test reject with equalto
        let template = env
            .template_from_str("{{ [1, 2, 3, 2, 1] | reject('equalto', 2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 3, 1]");

        // Test reject with match
        let template = env
            .template_from_str("{{ ['abc', 'def', 'ghi'] | reject('match', '^a') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"def\", \"ghi\"]");

        // Test reject with search
        let template = env
            .template_from_str("{{ ['abc', 'def', 'ghi'] | reject('search', 'e') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"abc\", \"ghi\"]");

        // Test reject with version_compare
        let template = env
            .template_from_str(
                "{{ ['1.0.0', '1.1.0', '2.0.0'] | reject('version_compare', '1.1.0') }}",
            )
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"1.0.0\", \"2.0.0\"]");
    }

    #[test]
    fn test_zip_filter() {
        let env = create_test_env();

        // Test basic zip
        let template = env
            .template_from_str("{{ [1, 2, 3] | zip([4, 5, 6]) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 4], [2, 5], [3, 6]]");

        // Test zip with different lengths
        let template = env
            .template_from_str("{{ [1, 2] | zip([4, 5, 6]) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 4], [2, 5]]");

        // Test zip with three lists
        let template = env
            .template_from_str("{{ [1, 2] | zip([4, 5], [7, 8]) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 4, 7], [2, 5, 8]]");
    }

    #[test]
    fn test_dict2items_items2dict_roundtrip() {
        let env = create_test_env();

        let original = "{'a': 1, 'b': {'nested': 2}, 'c': [3, 4]}";
        let template_str = format!("{{{{ {} | dict2items | items2dict }}}}", original);
        let template = env.template_from_str(&template_str).unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert!(
            result.contains("\"a\": 1")
                && result.contains("\"b\": {\"nested\": 2}")
                && result.contains("\"c\": [3, 4]")
        );
    }

    #[test]
    fn test_dictsort_filter() {
        let env = create_test_env();

        // Test basic dictsort by key
        let template = env
            .template_from_str("{{ {'c': 3, 'a': 1, 'b': 2} | dictsort }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // Should be sorted by key: a, b, c
        assert!(
            result.contains("\"a\": 1")
                && result.contains("\"b\": 2")
                && result.contains("\"c\": 3")
        );

        // Test dictsort by value
        let template = env
            .template_from_str("{{ {'c': 3, 'a': 1, 'b': 2} | dictsort(by='value') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // Should be sorted by value: a:1, b:2, c:3
        assert!(
            result.contains("\"a\": 1")
                && result.contains("\"b\": 2")
                && result.contains("\"c\": 3")
        );

        // Test dictsort reverse
        let template = env
            .template_from_str("{{ {'a': 1, 'b': 2, 'c': 3} | dictsort(reverse=true) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // Should be reverse sorted: c, b, a
        assert!(
            result.contains("\"c\": 3")
                && result.contains("\"b\": 2")
                && result.contains("\"a\": 1")
        );

        // Test dictsort with case insensitive
        let template = env
            .template_from_str("{{ {'B': 1, 'a': 2, 'C': 3} | dictsort(case_sensitive=false) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        // Should be sorted case insensitively
        assert!(
            result.contains("\"a\": 2")
                && result.contains("\"B\": 1")
                && result.contains("\"C\": 3")
        );
    }

    #[test]
    fn test_slice_filter() {
        let env = create_test_env();

        // Test basic slice
        let template = env
            .template_from_str("{{ [1, 2, 3, 4, 5] | slice(2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 2], [3, 4], [5]]");

        // Test slice with size 1
        let template = env.template_from_str("{{ [1, 2, 3] | slice(1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1], [2], [3]]");

        // Test slice with size larger than list
        let template = env.template_from_str("{{ [1, 2] | slice(5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 2]]");

        // Test slice with empty list
        let template = env.template_from_str("{{ [] | slice(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test slice with size 0
        let template = env.template_from_str("{{ [1, 2, 3] | slice(0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");
    }
}
