# Driftless TypeScript Template Extension Plugin Example

This example demonstrates how to create custom Jinja2 filters and functions for use in Driftless templates using TypeScript and WebAssembly.

## Overview

This plugin provides several template extensions:

### Filters
- **uppercase_words**: Capitalize the first letter of each word
- **extract_domain**: Extract domain from email addresses or URLs

### Functions
- **format_date**: Format current date/time with custom patterns
- **uuid_v4**: Generate random UUID v4 strings

## Building

```bash
# Install dependencies
npm install

# Build the plugin (includes WebAssembly compilation)
npm run build
```

This compiles TypeScript to JavaScript, bundles it with webpack, and then compiles the JavaScript to WebAssembly using `javy`.

## Usage

After building, the `dist/driftless-ts-template-extension-plugin.wasm` file can be used directly with Driftless as a plugin.

### Manual Build Steps

If you prefer to build step-by-step:

```bash
# Compile TypeScript
npm run dev

# Bundle JavaScript
npm run build:dev

# Compile to WebAssembly
npm run compile-wasm
```

## Template Examples

### Using Filters

```jinja2
# Capitalize words
{{ "hello world example" | uppercase_words }}
# Output: Hello World Example

# Extract domains
{{ "user@example.com" | extract_domain }}
# Output: example.com

{{ "https://github.com/user/repo" | extract_domain }}
# Output: github.com
```

### Using Functions

```jinja2
# Format current date
{{ format_date() }}
# Output: 2024-01-25

{{ format_date("YYYY-MM-DD HH:mm:ss") }}
# Output: 2024-01-25 14:30:45

# Generate UUIDs
{{ uuid_v4() }}
# Output: 550e8400-e29b-41d4-a716-446655440000
```

## Configuration

Template extensions can be configured in Driftless configuration:

```yaml
templates:
  extensions:
    - plugin: ts-template-extension-plugin
      filters: ["uppercase_words", "extract_domain"]
      functions: ["format_date", "uuid_v4"]
```

## Plugin Interface

This plugin implements the required Driftless plugin interface:

- `get_template_extensions()`: Returns JSON array of extension definitions with schemas
- `execute_template_filter()`: Executes template filters
- `execute_template_function()`: Executes template functions
- Other required functions return empty arrays (not implemented in this example)

## TypeScript Benefits

- **Type Safety**: Compile-time type checking prevents runtime errors
- **Better IDE Support**: Autocomplete and refactoring tools
- **Maintainability**: Interfaces and types make code self-documenting
- **Developer Experience**: Enhanced debugging and development workflow

## Security Notes

- Template extensions run in the same secure sandbox as other plugins
- Input validation is performed using JSON schemas
- No host system access is required for these text processing operations
- TypeScript provides additional safety through static typing

## Limitations

This example demonstrates TypeScript plugin development, but in practice:

- TypeScript/JavaScript needs to be compiled to WebAssembly for use with Driftless
- Tools like `javy` (Shopify) or `wasm-pack` with appropriate bindings are needed
- Runtime performance may differ from native WASM languages like Rust