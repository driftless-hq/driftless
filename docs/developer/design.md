# Driftless (Config Management)

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
- Move validation code from the pipeline workflows and into a new script in the `scripts` folder that the workflow calls. This will allow developers to discover potential pipeline failures before committing code. This should include `cargo clippy`, the documentation generator's validation, `cargo fmt`, and any other validation steps performed in the pipeline workflows that could potentially cause a check failure when creating a PR.
- Create a new GitHub workflow that enforces repository settings programmatically using values from `.github/repo-settings.yml`. It should run only when the `.github` folder contents is modified on the `main` branch. It should include options for enabling and configuring main branch protection, preventing pushes to main unless admin, requiring a number of approvals before merging (default to 1), auto-delete PR branches after merge, auto-merge after required checks pass, GitHub Pages source configuration (Actions or branch), and any other commonly used setting that might need to be managed as code. Review the current `driftless-hq/driftless` repository settings and ensure the `.github/repo-settings.yml` file includes all current customizations for the repository.
- Create task prompts in the TODO list that create, in managable pieces, an extensions/plugins system via the `wasmtime` crate that can create and register `apply`, `facts`, and `logs` task types and template filters and functions.
- Create task prompts in the TODO list that adds support for macOS and Windows operating systems
- Review usages of `dead_code` and `unused_imports` to silence warnings and determine if code should be used or cleaned up
- Review the codebase for usage of Rust best-practices and guidelines
- Review the codebase for safety, security, and production-readiness
- Ensure comprehensive test coverage
- Review the README.md and validate information is accurate. Look for cleanup and reorganization opportunities.
