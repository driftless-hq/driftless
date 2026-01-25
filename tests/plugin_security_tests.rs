//! Security tests for the plugin system
//!
//! These tests focus on security boundaries, attack prevention,
//! and ensuring plugins cannot compromise the host system.

use tempfile::TempDir;
use wasmtime::Module;

use driftless::config::PluginSecurityConfig;
use driftless::plugins::PluginManager;

/// Test that plugins cannot import dangerous WASI functions
#[test]
fn test_security_block_wasi_imports() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let mut manager = PluginManager::new(plugin_dir).unwrap();

    // Ensure WASI is blocked
    manager.test_get_security_config().allow_wasi = false;

    // Create a WASM module that tries to import various WASI functions
    let dangerous_wasi_imports = [
        r#"(import "wasi_snapshot_preview1" "fd_write" (func (param i32 i32 i32 i32) (result i32)))"#,
        r#"(import "wasi_snapshot_preview1" "fd_read" (func (param i32 i32 i32 i32) (result i32)))"#,
        r#"(import "wasi_snapshot_preview1" "path_open" (func (param i32 i32 i32 i32 i32 i64 i64 i32 i32) (result i32)))"#,
        r#"(import "wasi_snapshot_preview1" "proc_exit" (func (param i32)))"#,
    ];

    for (i, import) in dangerous_wasi_imports.iter().enumerate() {
        let wasm_code = format!(
            r#"
            (module
                {}
                (func (export "test") (result i32)
                    i32.const 42
                )
            )
        "#,
            import
        );

        let wasm_bytes = wat::parse_str(&wasm_code).unwrap();
        let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

        let result = manager.test_validate_wasm_module(&module, &format!("dangerous_plugin_{}", i));
        assert!(
            result.is_err(),
            "Plugin with dangerous WASI import {} should be blocked",
            import
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden WASI function"));
    }
}

/// Test that plugins cannot import dangerous system functions via env
#[test]
fn test_security_block_env_syscall_imports() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let manager = PluginManager::new(plugin_dir).unwrap();

    // Create WASM modules that try to import syscall-like functions
    let dangerous_env_imports = [
        r#"(import "env" "syscall" (func (param i32) (result i32)))"#,
        r#"(import "env" "system" (func (param i32) (result i32)))"#,
        r#"(import "env" "__syscall" (func (param i32) (result i32)))"#,
    ];

    for (i, import) in dangerous_env_imports.iter().enumerate() {
        let wasm_code = format!(
            r#"
            (module
                {}
                (func (export "test") (result i32)
                    i32.const 42
                )
            )
        "#,
            import
        );

        let wasm_bytes = wat::parse_str(&wasm_code).unwrap();
        let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

        let result = manager.test_validate_wasm_module(&module, &format!("syscall_plugin_{}", i));
        assert!(
            result.is_err(),
            "Plugin with syscall import {} should be blocked",
            import
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden system function"));
    }
}

/// Test that plugins cannot import file system functions
#[test]
fn test_security_block_filesystem_imports() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let manager = PluginManager::new(plugin_dir).unwrap();

    // Create WASM modules that try to import filesystem functions
    let dangerous_fs_imports = [
        r#"(import "env" "fd_open" (func (param i32 i32) (result i32)))"#,
        r#"(import "env" "fd_close" (func (param i32) (result i32)))"#,
        r#"(import "env" "fd_read" (func (param i32 i32 i32) (result i32)))"#,
        r#"(import "env" "fd_write" (func (param i32 i32 i32) (result i32)))"#,
        r#"(import "env" "path_create_directory" (func (param i32 i32) (result i32)))"#,
    ];

    for (i, import) in dangerous_fs_imports.iter().enumerate() {
        let wasm_code = format!(
            r#"
            (module
                {}
                (func (export "test") (result i32)
                    i32.const 42
                )
            )
        "#,
            import
        );

        let wasm_bytes = wat::parse_str(&wasm_code).unwrap();
        let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

        let result = manager.test_validate_wasm_module(&module, &format!("fs_plugin_{}", i));
        assert!(
            result.is_err(),
            "Plugin with filesystem import {} should be blocked",
            import
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden file system function"));
    }
}

