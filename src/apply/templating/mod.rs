//! Shared templating utilities for minijinja setup and rendering

pub mod encoding_filters;
pub mod list_filters;
pub mod math_filters;
pub mod path_filters;
pub mod path_operations;
pub mod string_filters;
pub mod tests;
pub mod utility_functions;

use minijinja::{Environment, Value as JinjaValue};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Type alias for template filter functions
type TemplateFilterFn = Arc<dyn Fn(JinjaValue, &[JinjaValue]) -> JinjaValue + Send + Sync>;

// Type alias for template function functions
type TemplateFunctionFn = Arc<dyn Fn(&[JinjaValue]) -> JinjaValue + Send + Sync>;

// Template filter registry entry
#[derive(Clone)]
pub struct TemplateFilterEntry {
    #[allow(unused)]
    pub name: String,
    pub description: String,
    pub category: String,
    pub arguments: Vec<(String, String)>,
    pub filter_fn: TemplateFilterFn,
}

// Template function registry entry
#[derive(Clone)]
pub struct TemplateFunctionEntry {
    #[allow(unused)]
    pub name: String,
    pub description: String,
    pub category: String,
    pub arguments: Vec<(String, String)>,
    pub function_fn: TemplateFunctionFn,
}

// Global template filter registry
static TEMPLATE_FILTER_REGISTRY: Lazy<RwLock<HashMap<String, TemplateFilterEntry>>> =
    Lazy::new(|| {
        let mut registry = HashMap::new();
        TemplateRegistry::initialize_builtin_filters(&mut registry);
        RwLock::new(registry)
    });

// Global template function registry
static TEMPLATE_FUNCTION_REGISTRY: Lazy<RwLock<HashMap<String, TemplateFunctionEntry>>> =
    Lazy::new(|| {
        let mut registry = HashMap::new();
        TemplateRegistry::initialize_builtin_functions(&mut registry);
        RwLock::new(registry)
    });

/// Template registry for runtime extensibility
pub struct TemplateRegistry;

impl TemplateRegistry {
    /// Register a template filter
    pub fn register_filter(
        registry: &mut HashMap<String, TemplateFilterEntry>,
        name: &str,
        description: &str,
        category: &str,
        arguments: Vec<(String, String)>,
        filter_fn: TemplateFilterFn,
    ) {
        let entry = TemplateFilterEntry {
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            arguments,
            filter_fn,
        };
        registry.insert(name.to_string(), entry);
    }

    /// Register a template function
    pub fn register_function(
        registry: &mut HashMap<String, TemplateFunctionEntry>,
        name: &str,
        description: &str,
        category: &str,
        arguments: Vec<(String, String)>,
        function_fn: TemplateFunctionFn,
    ) {
        let entry = TemplateFunctionEntry {
            name: name.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            arguments,
            function_fn,
        };
        registry.insert(name.to_string(), entry);
    }

    /// Initialize the registry with built-in filters
    pub fn initialize_builtin_filters(registry: &mut HashMap<String, TemplateFilterEntry>) {
        string_filters::register_string_filters(registry);
        list_filters::register_list_filters(registry);
        encoding_filters::register_encoding_filters(registry);
        math_filters::register_math_filters(registry);
        path_operations::register_path_filters(registry);
        path_filters::register_path_filters(registry);
    }

    /// Initialize the registry with built-in functions
    pub fn initialize_builtin_functions(registry: &mut HashMap<String, TemplateFunctionEntry>) {
        utility_functions::register_utility_functions(registry);
        path_operations::register_path_functions(registry);
    }

    /// Get all registered filter names
    pub fn get_registered_filters() -> Vec<String> {
        let registry = TEMPLATE_FILTER_REGISTRY.read().unwrap();
        registry.keys().cloned().collect()
    }

    /// Get all registered function names
    pub fn get_registered_functions() -> Vec<String> {
        let registry = TEMPLATE_FUNCTION_REGISTRY.read().unwrap();
        registry.keys().cloned().collect()
    }

    /// Get filter description
    pub fn get_filter_description(name: &str) -> Option<String> {
        let registry = TEMPLATE_FILTER_REGISTRY.read().unwrap();
        registry.get(name).map(|e| e.description.clone())
    }

    /// Get function description
    pub fn get_function_description(name: &str) -> Option<String> {
        let registry = TEMPLATE_FUNCTION_REGISTRY.read().unwrap();
        registry.get(name).map(|e| e.description.clone())
    }

    /// Get filter category
    pub fn get_filter_category(name: &str) -> Option<String> {
        let registry = TEMPLATE_FILTER_REGISTRY.read().unwrap();
        registry.get(name).map(|e| e.category.clone())
    }

    /// Get function category
    pub fn get_function_category(name: &str) -> Option<String> {
        let registry = TEMPLATE_FUNCTION_REGISTRY.read().unwrap();
        registry.get(name).map(|e| e.category.clone())
    }

    /// Get filter arguments
    pub fn get_filter_arguments(name: &str) -> Option<Vec<(String, String)>> {
        let registry = TEMPLATE_FILTER_REGISTRY.read().unwrap();
        registry.get(name).map(|e| e.arguments.clone())
    }

    /// Get function arguments
    pub fn get_function_arguments(name: &str) -> Option<Vec<(String, String)>> {
        let registry = TEMPLATE_FUNCTION_REGISTRY.read().unwrap();
        registry.get(name).map(|e| e.arguments.clone())
    }

    /// Register a new filter at runtime
    #[allow(unused)]
    pub fn register_filter_runtime(
        name: &str,
        description: &str,
        category: &str,
        arguments: Vec<(String, String)>,
        filter_fn: TemplateFilterFn,
    ) {
        let mut registry = TEMPLATE_FILTER_REGISTRY.write().unwrap();
        TemplateRegistry::register_filter(
            &mut registry,
            name,
            description,
            category,
            arguments,
            filter_fn,
        );
    }

    /// Register a new function at runtime
    #[allow(unused)]
    pub fn register_function_runtime(
        name: &str,
        description: &str,
        category: &str,
        arguments: Vec<(String, String)>,
        function_fn: TemplateFunctionFn,
    ) {
        let mut registry = TEMPLATE_FUNCTION_REGISTRY.write().unwrap();
        TemplateRegistry::register_function(
            &mut registry,
            name,
            description,
            category,
            arguments,
            function_fn,
        );
    }
}

/// Set up minijinja environment with custom filters and functions
pub fn setup_minijinja_env(env: &mut Environment) {
    // Add registered filters
    {
        let registry = TEMPLATE_FILTER_REGISTRY.read().unwrap();
        for (name, entry) in registry.iter() {
            let filter_fn = entry.filter_fn.clone();
            let name_owned = name.clone();
            env.add_filter(name_owned, move |value: JinjaValue, args: &[JinjaValue]| {
                filter_fn(value, args)
            });
        }
    }

    // Add registered functions
    {
        let registry = TEMPLATE_FUNCTION_REGISTRY.read().unwrap();
        for (name, entry) in registry.iter() {
            let function_fn = entry.function_fn.clone();
            let name_owned = name.clone();
            env.add_function(name_owned, move |args: &[JinjaValue]| function_fn(args));
        }
    }
}

/// Render a template with the given context using minijinja
pub fn render_with_context(
    template: &str,
    context: minijinja::Value,
) -> Result<String, minijinja::Error> {
    let mut env = Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    setup_minijinja_env(&mut env);

    let tmpl = env.template_from_str(template)?;
    tmpl.render(&context)
}
