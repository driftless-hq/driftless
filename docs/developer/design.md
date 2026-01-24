# Design Document for Driftless

## Goals
- Streamlined system configuration, inventory, and monitoring agent
- Built for GitOps using a single repository of truth
- Tiny, efficient, and memory-safe executable (via Rust)

## Features
- Configuration management
	- Applies declarative system configs idempotently
	- Requires list of configuration tasks
	- Writes audit/diff logs to a local directory (i.e. NFS), HTTP endpoint, or S3 bucket
	- Alternative to Ansible/Chef/Puppet
	- Crates: `git2 nix reqwest rust-s3 serde`
- Metrics gathering
	- Gather host metrics (CPU, mem, disk, etc)
	- Requires list of metrics to collect, poll interval, and thresholds
	- Export metrics via `/metrics` endpoint or push to S3 bucket
	- Alternative to Prometheus Node Exporter
	- Crates: `prometheus rust-s3 sysinfo`
- Log collection
	- Tails and forwards logs
	- Requires list of paths to tail and filters/parsers to use
	- Writes logs to local directory (i.e. NFS), S3 bucket, syslog, or HTTP (i.e. ELK stack)
	- Alternative to FileBeat
	- Crates: `flate2 reqwest rust-s3`
- Secrets management
	- Remove secrets from configuration using variable substitution
	- Reads secrets from environment variables and `env` files outside input directory
	- Crates: `secret_vault`

## Design details
- Runs as a CLI
- Uses the given directory of configuration files as input (i.e. cloned Git repo)
- Configuration files use an JSON, TOML, or YAML syntax (auto-detect file extension)
- Configuration file schemas include:
	- `apply`: Idempotent system configuration tasks
	- `facts`: Facts, metrics, and other information gathering tasks
	- `logs`: Log file tailing and forwarding tasks
- Default configuration directory: `~/.config/driftless/config`
- Secrets passed via environment variables or `~/.config/driftless/env`
- Sub-command names mirror file schemas (i.e. `apply`, `facts`, `logs`) for running tasks
	- The `apply` sub-command should include a `--dry-run` flag or similar to only output diffs
- Additional `agent` sub-command activates agent mode
	- Gathers built-in facts
	- If configured, starts Prometheus metrics endpoint (i.e. `0.0.0.0:8000/metrics)
	- Starts an event loop
		- Reads configuration files from directory
		- Gathers configured additional facts and metrics at requested interval
		- Starts collecting and forwarding configured log files
		- Runs apply tasks at requested interval

## Potential Future Enhancements
- Remote secrets provider support (AWS, GCP, Vault/OpenBao, etc)
- Distributed scheduling/task management
- Inventory reporting (hardware/software)
- Reusable modules
- Extensible with plugins via `wasmtime` crate
- Plugin registry and download manager

### Nix Integration Opportunities

The [nix crate](https://github.com/nix-rust/nix) provides Rust bindings to *nix APIs. We can leverage this for:

#### High Priority (System-level operations)
- [ ] **Process management** - Enhanced process monitoring/control
- [ ] **Signal handling** - Send signals to processes
- [ ] **File permissions** - More robust Unix permission handling
- [ ] **User/group operations** - Lower-level user/group management
- [ ] **Mount operations** - Filesystem mounting
- [ ] **Network interfaces** - Network interface management
- [ ] **System information** - Detailed system/hardware info

#### Medium Priority (Infrastructure automation)
- [ ] **Sysctl operations** - Kernel parameter management
- [ ] **Capability management** - Linux capabilities
- [ ] **Namespace operations** - Container/namespace management
- [ ] **Cgroup management** - Control groups
- [ ] **Inotify monitoring** - File system monitoring
- [ ] **Socket operations** - Unix domain sockets

#### Low Priority (Advanced features)
- [ ] **ACL management** - Access control lists
- [ ] **Extended attributes** - File extended attributes
- [ ] **Audit operations** - System audit logging
- [ ] **KVM operations** - Kernel-based virtual machines

## TODO
- **Apply Task Plugins**: Implement support for loading and executing custom apply tasks from plugins, ensuring idempotency and integration with the existing apply executor.
- **Facts Task Plugins**: Implement support for loading and executing custom facts gathering tasks from plugins, allowing plugins to collect and return custom metrics or system information.
- **Logs Task Plugins**: Implement support for loading and executing custom log processing tasks from plugins, enabling custom parsers, filters, and output handlers.
- **Template Extensions**: Add support for plugins to register custom template filters and functions, integrating with the templating engine to allow dynamic extension of template capabilities.
- **Plugin Registry**: Create a plugin registry system that scans a designated directory for plugin files, validates them, and makes them available for use in configurations.
- **Plugin Lifecycle Management**: Implement functionality to list, manage, and install configured plugin executables during startup, ensuring they are available for use. This includes downloading plugins from a registry if needed, validating them, and caching them locally. It is expected that plugins will be distributed as pre-compiled WASM binaries to an artifact registry.
- **Security Hardening**: Implement security measures such as WASM module validation, execution timeouts, memory limits, and restricted system access to prevent malicious plugins from compromising the host system.
- **Plugin Documentation**: Write comprehensive documentation in `docs/developer/plugins.md` explaining how to create plugins, including API references, security guidelines, and deployment instructions. Be sure to include a GitHub workflow example for building and publishing plugins as WASM binaries attached to GitHub Releases.
- **Plugin Examples**: Create example plugins in multiple programming languages (Rust, JavaScript/TypeScript, Python via pyodide if feasible) demonstrating custom tasks and template extensions, placed in `docs/developer/examples/plugins/`.
- **Plugin Testing**: Develop unit and integration tests for the plugin system, including tests for loading, execution, error handling, and security boundaries.
- **Plugin CLI Integration**: Update the CLI to support plugin-related commands (e.g., `driftless plugins list`, `driftless plugins validate <plugin>`) for managing and inspecting loaded plugins.
- Create task prompts in the TODO list that adds support for macOS and Windows operating systems
- Review usages of `dead_code` and `unused_imports` to silence warnings and determine if code should be used or cleaned up
- Ensure all dependencies in `Cargo.toml` are up-to-date with the latest stable versions
- Review the codebase for usage of Rust best-practices and guidelines
- Review the codebase for safety and security vulnerabilities and apply mitigations as needed
- Ensure comprehensive test coverage and cleanup any clippy warnings
- Review the auto-generated and manually-managed documentation in the `docs/` directory and validate information is accurate against the current codebase. Look for cleanup, clarification, expansion, and reorganization opportunities. Ensure all auto-generated documentation contains a banner indicating it is auto-generated and should not be manually edited.
- Perform a final review of the entire codebase, documentation, and project structure to ensure consistency, quality, and readiness for production use.
- Review usages of `dead_code` and `unused_imports` to silence warnings and determine if code should be used or cleaned up
- Review the codebase for usage of Rust best-practices and guidelines
- Review the codebase for safety and security vulnerabilities and apply mitigations as needed
- Ensure comprehensive test coverage and cleanup any clippy warnings
- Review the auto-generated and manually-managed documentation in the `docs/` directory and validate information is accurate against the current codebase. Look for cleanup, clarification, expansion, and reorganization opportunities. Ensure all auto-generated documentation contains a banner indicating it is auto-generated and should not be manually edited.
- Perform a final review of the entire codebase, documentation, and project structure to ensure consistency, quality, and readiness for production use.
