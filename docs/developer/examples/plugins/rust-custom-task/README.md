# Driftless Custom Task Plugin Example

This example demonstrates how to create custom tasks for the Driftless agent using Rust and WebAssembly.

## Overview

This plugin provides two custom tasks:

1. **echo**: Logs messages at different levels (info, warn, error)
2. **file_touch**: Creates empty files with specified permissions

## Building

```bash
# Install wasm-pack if not installed
cargo install wasm-pack

# Build the plugin
wasm-pack build --target web --release --out-dir pkg
```

## Usage

After building, copy the `.wasm` file to your Driftless plugins directory:

```bash
cp pkg/driftless_custom_task_plugin_bg.wasm ~/.driftless/plugins/
```

## Configuration Examples

### Echo Task

```yaml
apply:
  - name: log-hello
    plugin: driftless-custom-task-plugin
    task: echo
    config:
      message: "Hello from custom plugin!"
      level: info
```

### File Touch Task

```yaml
apply:
  - name: create-marker-file
    plugin: driftless-custom-task-plugin
    task: file_touch
    config:
      path: "/tmp/driftless-marker"
      mode: "644"
```

## Plugin Interface

This plugin implements the required Driftless plugin interface:

- `get_task_definitions()`: Returns JSON array of task definitions with schemas
- `execute_task(name, config)`: Executes the specified task
- Other required functions return empty arrays (not implemented in this example)

## Security Notes

- This plugin demonstrates safe operations that don't require host system access
- In a real implementation, file operations would use host-provided functions
- All input validation is performed using JSON schemas