//! Unit tests for the plugin system
//!
//! These tests cover plugin loading, validation, security boundaries,
//! and error handling.

use std::fs;
use tempfile::TempDir;
use wasmtime::Module;

use driftless::config::PluginSecurityConfig;
use driftless::plugins::{parse_plugin_component_name, PluginManager, PluginRegistry};

/// Test parsing of plugin component names
#[test]
fn test_parse_plugin_component_name() {
    // Valid component names
    assert_eq!(
        parse_plugin_component_name("my_plugin.my_task").unwrap(),
        ("my_plugin", "my_task")
    );
    assert_eq!(
        parse_plugin_component_name("test.facts_collector").unwrap(),
        ("test", "facts_collector")
    );

    // Invalid component names
    assert!(parse_plugin_component_name("single_part").is_err());
    assert!(parse_plugin_component_name("too.many.parts.here").is_err());
    assert!(parse_plugin_component_name("").is_err());
}

/// Test plugin registry basic functionality
#[test]
fn test_plugin_registry_basic() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let mut registry = PluginRegistry::new(plugin_dir.clone());

    // Initially not scanned
    assert!(!registry.is_scanned());
    assert_eq!(registry.get_discovered_plugins().len(), 0);
    assert_eq!(registry.plugin_dir(), plugin_dir);

    // Scan empty directory
    registry.scan_plugins().unwrap();
    assert!(registry.is_scanned());
    assert_eq!(registry.get_discovered_plugins().len(), 0);
}

/// Test plugin registry with mock plugin files
#[test]
fn test_plugin_registry_with_plugins() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create mock plugin files
    let plugin1_path = plugin_dir.join("test_plugin1.wasm");
    let plugin2_path = plugin_dir.join("test_plugin2.wasm");
    let not_plugin_path = plugin_dir.join("not_a_plugin.txt");

    fs::write(&plugin1_path, b"mock wasm content 1").unwrap();
    fs::write(&plugin2_path, b"mock wasm content 2").unwrap();
    fs::write(&not_plugin_path, b"not a plugin").unwrap();

    let mut registry = PluginRegistry::new(plugin_dir);

    // Scan for plugins
    registry.scan_plugins().unwrap();
    assert!(registry.is_scanned());

    let plugins = registry.get_discovered_plugins();
    assert_eq!(plugins.len(), 2);

    // Check plugin info
    let plugin1_info = plugins.get("test_plugin1").unwrap();
    assert_eq!(plugin1_info.name, "test_plugin1");
    assert_eq!(plugin1_info.path, plugin1_path);
    assert!(!plugin1_info.loaded);
    assert!(plugin1_info.load_error.is_none());

    let plugin2_info = plugins.get("test_plugin2").unwrap();
    assert_eq!(plugin2_info.name, "test_plugin2");
    assert_eq!(plugin2_info.path, plugin2_path);

    // Test getting plugin names
    let names = registry.get_discovered_plugin_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"test_plugin1".to_string()));
    assert!(names.contains(&"test_plugin2".to_string()));
}

/// Test plugin registry update methods
#[test]
fn test_plugin_registry_updates() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let plugin_path = plugin_dir.join("test_plugin.wasm");
    fs::write(&plugin_path, b"mock wasm").unwrap();

    let mut registry = PluginRegistry::new(plugin_dir);
    registry.scan_plugins().unwrap();

    // Update plugin info after successful loading
    registry.update_plugin_info(
        "test_plugin",
        Some("1.0.0".to_string()),
        Some("Test plugin".to_string()),
    );

    let plugin_info = registry.get_plugin_info("test_plugin").unwrap();
    assert!(plugin_info.loaded);
    assert!(plugin_info.load_error.is_none());
    assert_eq!(plugin_info.version.as_deref(), Some("1.0.0"));
    assert_eq!(plugin_info.description.as_deref(), Some("Test plugin"));

    // Update plugin load error
    registry.update_plugin_load_error("test_plugin", "Load failed".to_string());

    let plugin_info = registry.get_plugin_info("test_plugin").unwrap();
    assert!(!plugin_info.loaded);
    assert_eq!(plugin_info.load_error.as_deref(), Some("Load failed"));
}

/// Test plugin manager creation with default security config
#[test]
fn test_plugin_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let _manager = PluginManager::new(plugin_dir).unwrap();

    // Manager should be created successfully
}

/// Test plugin manager creation with custom security config
#[test]
fn test_plugin_manager_custom_security() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let security_config = PluginSecurityConfig {
        max_memory: 128 * 1024 * 1024, // 128MB
        fuel_limit: 500_000_000,       // 500M instructions
        ..Default::default()
    };

    let _manager = PluginManager::new_with_security_config(plugin_dir, security_config).unwrap();

    // Manager should be created successfully with custom config
}

/// Test plugin validation with non-existent plugin
#[test]
fn test_plugin_validation_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let mut manager = PluginManager::new(plugin_dir).unwrap();
    manager.scan_plugins().unwrap();

    let result = manager.validate_plugin("nonexistent");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not found in registry"));
}

/// Test plugin validation with invalid WASM file
#[test]
fn test_plugin_validation_invalid_wasm() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create invalid "WASM" file
    let plugin_path = plugin_dir.join("invalid_plugin.wasm");
    fs::write(&plugin_path, b"this is not wasm").unwrap();

    let mut manager = PluginManager::new(plugin_dir).unwrap();
    manager.scan_plugins().unwrap();

    let result = manager.validate_plugin("invalid_plugin");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid WASM file"));
}

