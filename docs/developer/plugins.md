# Plugin Development Guide

This guide covers creating, building, and deploying plugins for the Driftless system. Plugins are WebAssembly (WASM) modules that extend Driftless functionality with custom tasks, facts collectors, template extensions, and log processing components.

## Overview

Driftless plugins are compiled to WebAssembly and run in a secure sandbox with strict resource limits and execution timeouts. Plugins communicate with the host system through a JSON-based API, ensuring cross-language compatibility.

## Plugin Architecture

### Security Model

Plugins run in a restricted WebAssembly environment with:
- **Memory limits**: 64MB per plugin instance (configurable)
- **Execution timeouts**: 30 seconds maximum (configurable)
- **Fuel limits**: 1 billion instructions per execution
- **No host system access**: No filesystem, network, or system calls
- **Import validation**: Dangerous imports are blocked

### Plugin Types

Plugins can register the following component types:

1. **Tasks**: Custom automation tasks (apply, facts, logs)
2. **Facts Collectors**: System information gathering
3. **Template Extensions**: Custom Jinja2 filters and functions
4. **Log Sources**: Custom log data sources
5. **Log Parsers**: Custom log parsing logic
6. **Log Filters**: Custom log filtering rules
7. **Log Outputs**: Custom log output destinations

## Getting Started

### Examples

Before diving into the details, check out our [plugin examples](../examples/plugins/README.md) that demonstrate complete working plugins in multiple languages:

- **Rust**: Custom tasks and template extensions
- **JavaScript**: Custom tasks with webpack bundling
- **TypeScript**: Type-safe template extensions
- **Python**: Facts collectors (experimental)

Each example includes source code, build instructions, and usage documentation.

### Prerequisites

- Rust 1.92+ with `wasm32-wasi` target
- `wasm-pack` for building and packaging
- Basic knowledge of WebAssembly concepts

### Setting Up a Plugin Project

Create a new Rust library project:

```bash
cargo new --lib my-plugin
cd my-plugin
```

Add dependencies to `Cargo.toml`:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wasm-bindgen = "0.2"

[dependencies.driftless-plugin]
# Use local path during development
path = "../driftless/src/plugin_interface"
# Or use published crate when available
# version = "0.1"
```

### Basic Plugin Structure

```rust
use serde_json::Value;
use wasm_bindgen::prelude::*;

// Export required plugin interface functions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Helper macro for logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn get_task_definitions() -> String {
    let definitions = vec![
        serde_json::json!({
            "name": "my_custom_task",
            "type": "apply",
            "config_schema": {
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                },
                "required": ["message"]
            }
        })
    ];

    serde_json::to_string(&definitions).unwrap()
}

#[wasm_bindgen]
pub fn execute_task(name: &str, config_json: &str) -> String {
    match name {
        "my_custom_task" => {
            let config: Value = serde_json::from_str(config_json).unwrap();
            let message = config["message"].as_str().unwrap();

            console_log!("Executing custom task with message: {}", message);

            // Task implementation here
            serde_json::json!({
                "status": "success",
                "message": format!("Task executed with: {}", message)
            }).to_string()
        }
        _ => serde_json::json!({
            "status": "error",
            "message": format!("Unknown task: {}", name)
        }).to_string()
    }
}
```

## API Reference

### Required Exports

All plugins must export these functions:

#### `get_task_definitions() -> String`
Returns a JSON array of task definitions.

**Format:**
```json
[{
  "name": "task_name",
  "type": "apply|facts|logs",
  "config_schema": {
    "type": "object",
    "properties": {...},
    "required": [...]
  }
}]
```

#### `get_facts_collectors() -> String`
Returns a JSON array of facts collector definitions.

#### `get_template_extensions() -> String`
Returns a JSON array of template extension definitions.

#### `get_log_sources() -> String`
Returns a JSON array of log source definitions.

#### `get_log_parsers() -> String`
Returns a JSON array of log parser definitions.

#### `get_log_filters() -> String`
Returns a JSON array of log filter definitions.

#### `get_log_outputs() -> String`
Returns a JSON array of log output definitions.

### Execution Functions

#### `execute_task(name: &str, config_json: &str) -> String`
Execute a registered task.

**Parameters:**
- `name`: Task name
- `config_json`: JSON string of task configuration

**Returns:** JSON string with execution result or error

#### `execute_facts_collector(name: &str, config_json: &str) -> String`
Execute a facts collector.

#### `execute_log_source(name: &str, config_json: &str) -> String`
Execute a log source.

#### `execute_log_parser(name: &str, config_json: &str, input: &str) -> String`
Execute a log parser.

#### `execute_log_filter(name: &str, config_json: &str, entry_json: &str) -> String`
Execute a log filter.

#### `execute_log_output(name: &str, config_json: &str, entry_json: &str) -> String`
Execute a log output.

#### `execute_template_filter(name: &str, config_json: &str, value_json: &str, args_json: &str) -> String`
Execute a template filter.

#### `execute_template_function(name: &str, config_json: &str, args_json: &str) -> String`
Execute a template function.

### Host Imports (Available)

#### `host_log(level: &str, message: &str)`
Log a message from the plugin.

**Parameters:**
- `level`: "error", "warn", "info", "debug"
- `message`: Log message string

#### `host_get_timestamp() -> u64`
Get current Unix timestamp.

## Security Guidelines

### Memory Management

- Plugins are limited to 64MB of memory per instance
- Avoid memory leaks by properly managing allocations
- Use stack-allocated data when possible

### Execution Limits

- Plugins have a 30-second execution timeout
- CPU usage is limited to 1 billion instructions per execution
- Long-running operations should be split into smaller tasks

### Input Validation

- Always validate input parameters
- Use JSON schemas for configuration validation
- Sanitize string inputs to prevent injection attacks

### Safe Coding Practices

- Avoid unsafe Rust code
- Don't use system calls or external libraries
- Don't attempt to access host filesystem or network
- Use only the provided host import functions

### Forbidden Imports

The following imports are blocked for security:

- `wasi_snapshot_preview1.*` (when WASI is disabled)
- `env.syscall*`, `env.system*` (system calls)
- `env.fd_*`, `env.path_*` (filesystem access)
- `env.sock*`, `env.net*` (network access)

## Building Plugins

### Development Build

```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Build for development
wasm-pack build --target web --out-dir pkg
```

### Production Build

```bash
# Build optimized WASM module
wasm-pack build --target web --release --out-dir pkg
```

### Cross-Platform Considerations

- Plugins run on the same platforms as Driftless (Linux, macOS, Windows)
- Use `wasm32-wasi` target for WASI support (if enabled)
- Test on target platforms before release

## Deployment

### Plugin Directory Structure

Plugins should be placed in Driftless's plugin directory:

```
~/.driftless/plugins/
├── my-plugin.wasm
├── another-plugin.wasm
└── ...
```

### Configuration

Add plugin security configuration to `plugins.toml`:

```toml
[security]
max_memory = 67108864        # 64MB
fuel_limit = 1000000000      # 1B instructions
execution_timeout_secs = 30  # 30 seconds
allow_wasi = false           # No WASI access
debug_enabled = false        # No debug features
```

### Registry Publishing

Plugins can be published to registries for distribution:

```toml
[[registries]]
name = "my-registry"
url = "https://plugins.example.com"
enabled = true
```

## GitHub Actions Workflow

Create `.github/workflows/release-plugin.yml` for automated plugin building and publishing:

```yaml
name: Release Plugin