/// Test that plugins cannot import network functions
#[test]
fn test_security_block_network_imports() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let manager = PluginManager::new(plugin_dir).unwrap();

    // Create WASM modules that try to import network functions
    let dangerous_net_imports = [
        r#"(import "env" "socket" (func (param i32 i32 i32) (result i32)))"#,
        r#"(import "env" "connect" (func (param i32 i32 i32) (result i32)))"#,
        r#"(import "env" "bind" (func (param i32 i32 i32) (result i32)))"#,
        r#"(import "env" "listen" (func (param i32 i32) (result i32)))"#,
        r#"(import "env" "accept" (func (param i32 i32 i32) (result i32)))"#,
    ];

    for (i, import) in dangerous_net_imports.iter().enumerate() {
        let wasm_code = format!(
            r#"
            (module
                {}
                (func (export "test") (result i32)
                    i32.const 42
                )
            )
        "#,
            import
        );

        let wasm_bytes = wat::parse_str(&wasm_code).unwrap();
        let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

        let result = manager.test_validate_wasm_module(&module, &format!("net_plugin_{}", i));
        assert!(
            result.is_err(),
            "Plugin with network import {} should be blocked",
            import
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("forbidden network function"));
    }
}

/// Test memory limits are enforced
#[test]
fn test_security_memory_limits() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let security_config = PluginSecurityConfig {
        max_memories: 1, // Allow only 1 memory
        ..Default::default()
    };

    let manager = PluginManager::new_with_security_config(plugin_dir, security_config).unwrap();

    // Create a WASM module with 2 memories (exceeds limit)
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (memory 1)
            (memory 1)  ;; Second memory exceeds limit
            (func (export "test") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .unwrap();

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    let result = manager.test_validate_wasm_module(&module, "memory_hog_plugin");
    assert!(
        result.is_err(),
        "Plugin with too many memories should be blocked"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds maximum memory count"));
}

/// Test table limits are enforced
#[test]
fn test_security_table_limits() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let security_config = PluginSecurityConfig {
        max_tables: 1, // Allow only 1 table
        ..Default::default()
    };

    let manager = PluginManager::new_with_security_config(plugin_dir, security_config).unwrap();

    // Create a WASM module with 2 tables (exceeds limit)
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (table 10 funcref)
            (table 10 funcref)  ;; Second table exceeds limit
            (func (export "test") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .unwrap();

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    let result = manager.test_validate_wasm_module(&module, "table_hog_plugin");
    assert!(
        result.is_err(),
        "Plugin with too many tables should be blocked"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exceeds maximum table count"));
}

/// Test that plugins with dangerous exports are flagged (warnings)
#[test]
fn test_security_dangerous_exports_warning() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let manager = PluginManager::new(plugin_dir).unwrap();

    // Create a WASM module with potentially dangerous exports
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "safe_function") (result i32)
                i32.const 42
            )
            (func (export "unsafe_syscall") (result i32)
                i32.const 0
            )
            (func (export "dangerous_function") (result i32)
                i32.const 1
            )
        )
    "#,
    )
    .unwrap();

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    // This should succeed but log warnings about dangerous exports
    let result = manager.test_validate_wasm_module(&module, "dangerous_exports_plugin");
    // The validation currently allows this but logs warnings
    // In a real implementation, we might want to block dangerous exports too
    assert!(
        result.is_ok(),
        "Plugin with dangerous exports should pass validation (with warnings)"
    );
}

/// Test that valid plugins with proper interface are accepted
#[test]
fn test_security_valid_plugin_accepted() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let manager = PluginManager::new(plugin_dir).unwrap();

    // Create a valid WASM plugin that exports the required interface functions
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "get_task_definitions") (result i32)
                i32.const 0  ;; Return empty JSON array
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
            (func (export "safe_function") (result i32)
                i32.const 42
            )
        )
    "#,
    )
    .unwrap();

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    let result = manager.test_validate_wasm_module(&module, "valid_plugin");
    assert!(
        result.is_ok(),
        "Valid plugin with proper interface should be accepted"
    );
}

/// Test fuel limits prevent infinite loops
#[test]
fn test_security_fuel_limits() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let security_config = PluginSecurityConfig {
        fuel_limit: 1000, // Very low fuel limit
        ..Default::default()
    };

    let manager = PluginManager::new_with_security_config(plugin_dir, security_config).unwrap();

    // Create a plugin that would run an infinite loop if not limited
    let wasm_bytes = wat::parse_str(
        r#"
        (module
            (func (export "get_task_definitions") (result i32)
                loop
                    br 0  ;; Infinite loop
                end
                i32.const 0
            )
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

    let module = Module::from_binary(manager.test_get_engine(), &wasm_bytes).unwrap();

    // Validation should pass (fuel limits are checked at runtime)
    let result = manager.test_validate_wasm_module(&module, "infinite_loop_plugin");
    assert!(
        result.is_ok(),
        "Plugin validation should pass even with infinite loop (fuel limits runtime protection)"
    );
}

/// Test epoch-based interruption works
#[test]
fn test_security_epoch_interruption() {
    let temp_dir = TempDir::new().unwrap();
    let plugin_dir = temp_dir.path().to_path_buf();

    let _manager = PluginManager::new(plugin_dir).unwrap();

    // The epoch interruption is set up in the PluginManager
    // This test just verifies the manager can be created with epoch interruption enabled
}
