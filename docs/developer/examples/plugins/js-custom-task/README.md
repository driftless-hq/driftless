# Driftless JavaScript Custom Task Plugin Example

This example demonstrates how to create custom tasks for the Driftless agent using JavaScript and WebAssembly.

## Overview

This plugin provides two custom tasks:

1. **js_echo**: Logs messages at different levels (info, warn, error)
2. **js_calculate**: Evaluates mathematical expressions safely

## Building

```bash
# Install dependencies
npm install

# Build the plugin
npm run build
```

This creates a bundled JavaScript file in the `dist/` directory that can be used with WebAssembly.

## Usage

After building, the JavaScript file needs to be packaged for use with Driftless. In a real implementation, this would be compiled to WebAssembly using tools like `javy` or similar.

## Configuration Examples

### Echo Task

```yaml
apply:
  - name: log-hello-js
    plugin: js-custom-task-plugin
    task: js_echo
    config:
      message: "Hello from JavaScript plugin!"
      level: info
```

### Calculate Task

```yaml
apply:
  - name: calculate-result
    plugin: js-custom-task-plugin
    task: js_calculate
    config:
      expression: "2 + 3 * 4"
      variable: "result"
```

## Plugin Interface

This plugin implements the required Driftless plugin interface:

- `get_task_definitions()`: Returns JSON array of task definitions with schemas
- `execute_task(name, config)`: Executes the specified task
- Other required functions return empty arrays (not implemented in this example)

## Security Considerations

- **Expression Evaluation**: The `js_calculate` task uses `Function()` constructor for evaluation, which is safer than `eval()` but still requires careful input validation
- **Input Sanitization**: All inputs should be validated before processing
- **Resource Limits**: JavaScript execution is subject to the same WASM resource limits

## Limitations

This example demonstrates JavaScript plugin development, but in practice:

- JavaScript needs to be compiled to WebAssembly for use with Driftless
- Tools like `javy` (Shopify) or `wasm-pack` with appropriate bindings are needed
- Performance may be different from native WASM languages like Rust