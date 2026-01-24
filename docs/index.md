# Driftless

> **Warning:** This is an experimental, AI-assisted project. Bugs and other shortcomings are expected and should be reported as GitHub Issues. PRs are welcome!

Streamlined system configuration, inventory, and monitoring agent with Ansible-like configuration operations, plus facts collection and log management.

## Features

- **Idempotent Configuration**: Define desired system state with YAML/JSON configurations
- **Multi-Platform**: Supports Linux systems with various package managers
- **Advanced Template System**: Ansible-like Jinja2 templating with variables, filters, and built-in functions
- **Three Distinct Operation Types**:
  - **Configuration Operations**: Define and enforce desired system state (like Ansible tasks)
  - **Facts Collectors**: Gather system metrics, inventory, and monitoring data
  - **Log Sources/Outputs**: Collect, process, and forward log data
- **Agent Mode**: Continuous monitoring and configuration drift detection
- **Rich Documentation**: Auto-generated comprehensive operation references with examples in YAML, JSON, and TOML
- **Multi-Format Support**: Full configuration examples in all supported formats
- **CI/CD Documentation**: Automatically maintained documentation through GitHub Actions

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

Driftless provides comprehensive auto-generated documentation:

### Generate Documentation
```bash
# Generate comprehensive Markdown documentation with examples in all formats
driftless docs --format markdown --output docs/tasks-reference.md

# Generate JSON schema for validation
driftless docs --format json --output docs/schema/apply-config.schema.json

# Generate all documentation at once
./scripts/generate-docs.sh

# Check if documentation is up-to-date
./scripts/check-docs.sh
```

### View Documentation
- [Configuration Operations Reference](reference/tasks-reference.md) - Complete operation documentation with examples in YAML, JSON, and TOML
- [JSON Schema](schema/apply-config.schema.json) - Schema for configuration validation
- [Example Configurations](examples/) - Comprehensive examples in all supported formats
- [API Documentation](https://driftless-hq.github.io/driftless/api/driftless/) - Generated Rust API docs

### Configuration Formats

Driftless supports three configuration formats, each with comprehensive examples:

#### YAML Examples
```yaml
tasks:
  - type: package
    name: nginx
    state: present
  - type: service
    name: nginx
    state: started
```

#### JSON Examples
```json
{
  "tasks": [
    {
      "type": "package",
      "name": "nginx",
      "state": "present"
    }
  ]
}
```

#### TOML Examples
```toml
[[tasks]]
type = "package"
name = "nginx"
state = "present"
```

## Available Configuration Operations

### System Administration
- **user** - User account management
- **group** - Group management
- **service** - System service control
- **cron** - Scheduled job management
- **mount** - Filesystem mounting
- **hostname** - System hostname configuration
- **timezone** - System timezone settings
- **sysctl** - Kernel parameter management
- **reboot/shutdown** - System power management

### File Operations
- **file** - File creation/modification/removal
- **directory** - Directory management
- **copy** - File copying operations
- **template** - Template file rendering
- **lineinfile** - Line-based file modifications
- **blockinfile** - Multi-line block management
- **replace** - Text replacement in files
- **stat** - File/directory statistics

### Network Operations
- **uri** - HTTP API interactions
- **get_url** - File downloading from URLs
- **fetch** - Remote file fetching (SCP/SFTP)
- **unarchive** - Archive extraction (supports URLs)

### Package Management
- **package** - Generic package management (auto-detects manager)
- **apt** - Debian/Ubuntu packages
- **yum/dnf** - RHEL/CentOS/Fedora packages
- **pacman** - Arch Linux packages
- **zypper** - SUSE packages
- **pip** - Python packages
- **npm** - Node.js packages
- **gem** - Ruby gems

### Command Execution
- **command** - Execute shell commands
- **script** - Execute local scripts
- **raw** - Execute commands without shell processing

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

### Documentation Generation

Documentation is auto-generated during build and CI/CD:

```bash
# Generate all documentation
./scripts/generate-docs.sh

# Check if docs are up-to-date
./scripts/check-docs.sh

# Manual generation of specific docs
driftless docs --format markdown --output docs/tasks-reference.md
driftless docs --format json --output docs/schema/apply-config.schema.json
```

#### Keeping Documentation Current

The documentation is automatically updated through:

1. **CI/CD Pipeline**: GitHub Actions automatically generates and commits updated docs on pushes to main
2. **Build Process**: `build.rs` generates schema and examples during compilation
3. **Development Workflow**: Use `./scripts/check-docs.sh` to verify docs are current

**Best Practice**: Run `./scripts/generate-docs.sh` after making changes to configuration operations.

#### CLI Commands

Driftless provides several CLI commands for different purposes:

```bash
# Configuration Operations
driftless apply                    # Apply configuration operations
driftless apply --dry-run         # Preview changes without applying

# Facts Collection
driftless facts                   # Run facts collectors

# Log Management
driftless logs                    # Run log sources and outputs

# Documentation (NEW)
driftless docs --help             # Show documentation options
driftless docs --format markdown  # Generate comprehensive docs
driftless docs --format json      # Generate JSON schema

# Development
./scripts/generate-docs.sh        # Generate all documentation
./scripts/check-docs.sh          # Verify docs are up-to-date
```

### Documentation Features

Driftless includes comprehensive auto-generated documentation:

#### Multi-Format Examples
All configuration examples are provided in three formats:
- **YAML**: Human-readable, concise syntax
- **JSON**: Machine-readable, API-friendly format
- **TOML**: Configuration-focused, table-based syntax

#### Auto-Generated Content
- **Operation Reference**: Complete documentation of all 36+ configuration operations
- **JSON Schema**: Validation schema for configuration files
- **Practical Examples**: Real-world scenarios with detailed explanations
- **CI/CD Integration**: Documentation automatically updated on code changes

#### Documentation Commands
```bash
# Generate all documentation
./scripts/generate-docs.sh

# Generate specific formats
driftless docs --format markdown  # Operation reference
driftless docs --format json      # JSON schema

# Check documentation status
./scripts/check-docs.sh
```

### Architecture

Driftless follows a modular architecture:

- **`apply/`** - Configuration operations execution engine with idempotent operations
- **`facts/`** - Facts collectors for system information and metrics gathering
- **`logs/`** - Log sources and outputs for log collection and forwarding
- **`docs/`** - Auto-generated documentation utilities

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Update documentation
5. Submit a pull request

### Configuration Operation Development

To add a new configuration operation type:

1. Define the operation struct in `src/apply/mod.rs`
2. Implement the executor in `src/apply/<operation_name>.rs`
3. Add comprehensive documentation and examples
4. Update the operation registry in the executor
5. Add tests

Example configuration operation implementation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyOperation {
    /// Description of the parameter
    pub param: String,
}

pub async fn execute_my_operation(operation: &MyOperation, dry_run: bool) -> Result<()> {
    // Implementation here
    Ok(())
}
```

## License

Licensed under the Apache License, Version 2.0.