# Driftless Template Extension Plugin Example

This example demonstrates how to create custom Jinja2 filters and functions for use in Driftless templates using Rust and WebAssembly.

## Overview

This plugin provides several template extensions:

### Filters
- **base64_encode**: Base64 encode strings
- **base64_decode**: Base64 decode strings
- **slugify**: Convert strings to URL-friendly slugs

### Functions
- **random_string**: Generate random alphanumeric strings

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
cp pkg/driftless_template_extension_plugin_bg.wasm ~/.driftless/plugins/
```

## Template Examples

### Using Filters

```jinja2
# Base64 encoding
{{ "Hello, World!" | base64_encode }}
# Output: SGVsbG8sIFdvcmxkIQ==

# Slug generation
{{ "Hello, World! This is a Test." | slugify }}
# Output: hello-world-this-is-a-test

# Base64 round-trip
{{ "Secret message" | base64_encode | base64_decode }}
# Output: Secret message
```

### Using Functions

```jinja2
# Generate random strings
{{ random_string() }}
# Output: aB3kL9mP2qR8sT

# Generate random string of specific length
{{ random_string(32) }}
# Output: aB3kL9mP2qR8sT5uV7wX9yZ1cD4eF6gH
```

## Configuration

Template extensions can be configured in Driftless configuration:

```yaml
templates:
  extensions:
    - plugin: driftless-template-extension-plugin
      filters: ["base64_encode", "base64_decode", "slugify"]
      functions: ["random_string"]
```

## Plugin Interface

This plugin implements the required Driftless plugin interface:

- `get_template_extensions()`: Returns JSON array of extension definitions with schemas
- `execute_template_filter()`: Executes template filters
- `execute_template_function()`: Executes template functions
- Other required functions return empty arrays (not implemented in this example)

## Security Notes

- Template extensions run in the same secure sandbox as other plugins
- Input validation is performed using JSON schemas
- No host system access is required for these text processing operations