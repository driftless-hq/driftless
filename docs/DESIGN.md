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

## TODO
- Implement the following encoding/decoding template filters in a new file and ensure they are compatible with the Ansible template filters of the same name and include comprehensive tests and documentation: b64encode, b64decode, to_json, from_json, to_yaml, from_yaml
- Implement the following path/filesystem template filters in a new file and ensure they are compatible with the Ansible template filters of the same name and include comprehensive tests and documentation: expanduser, realpath (beyond the current basename/dirname)
- Implement the following generator template functions in a new file and ensure they are compatible with the Ansible template functions of the same name and include comprehensive tests and documentation: range, random
- Implement the following utility template functions in a new file and ensure they are compatible with the Ansible template functions of the same name and include comprehensive tests and documentation: hash, uuid, timestamp
- Ensure the `lookup` template function is fully complete
- Identify any template features, filters, or functions implemented in Ansible that have not yet been implemented in this codebase
- Review the codebase for usage of Rust best-practices and guidelines
- Review the codebase for safety, security, and production-readiness
- Ensure comprehensive test coverage

### Template Functions That Need Implementing
- Generators: range, random.
- Utilities: hash, uuid, timestamp.
- Ansible-Specific: lookup (for plugins like env, file, etc., though a basic lookup is partially implemented; full plugin support would require extension).