on:
  push:
    tags:
      - 'v*'

jobs:
  build-and-release:
    runs-on: ubuntu-latest

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

    - name: Build WASM plugin
      run: wasm-pack build --target web --release --out-dir pkg

    - name: Create release archive
      run: |
        cd pkg
        tar -czf ../my-plugin-${{ github.ref_name }}.tar.gz *

    - name: Create GitHub Release
      uses: softprops/action-gh-release@v1
      with:
        files: my-plugin-${{ github.ref_name }}.tar.gz
        generate_release_notes: true
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish-to-registry:
    runs-on: ubuntu-latest
    if: github.event_name == 'release'

    steps:
    - name: Checkout code
      uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

    - name: Build and package
      run: |
        wasm-pack build --target web --release --out-dir pkg
        cd pkg
        # Create plugin metadata
        echo '{"name":"my-plugin","version":"'${{ github.ref_name }}'","description":"My custom plugin"}' > plugin.json

    - name: Upload to registry
      run: |
        # This would upload to your plugin registry
        # Implementation depends on your registry API
        echo "Plugin built and ready for registry upload"
```

## Testing Plugins

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_definitions() {
        let definitions: Vec<Value> = serde_json::from_str(&get_task_definitions()).unwrap();
        assert!(!definitions.is_empty());
        assert_eq!(definitions[0]["name"], "my_custom_task");
    }

    #[test]
    fn test_task_execution() {
        let config = r#"{"message": "test"}"#;
        let result: Value = serde_json::from_str(&execute_task("my_custom_task", config)).unwrap();
        assert_eq!(result["status"], "success");
    }
}
```

### Integration Testing

Create a test harness that loads and executes your plugin:

```rust
use wasmtime::{Engine, Module, Store};

#[test]
fn test_plugin_integration() {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "pkg/my_plugin_bg.wasm").unwrap();
    let mut store = Store::new(&engine, ());

    // Test plugin loading and basic functionality
    // ... test implementation
}
```

## Best Practices

### Performance

- Minimize memory allocations
- Use efficient data structures
- Avoid unnecessary string conversions
- Profile WASM execution time

### Error Handling

- Return structured error responses
- Use appropriate HTTP status codes in JSON responses
- Log errors for debugging

### Documentation

- Document all exported functions
- Provide JSON schema for configurations
- Include examples in documentation

### Versioning

- Use semantic versioning
- Document breaking changes
- Test compatibility with Driftless versions

## Troubleshooting

### Common Issues

**Plugin fails to load:**
- Check WASM compilation target
- Verify all required exports are present
- Check for forbidden imports

**Execution timeouts:**
- Optimize algorithm complexity
- Split large operations
- Increase timeout limits (if allowed)

**Memory limits exceeded:**
- Reduce memory usage
- Use streaming for large data
- Increase memory limits (if allowed)

**Security violations:**
- Remove forbidden imports
- Use only allowed host functions
- Follow security guidelines

### Debug Logging

Enable debug logging in plugin configuration:

```toml
[security]
debug_enabled = true
```

Use the host logging function:

```rust
console_log!("Debug message: {:?}", some_value);
```

## Examples

### Custom Task Plugin

See `examples/custom-task-plugin/` for a complete example.

### Template Filter Plugin

See `examples/template-filter-plugin/` for custom Jinja2 filters.

### Facts Collector Plugin

See `examples/facts-collector-plugin/` for system information gathering.

## Contributing

- Follow the security guidelines
- Include comprehensive tests
- Update documentation
- Use conventional commit messages

## Support

- Check the [Driftless documentation](https://driftless.dev/docs)
- Open issues on GitHub for bugs or feature requests
- Join the community Discord for discussions</content>
<parameter name="filePath">/workspaces/driftless/docs/developer/plugins.md