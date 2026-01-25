# Driftless

> **Warning:** This is an experimental, AI-assisted project. Bugs and other shortcomings are expected and should be reported as GitHub Issues. PRs are welcome!

A lightweight Rust agent for declarative system configuration, metrics gathering, and log forwarding via GitOps.

## Features

- **Idempotent Configuration**: Define desired system state with YAML/JSON/TOML configurations
- **Multi-Platform**: Supports Linux systems with various package managers
- **Advanced Template System**: Ansible-like Jinja2 templating with variables, filters, and built-in functions
- **Three Distinct Operation Types**:
  - **Configuration Operations**: Define and enforce desired system state (like Ansible tasks)
  - **Facts Collectors**: Gather system metrics, inventory, and monitoring data
  - **Log Sources/Outputs**: Collect, process, and forward log data
- **Agent Mode**: Continuous monitoring and configuration drift detection
- **Rich Documentation**: Comprehensive operation references with examples in YAML, JSON, and TOML

## Installation

```bash
cargo install driftless
```

## Quick Start

1. Create a configuration directory:
```bash
mkdir -p ~/.config/driftless/config
```

2. Create an apply configuration:
```yaml
# ~/.config/driftless/config/apply.yml
tasks:
  - type: package
    name: nginx
    state: present

  - type: service
    name: nginx
    state: started
    enabled: true

  - type: file
    path: /etc/nginx/sites-available/default
    state: present
    content: |
      server {
          listen 80;
          server_name _;
          root /var/www/html;
          index index.html;
      }
    mode: "0644"
```

3. Apply the configuration:
```bash
driftless apply
```

## Documentation

Driftless provides comprehensive documentation:

- [User Guide](user-guide/index.md) - Getting started, configuration examples, and agent mode
- [Reference](reference/index.md) - Complete operation references, facts, logs, and templates
- [Developer Guide](developer/index.md) - Development, plugins, and contributing

### CLI Commands

Driftless provides several CLI commands for different purposes:

```bash
# Configuration Operations
driftless apply                    # Apply configuration operations
driftless apply --dry-run         # Preview changes without applying

# Facts Collection
driftless facts                   # Run facts collectors

# Log Management
driftless logs                    # Run log sources and outputs

# Agent Mode
driftless agent                   # Run in continuous monitoring mode
```

## Configuration

Driftless supports multiple configuration formats:

### Directory Structure
```
~/.config/driftless/
├── config/
│   ├── apply.yml       # Configuration operation definitions
│   ├── facts.yml       # Facts collector settings
│   └── logs.yml        # Log source/output settings
└── data/               # Runtime data directory
```

### Configuration Formats

All configuration files support YAML, JSON, or TOML formats. Files can be named with any extension (`.yml`, `.yaml`, `.json`, `.toml`) and Driftless will auto-detect the format.

## Development

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Update documentation
5. Submit a pull request

For detailed development information, see the [Developer Guide](developer/index.md).

## License

Licensed under the Apache License, Version 2.0.