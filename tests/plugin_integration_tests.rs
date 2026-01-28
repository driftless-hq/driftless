//! Integration tests for the plugin system
//!
//! These tests verify plugin functionality through the command-line interface
//! and test end-to-end plugin operations.

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Test plugin CLI commands with basic functionality
#[test]
fn test_plugin_cli_installed_empty() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Build the binary path
    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test plugins installed command with empty directory
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("installed")
        .output()
        .expect("Failed to run plugins installed");

    assert!(
        output.status.success(),
        "Plugin installed command should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No plugins installed locally"),
        "Should report no plugins installed"
    );
}

/// Test plugin CLI validate command with non-existent plugin
#[test]
fn test_plugin_cli_validate_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test plugins validate command with non-existent plugin
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("validate")
        .arg("nonexistent_plugin")
        .output()
        .expect("Failed to run plugins validate");

    assert!(
        !output.status.success(),
        "Plugin validate should fail for non-existent plugin"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("validation failed") || stderr.contains("not found"),
        "Should report validation failure"
    );
}

/// Test plugin CLI list command (should handle network errors gracefully)
#[test]
fn test_plugin_cli_list_handles_errors() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test plugins list command (should fail gracefully with network error)
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("list")
        .output()
        .expect("Failed to run plugins list");

    // Should fail but not crash
    assert!(
        !output.status.success(),
        "Plugin list should fail when registry is unreachable"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to list plugins"),
        "Should report failure to list plugins"
    );
}

/// Test plugin CLI with invalid plugin file
#[test]
fn test_plugin_cli_validate_invalid_file() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create an invalid plugin file
    let invalid_plugin_path = plugin_dir.join("invalid.wasm");
    fs::write(&invalid_plugin_path, b"this is not a valid wasm file").unwrap();

    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test plugins validate command with invalid file
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("validate")
        .arg("invalid")
        .output()
        .expect("Failed to run plugins validate");

    assert!(
        !output.status.success(),
        "Plugin validate should fail for invalid WASM"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("validation failed") || stderr.contains("Invalid WASM"),
        "Should report validation failure for invalid WASM"
    );
}

/// Test plugin CLI help commands
#[test]
fn test_plugin_cli_help() {
    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test main plugins help
    let output = Command::new(binary_path)
        .arg("plugins")
        .arg("--help")
        .output()
        .expect("Failed to run plugins --help");

    assert!(output.status.success(), "Plugin help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"), "Help should mention list command");
    assert!(
        stdout.contains("installed"),
        "Help should mention installed command"
    );
    assert!(
        stdout.contains("validate"),
        "Help should mention validate command"
    );

    // Test specific command help
    let output = Command::new(binary_path)
        .arg("plugins")
        .arg("validate")
        .arg("--help")
        .output()
        .expect("Failed to run plugins validate --help");

    assert!(
        output.status.success(),
        "Plugin validate help should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Plugin name"),
        "Help should describe plugin name parameter"
    );
}

/// Test plugin CLI with oversized file
#[test]
fn test_plugin_cli_validate_oversized() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create an oversized plugin file (60MB)
    let oversized_plugin_path = plugin_dir.join("oversized.wasm");
    let large_content = vec![0u8; 60 * 1024 * 1024];
    fs::write(&oversized_plugin_path, large_content).unwrap();

    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test plugins validate command with oversized file
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("validate")
        .arg("--file")
        .arg(&oversized_plugin_path)
        .output()
        .expect("Failed to run plugins validate");

    assert!(
        !output.status.success(),
        "Plugin validate should fail for oversized file"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("too large"),
        "Should report file too large error"
    );
}

/// Test plugin directory creation through CLI
#[test]
fn test_plugin_directory_creation_via_cli() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().join("auto_created_plugins");

    // Directory doesn't exist yet
    assert!(!plugin_dir.exists());

    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Running plugins installed should create the directory
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("installed")
        .output()
        .expect("Failed to run plugins installed");

    assert!(
        output.status.success(),
        "Plugin installed command should succeed"
    );

    // Directory should now exist
    assert!(plugin_dir.exists(), "Plugin directory should be created");
    assert!(
        plugin_dir.is_dir(),
        "Plugin directory should be a directory"
    );
}

/// Test plugin CLI with multiple plugins
#[test]
fn test_plugin_cli_multiple_plugins() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    // Create multiple mock plugin files
    let plugin1_path = plugin_dir.join("plugin1.wasm");
    let plugin2_path = plugin_dir.join("plugin2.wasm");
    let plugin3_path = plugin_dir.join("plugin3.wasm");

    // Create minimal valid WASM files
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
    fs::write(&plugin3_path, &wasm_bytes).unwrap();

    let binary_path = env!("CARGO_BIN_EXE_driftless");

    // Test plugins installed command with multiple plugins
    let output = Command::new(binary_path)
        .arg("--plugin-dir")
        .arg(&plugin_dir)
        .arg("plugins")
        .arg("installed")
        .output()
        .expect("Failed to run plugins installed");

    assert!(
        output.status.success(),
        "Plugin installed command should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("plugin1"), "Should list plugin1");
    assert!(stdout.contains("plugin2"), "Should list plugin2");
    assert!(stdout.contains("plugin3"), "Should list plugin3");
}
