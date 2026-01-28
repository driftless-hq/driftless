# Driftless JavaScript Custom Task Plugin Example

This example demonstrates how to create custom tasks for the Driftless agent using JavaScript and WebAssembly.

## Overview

This plugin provides two custom tasks:

1. **js_echo**: Logs messages at different levels (info, warn, error)
2. **js_calculate**: Evaluates mathematical expressions safely using the `expr-eval` library

## Dependencies

Install the required dependencies:

```bash
npm install
```

### Required Packages
- **expr-eval**: Safe mathematical expression evaluator that prevents code injection
- **webpack**: For bundling the JavaScript

## Building

```bash
# Install dependencies
npm install

# Build the plugin (includes WebAssembly compilation)
npm run build
```

This bundles the JavaScript with webpack and then compiles it to WebAssembly using `javy`.

## Usage

After building, the `dist/driftless-js-custom-task-plugin.wasm` file can be used directly with Driftless as a plugin.

### Manual Build Steps

If you prefer to build step-by-step:

```bash
# Bundle JavaScript
npm run build:dev

# Compile to WebAssembly
npm run compile-wasm
```

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

- **Expression Evaluation**: The `js_calculate` task uses the `expr-eval` library for safe mathematical expression evaluation, preventing code injection attacks
- **Input Sanitization**: All inputs should be validated before processing
- **Resource Limits**: JavaScript execution is subject to the same WASM resource limits

## Limitations

This example demonstrates JavaScript plugin development, but in practice:

- JavaScript needs to be compiled to WebAssembly for use with Driftless
- Tools like `javy` (Shopify) or `wasm-pack` with appropriate bindings are needed
- Performance may be different from native WASM languages like Rust