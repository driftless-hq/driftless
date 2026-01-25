# Plugin Examples

This directory contains example plugins demonstrating how to extend Driftless functionality using different programming languages and WebAssembly.

## Examples Overview

### Rust Examples

#### [Custom Task Plugin](./rust-custom-task/)
- **Language**: Rust
- **Features**: Custom tasks (`echo`, `file_touch`)
- **Demonstrates**: Task definition schemas, execution logic, error handling
- **Build Tool**: `wasm-pack`

#### [Template Extension Plugin](./rust-template-extension/)
- **Language**: Rust
- **Features**: Template filters (`base64_encode`, `slugify`) and functions (`random_string`)
- **Demonstrates**: Filter/function implementation, text processing, random generation
- **Build Tool**: `wasm-pack`

### JavaScript Examples

#### [Custom Task Plugin](./js-custom-task/)
- **Language**: JavaScript
- **Features**: Custom tasks (`js_echo`, `js_calculate`)
- **Demonstrates**: JavaScript plugin development, expression evaluation, logging
- **Build Tool**: Webpack

### TypeScript Examples

#### [Template Extension Plugin](./ts-template-extension/)
- **Language**: TypeScript
- **Features**: Template filters (`uppercase_words`, `extract_domain`) and functions (`format_date`, `uuid_v4`)
- **Demonstrates**: Type-safe plugin development, date formatting, UUID generation
- **Build Tool**: TypeScript + Webpack

### Python Examples (Experimental)

#### [Facts Collector Plugin](./python-facts-collector/)
- **Language**: Python
- **Features**: Facts collectors (`python_system_info`, `python_network_interfaces`)
- **Demonstrates**: Python plugin concepts, system information gathering
- **Build Tool**: Pyodide (experimental)
- **Note**: Python plugin support is experimental and may require additional tooling

## Getting Started

Each example includes:

- **Source code** with comprehensive comments
- **Build instructions** and dependencies
- **Configuration examples** for Driftless
- **README** with detailed usage instructions

### Building Examples

```bash
# Rust examples
cd rust-custom-task
wasm-pack build --target web --release

# JavaScript/TypeScript examples
cd js-custom-task
npm install && npm run build

# Python example (experimental)
cd python-facts-collector
# Build process not fully implemented
```

### Using Examples

1. Build the plugin using the provided instructions
2. Copy the resulting `.wasm` file to `~/.driftless/plugins/`
3. Configure the plugin in your Driftless configuration
4. Run Driftless with the plugin enabled

## Plugin Architecture

All examples follow the same plugin interface:

- **Required Exports**: Functions that return component definitions
- **Execution Functions**: Functions that execute specific components
- **JSON Communication**: All data exchange uses JSON strings
- **Security Sandbox**: All plugins run in WebAssembly sandbox

## Language Comparisons

| Language | Performance | Development Speed | Ecosystem | Type Safety | WASM Maturity |
|----------|-------------|-------------------|-----------|-------------|---------------|
| Rust | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| TypeScript | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| JavaScript | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| Python | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ (experimental) |

## Contributing

When adding new examples:

1. Follow the existing directory structure
2. Include comprehensive README documentation
3. Provide build instructions and dependencies
4. Test the example with Driftless
5. Update this index file

## Security Considerations

All examples demonstrate safe plugin development practices:

- Input validation using JSON schemas
- No dangerous system calls
- Proper error handling
- Resource-aware implementations

Remember: Plugins run with strict resource limits and no host system access.