/// Test plugin validation with oversized file
#[test]
fn test_plugin_validation_oversized() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create oversized file (60MB)
    let plugin_path = plugin_dir.join("oversized_plugin.wasm");
    let large_content = vec![0u8; 60 * 1024 * 1024];
    fs::write(&plugin_path, large_content).unwrap();

    let mut manager = PluginManager::new(plugin_dir).unwrap();
    manager.scan_plugins().unwrap();

    let result = manager.validate_plugin("oversized_plugin");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));
}

/// Test security validation of WASM module imports
#[test]
fn test_security_validation_imports() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let manager = PluginManager::new(plugin_dir).unwrap();

    // Create a minimal valid WASM module for testing
    // This is a very basic WASM module that just exports a function
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "test") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .unwrap();

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    // This should pass basic validation
    let result = manager.test_validate_wasm_module(&module, "test_plugin");
    assert!(result.is_ok());
}

/// Test security validation with dangerous imports (if we had a module with them)
/// This test would require creating a WASM module with forbidden imports
#[test]
fn test_security_validation_dangerous_imports() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let mut manager = PluginManager::new(plugin_dir).unwrap();

    // Disable WASI allowance to test blocking
    manager.test_get_security_config().allow_wasi = false;

    // Create a WASM module that imports WASI functions
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write" (func (param i32 i32 i32 i32) (result i32)))
            (func (export "test") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .unwrap();

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    // This should fail security validation
    let result = manager.test_validate_wasm_module(&module, "dangerous_plugin");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("forbidden WASI function"));
}

/// Test plugin loading with valid plugin
#[test]
fn test_plugin_loading_valid() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create a valid WASM plugin that exports required interface functions
    let plugin_path = plugin_dir.join("valid_plugin.wasm");
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "get_task_definitions") (result i32)
                i32.const 0  ;; Return empty array
            )
            (func (export "get_facts_collectors") (result i32)
                i32.const 0
            )
            (func (export "get_template_extensions") (result i32)
                i32.const 0
            )
            (func (export "get_log_sources") (result i32)
                i32.const 0
            )
            (func (export "get_log_parsers") (result i32)
                i32.const 0
            )
            (func (export "get_log_filters") (result i32)
                i32.const 0
            )
            (func (export "get_log_outputs") (result i32)
                i32.const 0
            )
        )
    "#,
    )
    .unwrap();

    fs::write(&plugin_path, wasm_bytes).unwrap();

    let mut manager = PluginManager::new(plugin_dir).unwrap();
    manager.scan_plugins().unwrap();

    // This should succeed - the plugin exports required interface functions
    let result = manager.load_plugin("valid_plugin");
    assert!(result.is_ok(), "Valid plugin should load successfully");

    // Check that the plugin was marked as loaded
    let registry = manager.get_registry();
    let plugin_info = registry.get_plugin_info("valid_plugin").unwrap();
    assert!(plugin_info.loaded);
    assert!(plugin_info.load_error.is_none());
}

/// Test plugin loading with invalid plugin
#[test]
fn test_plugin_loading_invalid() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create invalid plugin file
    let plugin_path = plugin_dir.join("invalid_plugin.wasm");
    fs::write(&plugin_path, b"not wasm").unwrap();

    let mut manager = PluginManager::new(plugin_dir).unwrap();
    manager.scan_plugins().unwrap();

    let result = manager.load_plugin("invalid_plugin");
    assert!(result.is_err());
}

/// Test loading all plugins
#[test]
fn test_load_all_plugins() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create multiple mock plugins
    let plugin1_path = plugin_dir.join("plugin1.wasm");
    let plugin2_path = plugin_dir.join("plugin2.wasm");

    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "get_task_definitions") (result i32) i32.const 0)
            (func (export "get_facts_collectors") (result i32) i32.const 0)
            (func (export "get_template_extensions") (result i32) i32.const 0)
            (func (export "get_log_sources") (result i32) i32.const 0)
            (func (export "get_log_parsers") (result i32) i32.const 0)
            (func (export "get_log_filters") (result i32) i32.const 0)
            (func (export "get_log_outputs") (result i32) i32.const 0)
        )
    "#,
    )
    .unwrap();

    fs::write(&plugin1_path, &wasm_bytes).unwrap();
    fs::write(&plugin2_path, &wasm_bytes).unwrap();

    let mut manager = PluginManager::new(plugin_dir).unwrap();
    manager.scan_plugins().unwrap();

    // Load all plugins (should succeed)
    let result = manager.load_all_plugins();
    assert!(result.is_ok());

    // Check that plugins were loaded successfully
    let registry = manager.get_registry();
    let plugin1_info = registry.get_plugin_info("plugin1").unwrap();
    let plugin2_info = registry.get_plugin_info("plugin2").unwrap();

    // Both should be loaded successfully
    assert!(plugin1_info.loaded);
    assert!(plugin2_info.loaded);
    assert!(plugin1_info.load_error.is_none());
    assert!(plugin2_info.load_error.is_none());
}

/// Test security configuration defaults
#[test]
fn test_security_config_defaults() {
    let config = PluginSecurityConfig::default();

    assert!(config.max_memory > 0);
    assert!(config.fuel_limit > 0);
    assert!(config.max_stack_size > 0);
    assert!(config.max_tables > 0);
    assert!(config.max_memories > 0);
    assert!(!config.allow_wasi); // WASI should be disabled by default
    assert!(!config.debug_enabled); // Debug should be disabled by default
}

/// Test that plugin directory creation works
#[test]
fn test_plugin_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().join("nonexistent_plugins");

    // Directory doesn't exist yet
    assert!(!plugin_dir.exists());

    let mut registry = PluginRegistry::new(plugin_dir.clone());

    // Scanning should create the directory
    registry.scan_plugins().unwrap();

    assert!(plugin_dir.exists());
    assert!(plugin_dir.is_dir());
}
