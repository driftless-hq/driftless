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
- Default configuration directory: `/etc/driftless/config` (system-wide) or `~/.config/driftless/config` (user)
- Secrets passed via environment variables, `/etc/driftless/secrets.yml`, `/etc/driftless/secrets.env`, `~/.config/driftless/secrets.yml`, or `~/.config/driftless/secrets.env`
- Sub-command names mirror file schemas (i.e. `apply`, `facts`, `logs`) for running tasks
	- The `apply` sub-command should include a `--dry-run` flag or similar to only output diffs
- Additional `agent` sub-command activates agent mode
	- Gathers built-in facts
	- If configured, starts Prometheus metrics endpoint (i.e. `0.0.0.0:8000/metrics)`
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
- [x] Review the entire codebase to find all placeholders, `TODO`, `in practice`, and `in a real implementation` comments and add TODO list items to address them
- Create task prompts in the TODO list that adds support for macOS and Windows operating systems in all applicable areas of the codebase
- Review usages of `dead_code`, `unsafe`, and `unused_imports` to silence warnings and determine if code should be used or cleaned up according to Rust best practices. Use this opportunity to cleanup unused code and dependencies to reduce release binary size and improve maintainability.
- Review the codebase for consistent error-handling patterns and improve as needed
- Ensure all dependencies in `Cargo.toml` are up-to-date with the latest stable versions
- Review the codebase for usage of Rust best-practices and guidelines
- Review the codebase for safety and security vulnerabilities and apply mitigations as needed
- Ensure comprehensive test coverage and cleanup any clippy warnings. Tests should be written for the intent of the code not the implementation details.
- Review the auto-generated and manually-managed documentation in the `docs/` directory and validate information is accurate against the current codebase. Look for cleanup, clarification, expansion, and reorganization opportunities. Ensure all auto-generated documentation contains a banner indicating it is auto-generated and should not be manually edited.
- Perform a final review of the entire codebase, documentation, and project structure to ensure consistency, quality, and readiness for production use.

### Codebase Implementation Gaps (from TODO, placeholder, and implementation comments)

Ensure these task items have full production-ready implementations or provide additional task items if too complex for a single prompt:

#### Template System
- [ ] **Template error handling**: Make undefined/none values in template filters cause rendering failures instead of returning error indications

#### Configuration Comparison
- [ ] **Log output comparison**: Implement proper comparison of log output configurations by type instead of simplified equality check

#### Plugin Examples
- [ ] **TypeScript plugin compilation**: Implement proper WebAssembly compilation for TypeScript plugins using tools like `javy`
- [ ] **JavaScript plugin compilation**: Implement proper WebAssembly compilation for JavaScript plugins using tools like `javy`
- [ ] **Python plugin APIs**: Replace platform-specific placeholder implementations with proper APIs (psutil, socket, etc)

#### Plugin Development
- [ ] **Safe expression evaluation**: Implement safe expression evaluator for JavaScript plugins instead of placeholder comment
