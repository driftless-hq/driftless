# Driftless Python Facts Collector Plugin Example

This example demonstrates how to create custom facts collectors for the Driftless agent using Python and Pyodide for WebAssembly compilation.

## Overview

This plugin provides two custom facts collectors:

1. **python_system_info**: Collects basic system information (CPU, memory, Python version)
2. **python_network_interfaces**: Collects network interface information

## Important Notes

**⚠️ Experimental**: Python plugin support via Pyodide is experimental and may not be fully compatible with the current Driftless plugin system. This example demonstrates the conceptual approach.

**Requirements**:
- Pyodide for Python-to-WebAssembly compilation
- Additional tooling for WASM integration
- May require custom build processes

## Building (Conceptual)

```bash
# Install Pyodide build tools
pip install -r requirements.txt

# Build process would typically involve:
# 1. Using Pyodide to compile Python to WebAssembly
# 2. Creating appropriate JavaScript bindings
# 3. Packaging for Driftless plugin system

# This is not currently implemented in Driftless
pyodide build plugin.py
```

## Usage

If fully implemented, after building, copy the compiled artifacts to your Driftless plugins directory.

## Configuration Examples

### System Info Collector

```yaml
facts:
  - name: python-system-facts
    plugin: python-facts-collector-plugin
    collector: python_system_info
    config:
      include_cpu: true
      include_memory: true
    interval: 300  # 5 minutes
```

### Network Interfaces Collector

```yaml
facts:
  - name: python-network-facts
    plugin: python-facts-collector-plugin
    collector: python_network_interfaces
    config:
      include_loopback: false
    interval: 60  # 1 minute
```

## Sample Output

### System Info Facts
```json
{
  "cpu": {
    "architecture": "x86_64",
    "cores": 8,
    "python_version": "3.11.0"
  },
  "memory": {
    "total_gb": 16,
    "available_gb": 8,
    "used_percent": 50.0
  },
  "timestamp": "2024-01-25T10:30:00Z"
}
```

### Network Interfaces Facts
```json
{
  "eth0": {
    "addresses": ["192.168.1.100"],
    "mac": "00:11:22:33:44:55",
    "status": "up"
  },
  "wlan0": {
    "addresses": ["192.168.1.101"],
    "mac": "66:77:88:99:AA:BB",
    "status": "up"
  }
}
```

## Plugin Interface

This plugin implements the required Driftless plugin interface:

- `get_facts_collectors()`: Returns JSON array of facts collector definitions with schemas
- `execute_facts_collector(name, config)`: Executes the specified facts collector
- Other required functions return empty arrays (not implemented in this example)

## Python Advantages

- **Rich Ecosystem**: Access to extensive Python libraries
- **Familiar Syntax**: Easy for Python developers to contribute
- **Data Science**: Natural fit for data collection and analysis tasks
- **Rapid Development**: Quick prototyping and testing

## Current Limitations

- **Pyodide Integration**: Requires additional WASM compilation tooling
- **Performance**: Python execution in WASM may be slower than native WASM languages
- **Library Compatibility**: Not all Python libraries work in Pyodide environment
- **Security**: Additional sandboxing considerations for Python execution

## Future Considerations

- **WASM Compilation**: Integrate with Pyodide build pipeline
- **Library Support**: Determine which Python libraries are compatible
- **Performance Optimization**: Optimize for WASM execution environment
- **Security Review**: Ensure Python execution doesn't compromise sandbox

## Testing

Run the plugin directly for testing:

```bash
python plugin.py
```

This will output sample facts data for verification.