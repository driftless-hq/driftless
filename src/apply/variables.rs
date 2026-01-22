//! Variable and fact management system
//!
//! Provides storage and templating for variables used throughout task execution.
//! Supports Jinja2-style templating with filters and built-in functions.

use std::collections::HashMap;
use std::path::Path;
use serde_yaml::Mapping;

/// Variable storage for task execution context
#[derive(Debug, Clone, Default)]
pub struct VariableContext {
    variables: HashMap<String, serde_yaml::Value>,
    /// Built-in facts (system information)
    facts: HashMap<String, serde_yaml::Value>,
}

impl VariableContext {
    /// Create a new empty variable context
    pub fn new() -> Self {
        let mut ctx = Self::default();
        ctx.initialize_builtin_facts();
        ctx
    }

    /// Initialize built-in facts and functions
    fn initialize_builtin_facts(&mut self) {
        // System facts
        self.facts.insert("driftless_version".to_string(), serde_yaml::Value::String(env!("CARGO_PKG_VERSION").to_string()));
        self.facts.insert("driftless_distribution".to_string(), serde_yaml::Value::String("Linux".to_string())); // Placeholder
        self.facts.insert("driftless_os_family".to_string(), serde_yaml::Value::String("Linux".to_string()));
        self.facts.insert("driftless_architecture".to_string(), serde_yaml::Value::String(std::env::consts::ARCH.to_string()));

        // Load environment variables into driftless_env
        self.load_environment_variables();
    }

    /// Load environment variables into driftless_env fact
    fn load_environment_variables(&mut self) {
        let mut env_vars = Mapping::new();
        for (key, value) in std::env::vars() {
            env_vars.insert(serde_yaml::Value::String(key), serde_yaml::Value::String(value));
        }
        self.facts.insert("env".to_string(), serde_yaml::Value::Mapping(env_vars));
    }

    /// Load variables from env file
    ///
    /// Supports .env format: KEY=value
    /// Variables are added to the 'env' fact for template access
    pub fn load_env_file(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(()); // Silently ignore missing env files
        }

        let content = std::fs::read_to_string(path)?;

        // Ensure we have an env mapping
        if !self.facts.contains_key("env") {
            self.facts.insert("env".to_string(), serde_yaml::Value::Mapping(Mapping::new()));
        }

