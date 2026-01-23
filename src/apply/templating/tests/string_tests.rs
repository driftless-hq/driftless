//! Tests for string operation filters and functions

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
    fn test_upper_filter() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ 'hello world' | upper }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "HELLO WORLD");
    }

    #[test]
    fn test_lower_filter() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ 'HELLO WORLD' | lower }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_capitalize_filter() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ 'hello world' | capitalize }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_truncate_filter() {
        let env = create_test_env();

        // Test basic truncation
        let template = env
            .template_from_str("{{ 'hello world' | truncate(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "he...");

        // Test truncation with custom suffix
        let template = env
            .template_from_str("{{ 'hello world' | truncate(5, '...') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "he...");

        // Test truncation without suffix when length is exact
        let template = env
            .template_from_str("{{ 'hello' | truncate(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test truncation with longer suffix
        let template = env
            .template_from_str("{{ 'hello world' | truncate(5, ' [truncated]') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, " [truncated]");
    }

    #[test]
    fn test_length_filter() {
        let env = create_test_env();
        let template = env.template_from_str("{{ 'hello' | length }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_length_function() {
        let env = create_test_env();
        let template = env.template_from_str("{{ length('hello') }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "5");
    }

    #[test]
    fn test_length_function_array() {
        let env = create_test_env();
        let template = env.template_from_str("{{ length([1, 2, 3]) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");
    }

    #[test]
    fn test_length_function_object() {
        let env = create_test_env();
        let template = env
            .template_from_str("{{ length({'a': 1, 'b': 2}) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "2");
    }

    #[test]
    fn test_center_filter() {
        let env = create_test_env();

        // Test basic centering
        let template = env.template_from_str("{{ 'hello' | center(10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "  hello   ");

        // Test centering with custom fill character
        let template = env
            .template_from_str("{{ 'hello' | center(10, '*') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "**hello***");

        // Test centering when string is longer than width
        let template = env
            .template_from_str("{{ 'hello world' | center(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");

        // Test centering with width 0
        let template = env.template_from_str("{{ 'hello' | center(0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test centering empty string
        let template = env.template_from_str("{{ '' | center(5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "     ");
    }

    #[test]
    fn test_ljust_filter() {
        let env = create_test_env();

        // Test basic left justification
        let template = env.template_from_str("{{ 'hello' | ljust(10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello     ");

        // Test left justification with custom fill character
        let template = env
            .template_from_str("{{ 'hello' | ljust(10, '*') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello*****");

        // Test left justification when string is longer than width
        let template = env
            .template_from_str("{{ 'hello world' | ljust(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");

        // Test left justification with width 0
        let template = env.template_from_str("{{ 'hello' | ljust(0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test left justification empty string
        let template = env.template_from_str("{{ '' | ljust(5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "     ");
    }

    #[test]
    fn test_rjust_filter() {
        let env = create_test_env();

        // Test basic right justification
        let template = env.template_from_str("{{ 'hello' | rjust(10) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "     hello");

        // Test right justification with custom fill character
        let template = env
            .template_from_str("{{ 'hello' | rjust(10, '*') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "*****hello");

        // Test right justification when string is longer than width
        let template = env
            .template_from_str("{{ 'hello world' | rjust(5) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world");

        // Test right justification with width 0
        let template = env.template_from_str("{{ 'hello' | rjust(0) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test right justification empty string
        let template = env.template_from_str("{{ '' | rjust(5) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "     ");
    }

    #[test]
    fn test_indent_filter() {
        let env = create_test_env();

        // Test basic indentation
        let template = env
            .template_from_str("{{ 'hello\nworld' | indent(2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello\n  world");

        // Test indentation with indentfirst=true
        let template = env
            .template_from_str("{{ 'hello\nworld' | indent(2, true) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "  hello\n  world");

        // Test indentation with width 0
        let template = env
            .template_from_str("{{ 'hello\nworld' | indent(0) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello\nworld");

        // Test indentation with single line
        let template = env.template_from_str("{{ 'hello' | indent(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test indentation with single line and indentfirst=true
        let template = env
            .template_from_str("{{ 'hello' | indent(2, true) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "  hello");

        // Test indentation with empty string
        let template = env.template_from_str("{{ '' | indent(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test indentation with multiple lines including empty lines
        let template = env
            .template_from_str("{{ 'hello\n\nworld' | indent(2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello\n\n  world");

        // Test indentation with trailing newline
        let template = env
            .template_from_str("{{ 'hello\nworld\n' | indent(2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello\n  world\n");
    }

    #[test]
    fn test_lstrip_filter() {
        let env = create_test_env();

        // Test basic left strip
        let template = env
            .template_from_str("{{ '  hello world  ' | lstrip }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world  ");

        // Test left strip with no leading whitespace
        let template = env
            .template_from_str("{{ 'hello world  ' | lstrip }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello world  ");

        // Test left strip with only whitespace
        let template = env.template_from_str("{{ '   ' | lstrip }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test left strip with empty string
        let template = env.template_from_str("{{ '' | lstrip }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_rstrip_filter() {
        let env = create_test_env();

        // Test basic right strip
        let template = env
            .template_from_str("{{ '  hello world  ' | rstrip }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "  hello world");

        // Test right strip with no trailing whitespace
        let template = env
            .template_from_str("{{ '  hello world' | rstrip }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "  hello world");

        // Test right strip with only whitespace
        let template = env.template_from_str("{{ '   ' | rstrip }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test right strip with empty string
        let template = env.template_from_str("{{ '' | rstrip }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_title_filter() {
        let env = create_test_env();

        // Test basic title case
        let template = env
            .template_from_str("{{ 'hello world' | title }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello World");

        // Test title case with mixed case
        let template = env
            .template_from_str("{{ 'HELLO WORLD' | title }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello World");

        // Test title case with multiple spaces
        let template = env
            .template_from_str("{{ 'hello   world' | title }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello   World");

        // Test title case with empty string
        let template = env.template_from_str("{{ '' | title }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test title case with single word
        let template = env.template_from_str("{{ 'hello' | title }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_splitlines_filter() {
        let env = create_test_env();

        // Test basic splitlines
        let template = env
            .template_from_str("{{ 'hello\nworld\ntest' | splitlines }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"hello\", \"world\", \"test\"]");

        // Test splitlines with trailing newline
        let template = env
            .template_from_str("{{ 'hello\nworld\n' | splitlines }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"hello\", \"world\"]");

        // Test splitlines with empty string
        let template = env.template_from_str("{{ '' | splitlines }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test splitlines with no newlines
        let template = env
            .template_from_str("{{ 'hello world' | splitlines }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"hello world\"]");
    }

    #[test]
    fn test_wordcount_filter() {
        let env = create_test_env();

        // Test basic word count
        let template = env
            .template_from_str("{{ 'hello world test' | wordcount }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");

        // Test word count with multiple spaces
        let template = env
            .template_from_str("{{ 'hello   world    test' | wordcount }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");

        // Test word count with empty string
        let template = env.template_from_str("{{ '' | wordcount }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0");

        // Test word count with only spaces
        let template = env.template_from_str("{{ '   ' | wordcount }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "0");

        // Test word count with single word
        let template = env.template_from_str("{{ 'hello' | wordcount }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_first_filter() {
        let env = create_test_env();

        // Test first with list
        let template = env.template_from_str("{{ [1, 2, 3] | first }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "1");

        // Test first with empty list
        let template = env.template_from_str("{{ [] | first }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test first with string (should return empty)
        let template = env.template_from_str("{{ 'hello' | first }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_last_filter() {
        let env = create_test_env();

        // Test last with list
        let template = env.template_from_str("{{ [1, 2, 3] | last }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "3");

        // Test last with empty list
        let template = env.template_from_str("{{ [] | last }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test last with string (should return empty)
        let template = env.template_from_str("{{ 'hello' | last }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_join_filter() {
        let env = create_test_env();

        // Test join with default separator
        let template = env
            .template_from_str("{{ ['hello', 'world', 'test'] | join }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "helloworldtest");

        // Test join with custom separator
        let template = env
            .template_from_str("{{ ['hello', 'world', 'test'] | join(', ') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello, world, test");

        // Test join with empty list
        let template = env.template_from_str("{{ [] | join }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");

        // Test join with single item
        let template = env.template_from_str("{{ ['hello'] | join }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");

        // Test join with string (returns the string)
        let template = env.template_from_str("{{ 'hello' | join }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_reverse_filter() {
        let env = create_test_env();

        // Test reverse with list
        let template = env.template_from_str("{{ [1, 2, 3] | reverse }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[3, 2, 1]");

        // Test reverse with empty list
        let template = env.template_from_str("{{ [] | reverse }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test reverse with string
        let template = env.template_from_str("{{ 'hello' | reverse }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "olleh");

        // Test reverse with empty string
        let template = env.template_from_str("{{ '' | reverse }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_sort_filter() {
        let env = create_test_env();

        // Test sort with list of numbers
        let template = env.template_from_str("{{ [3, 1, 2] | sort }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2, 3]");

        // Test sort with list of strings
        let template = env
            .template_from_str("{{ ['c', 'a', 'b'] | sort }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"a\", \"b\", \"c\"]");

        // Test sort in reverse
        let template = env
            .template_from_str("{{ [3, 1, 2] | sort(true) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[3, 2, 1]");

        // Test sort case insensitive
        let template = env
            .template_from_str("{{ ['B', 'a', 'C'] | sort(false, false) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"a\", \"B\", \"C\"]");

        // Test sort with empty list
        let template = env.template_from_str("{{ [] | sort }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test sort with string (sorts characters)
        let template = env.template_from_str("{{ 'cba' | sort }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_unique_filter() {
        let env = create_test_env();

        // Test unique with list
        let template = env
            .template_from_str("{{ [1, 2, 2, 3, 1] | unique }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2, 3]");

        // Test unique with strings
        let template = env
            .template_from_str("{{ ['a', 'b', 'a', 'c'] | unique }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[\"a\", \"b\", \"c\"]");

        // Test unique with empty list
        let template = env.template_from_str("{{ [] | unique }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test unique with no duplicates
        let template = env.template_from_str("{{ [1, 2, 3] | unique }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[1, 2, 3]");

        // Test unique with string (returns as-is)
        let template = env.template_from_str("{{ 'hello' | unique }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_batch_filter() {
        let env = create_test_env();

        // Test batch with even division
        let template = env
            .template_from_str("{{ [1, 2, 3, 4] | batch(2) }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 2], [3, 4]]");

        // Test batch with uneven division
        let template = env.template_from_str("{{ [1, 2, 3] | batch(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 2], [3]]");

        // Test batch with fill
        let template = env
            .template_from_str("{{ [1, 2, 3] | batch(2, 'x') }}")
            .unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1, 2], [3, \"x\"]]");

        // Test batch with empty list
        let template = env.template_from_str("{{ [] | batch(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");

        // Test batch with size 1
        let template = env.template_from_str("{{ [1, 2, 3] | batch(1) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[[1], [2], [3]]");

        // Test batch with string (returns empty list)
        let template = env.template_from_str("{{ 'hello' | batch(2) }}").unwrap();
        let result = template.render(minijinja::context!()).unwrap();
        assert_eq!(result, "[]");
    }
}