        if let Some(serde_yaml::Value::Mapping(ref mut env_map)) = self.facts.get_mut("env") {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some(eq_pos) = line.find('=') {
                    let key = line[..eq_pos].trim();
                    let value = line[eq_pos + 1..].trim();

                    // Remove surrounding quotes if present
                    let value = if (value.starts_with('"') && value.ends_with('"')) ||
                                (value.starts_with('\'') && value.ends_with('\'')) {
                        &value[1..value.len()-1]
                    } else {
                        value
                    };

                    env_map.insert(
                        serde_yaml::Value::String(key.to_string()),
                        serde_yaml::Value::String(value.to_string())
                    );
                }
            }
        }

        Ok(())
    }

    /// Set a variable value
    pub fn set(&mut self, key: String, value: serde_yaml::Value) {
        self.variables.insert(key, value);
    }

    /// Get a variable value
    pub fn get(&self, key: &str) -> Option<&serde_yaml::Value> {
        self.variables.get(key)
    }

    /// Check if a variable exists
    pub fn contains(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }


    /// Render a template string with variable substitution
    ///
    /// Supports Jinja2-style templating with filters and expressions
    pub fn render_template(&self, template: &str) -> String {
        let mut result = template.to_string();

        // Process {{ expressions }}
        result = self.process_expressions(&result);

        result
    }

    /// Process {{ expressions }} in template
    fn process_expressions(&self, template: &str) -> String {
        let mut result = template.to_string();
        let mut start = 0;

        while let Some(expr_start) = result[start..].find("{{") {
            let expr_start = start + expr_start;
            if let Some(expr_end) = result[expr_start + 2..].find("}}") {
                let expr_end = expr_start + 2 + expr_end + 2;
                let expression = result[expr_start + 2..expr_end - 2].trim();

                if let Some(replacement) = self.evaluate_expression(expression) {
                    result.replace_range(expr_start..expr_end, &replacement);
                    // Reset search position to handle nested expressions
                    start = expr_start;
                } else {
                    start = expr_end;
                }
            } else {
                break;
            }
        }

        result
    }

    /// Evaluate a template expression
    fn evaluate_expression(&self, expression: &str) -> Option<String> {
        // Handle filters: value | filter
        if let Some(pipe_pos) = expression.find('|') {
            let value_part = expression[..pipe_pos].trim();
            let filter_part = expression[pipe_pos + 1..].trim();

            if let Some(value) = self.evaluate_simple_expression(value_part) {
                return self.apply_filter(&value, filter_part);
            }
            return None;
        }

        // Handle function calls: function(arg)
        if let Some(open_paren) = expression.find('(') {
            if let Some(close_paren) = expression.rfind(')') {
                let func_name = expression[..open_paren].trim();
                let args_str = expression[open_paren + 1..close_paren].trim();
                return self.call_function(func_name, args_str);
            }
        }

        // Handle simple variable access
        self.evaluate_simple_expression(expression)
    }

    /// Evaluate simple expressions (variables, literals)
    fn evaluate_simple_expression(&self, expr: &str) -> Option<String> {
        let trimmed = expr.trim();

        // Handle string literals
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return Some(trimmed[1..trimmed.len()-1].to_string());
        }
        if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
            return Some(trimmed[1..trimmed.len()-1].to_string());
        }

        // Handle numeric literals
        if trimmed.parse::<f64>().is_ok() {
            return Some(trimmed.to_string());
        }

        // Handle boolean literals
        match trimmed.to_lowercase().as_str() {
            "true" => return Some("true".to_string()),
            "false" => return Some("false".to_string()),
            _ => {}
        }

        // Handle dot notation for nested access (e.g., env.USER)
        if let Some(dot_pos) = trimmed.find('.') {
            let base = &trimmed[..dot_pos];
            let key = &trimmed[dot_pos + 1..];

            // Check if base is a fact with nested structure
            if let Some(serde_yaml::Value::Mapping(map)) = self.facts.get(base) {
                if let Some(value) = map.get(serde_yaml::Value::String(key.to_string())) {
                    match value {
                        serde_yaml::Value::String(s) => return Some(s.clone()),
                        serde_yaml::Value::Number(n) => return Some(n.to_string()),
                        serde_yaml::Value::Bool(b) => return Some(b.to_string()),
                        _ => return Some(format!("{:?}", value)),
                    }
                }
            }
        }

        // Handle variable access
        if let Some(value) = self.get(trimmed) {
            match value {
                serde_yaml::Value::String(s) => Some(s.clone()),
                serde_yaml::Value::Number(n) => Some(n.to_string()),
                serde_yaml::Value::Bool(b) => Some(b.to_string()),
                serde_yaml::Value::Sequence(seq) => Some(format!("{:?}", seq)),
                serde_yaml::Value::Mapping(map) => Some(format!("{:?}", map)),
                _ => Some(format!("{:?}", value)),
            }
        } else if let Some(fact) = self.facts.get(trimmed) {
            match fact {
                serde_yaml::Value::String(s) => Some(s.clone()),
                serde_yaml::Value::Number(n) => Some(n.to_string()),
                serde_yaml::Value::Bool(b) => Some(b.to_string()),
                _ => Some(format!("{:?}", fact)),
            }
        } else {
            None
        }
    }

    /// Apply a Jinja2-style filter
    fn apply_filter(&self, value: &str, filter: &str) -> Option<String> {
        match filter {
            "length" | "len" => Some(value.len().to_string()),
            "upper" => Some(value.to_uppercase()),
            "lower" => Some(value.to_lowercase()),
            "basename" => Some(Path::new(value).file_name()?.to_str()?.to_string()),
            "dirname" => Some(Path::new(value).parent()?.to_str()?.to_string()),
            "abs" => value.parse::<f64>().ok().map(|n| n.abs().to_string()),
            "int" => value.parse::<f64>().ok().map(|n| n.trunc().to_string()),
            _ => Some(value.to_string()), // Unknown filter, return original value
        }
    }

    /// Call a built-in function
    fn call_function(&self, name: &str, args: &str) -> Option<String> {
        match name {
            "length" | "len" => self
                .evaluate_simple_expression(args)
                .map(|value| value.len().to_string()),
            "basename" => {
                if let Some(value) = self.evaluate_simple_expression(args) {
                    Some(Path::new(&value).file_name()?.to_str()?.to_string())
                } else {
                    None
                }
            }
            "dirname" => {
                if let Some(value) = self.evaluate_simple_expression(args) {
                    Some(Path::new(&value).parent()?.to_str()?.to_string())
                } else {
                    None
                }
            }
            "abs" => {
                if let Some(value) = self.evaluate_simple_expression(args) {
                    value.parse::<f64>().ok().map(|n| n.abs().to_string())
                } else {
                    None
                }
            }
            "lookup" => {
                self.call_lookup_function(args)
            }
            _ => None,
        }
    }

    /// Call lookup function (Driftless-style)
    fn call_lookup_function(&self, args: &str) -> Option<String> {
        // Parse lookup('type', 'arg1', 'arg2', ...)
        let args = args.trim();

        // Handle the format: "'env', 'VAR_NAME'"
        if let Some(var_start) = args.find("'env', '") {
            if let Some(var_end) = args[var_start + 8..].find("'") {
                let var_name = &args[var_start + 8..var_start + 8 + var_end];
                return std::env::var(var_name).ok();
            }
        }

        None
    }

    /// Evaluate a boolean expression
    ///
    /// Supports complex Driftless expressions with variables, comparisons, and logical operators
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        let trimmed = condition.trim();

        // Handle simple boolean literals
        match trimmed.to_lowercase().as_str() {
            "true" | "yes" | "1" => return true,
            "false" | "no" | "0" => return false,
            _ => {}
        }

        // Render template expressions first
        let rendered = self.render_template(trimmed);

        // Parse the rendered expression
        self.evaluate_boolean_expression(&rendered)
    }

    /// Evaluate a boolean expression after template rendering
    fn evaluate_boolean_expression(&self, expr: &str) -> bool {
        let expr = expr.trim();

        // Handle logical NOT
        if expr.starts_with("not ") || expr.starts_with("!") {
            let rest = expr
                .strip_prefix("not ")
                .or_else(|| expr.strip_prefix('!'))
                .unwrap_or(expr);
            return !self.evaluate_boolean_expression(rest);
        }

        // Handle logical AND
        if let Some(and_pos) = expr.find(" and ") {
            let left = &expr[..and_pos];
            let right = &expr[and_pos + 5..];
            return self.evaluate_boolean_expression(left) && self.evaluate_boolean_expression(right);
        }

        // Handle logical OR
        if let Some(or_pos) = expr.find(" or ") {
            let left = &expr[..or_pos];
            let right = &expr[or_pos + 4..];
            return self.evaluate_boolean_expression(left) || self.evaluate_boolean_expression(right);
        }

        // Handle comparisons
        if let Some(op_pos) = expr.find(" == ") {
            let left = expr[..op_pos].trim();
            let right = expr[op_pos + 4..].trim();
            return self.compare_values(left, right, "==");
        }
        if let Some(op_pos) = expr.find(" != ") {
            let left = expr[..op_pos].trim();
            let right = expr[op_pos + 4..].trim();
            return self.compare_values(left, right, "!=");
        }
        if let Some(op_pos) = expr.find(" < ") {
            let left = expr[..op_pos].trim();
            let right = expr[op_pos + 3..].trim();
            return self.compare_values(left, right, "<");
        }
        if let Some(op_pos) = expr.find(" > ") {
            let left = expr[..op_pos].trim();
            let right = expr[op_pos + 3..].trim();
            return self.compare_values(left, right, ">");
        }
        if let Some(op_pos) = expr.find(" <= ") {
            let left = expr[..op_pos].trim();
            let right = expr[op_pos + 4..].trim();
            return self.compare_values(left, right, "<=");
        }
        if let Some(op_pos) = expr.find(" >= ") {
            let left = expr[..op_pos].trim();
            let right = expr[op_pos + 4..].trim();
            return self.compare_values(left, right, ">=");
        }

        // Handle "is defined" checks
        if let Some(var_name) = expr.strip_suffix(" is defined") {
            return self.contains(var_name.trim());
        }
        if let Some(var_name) = expr.strip_suffix(" is not defined") {
            return !self.contains(var_name.trim());
        }

        // Handle "in" operator
        if let Some(in_pos) = expr.find(" in ") {
            let item = expr[..in_pos].trim();
            let container = expr[in_pos + 4..].trim();
            return self.check_membership(item, container);
        }

        // Try to evaluate as a simple value
        match expr.to_lowercase().as_str() {
            "true" | "yes" => true,
            "false" | "no" => false,
            _ => {
                // Check if it's a variable that evaluates to a boolean
                if let Some(value) = self.evaluate_simple_expression(expr) {
                    match value.to_lowercase().as_str() {
                        "true" | "yes" | "1" => true,
                        "false" | "no" | "0" => false,
                        _ => false,
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Compare two values
    fn compare_values(&self, left: &str, right: &str, op: &str) -> bool {
        // Try numeric comparison first
        if let (Ok(left_num), Ok(right_num)) = (left.parse::<f64>(), right.parse::<f64>()) {
            return match op {
                "==" => left_num == right_num,
                "!=" => left_num != right_num,
                "<" => left_num < right_num,
                ">" => left_num > right_num,
                "<=" => left_num <= right_num,
                ">=" => left_num >= right_num,
                _ => false,
            };
        }

        // String comparison
        match op {
            "==" => left == right,
            "!=" => left != right,
            "<" => left < right,
            ">" => left > right,
            "<=" => left <= right,
            ">=" => left >= right,
            _ => false,
        }
    }

    /// Check if item is in container
    fn check_membership(&self, item: &str, container_expr: &str) -> bool {
        let container = container_expr.trim();

        // Handle YAML sequence syntax like ["a", "b"]
        if container.starts_with('[') && container.ends_with(']') {
            let items_str = &container[1..container.len()-1];
            let items: Vec<&str> = items_str.split(',')
                .map(|s| s.trim().trim_matches('"').trim_matches('\''))
                .collect();
            return items.contains(&item);
        }

        // Check if container is a variable that holds a sequence
        if let Some(serde_yaml::Value::Sequence(seq)) = self.get(container) {
            return seq.iter().any(|v| match v {
                serde_yaml::Value::String(s) => s == item,
                serde_yaml::Value::Number(n) => n.to_string() == item,
                _ => false,
            });
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_context() {
        let mut ctx = VariableContext::new();

        // Test setting and getting variables
        ctx.set("name".to_string(), serde_yaml::Value::String("alice".to_string()));
        ctx.set("age".to_string(), serde_yaml::Value::Number(30.into()));

        assert_eq!(ctx.get("name"), Some(&serde_yaml::Value::String("alice".to_string())));
        assert_eq!(ctx.get("age"), Some(&serde_yaml::Value::Number(30.into())));
        assert_eq!(ctx.get("missing"), None);
    }

    #[test]
    fn test_template_rendering() {
        let mut ctx = VariableContext::new();
        ctx.set("user".to_string(), serde_yaml::Value::String("bob".to_string()));
        ctx.set("count".to_string(), serde_yaml::Value::Number(42.into()));
        ctx.set("path".to_string(), serde_yaml::Value::String("/home/user/file.txt".to_string()));

        // Basic variable substitution
        assert_eq!(ctx.render_template("Hello {{ user }}!"), "Hello bob!");
        assert_eq!(ctx.render_template("Count: {{ count }}"), "Count: 42");

        // Filters
        assert_eq!(ctx.render_template("{{ user | upper }}"), "BOB");
        assert_eq!(ctx.render_template("{{ path | basename }}"), "file.txt");
        assert_eq!(ctx.render_template("{{ path | dirname }}"), "/home/user");

        // Functions
        assert_eq!(ctx.render_template("{{ length(user) }}"), "3");
        assert_eq!(ctx.render_template("{{ basename(path) }}"), "file.txt");

        // No substitutions
        assert_eq!(ctx.render_template("No vars here"), "No vars here");
    }

    #[test]
    fn test_condition_evaluation() {
        let mut ctx = VariableContext::new();
        ctx.set("status".to_string(), serde_yaml::Value::String("ready".to_string()));
        ctx.set("enabled".to_string(), serde_yaml::Value::Bool(true));
        ctx.set("count".to_string(), serde_yaml::Value::Number(42.into()));
        ctx.set("items".to_string(), serde_yaml::Value::Sequence(vec![
            serde_yaml::Value::String("a".to_string()),
            serde_yaml::Value::String("b".to_string()),
        ]));

        // Basic boolean literals
        assert!(ctx.evaluate_condition("true"));
        assert!(!ctx.evaluate_condition("false"));

        // Variable comparisons
        assert!(ctx.evaluate_condition("{{ enabled }} == true"));
        assert!(!ctx.evaluate_condition("{{ status }} == pending"));
        assert!(ctx.evaluate_condition("{{ count }} == 42"));
        assert!(ctx.evaluate_condition("{{ count }} > 40"));

        // Logical operators
        assert!(ctx.evaluate_condition("{{ enabled }} and {{ count }} > 40"));
        assert!(ctx.evaluate_condition("{{ status }} == ready or {{ count }} < 10"));
        assert!(!ctx.evaluate_condition("not {{ enabled }}"));

        // Membership tests - check if variable exists
        assert!(ctx.evaluate_condition("items is defined"));
        assert!(!ctx.evaluate_condition("nonexistent is defined"));

        // Defined checks
        assert!(ctx.evaluate_condition("status is defined"));
        assert!(!ctx.evaluate_condition("missing_var is defined"));
        assert!(ctx.evaluate_condition("missing_var is not defined"));
    }
}