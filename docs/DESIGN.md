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
- **Facts Collection Tasks** - Implement system metrics collection task types:
  - **System Facts Task**: Create `src/facts/system_facts.rs` with hostname, OS info, kernel version, uptime, boot time, and CPU architecture collection. Include comprehensive tests and documentation.
  - **CPU Metrics Task**: Create `src/facts/cpu_facts.rs` with CPU usage, per-core metrics, frequency, temperature, and load average collection. Include threshold monitoring and alerting.
  - **Memory Metrics Task**: Create `src/facts/memory_facts.rs` with total/used/free/available memory, swap usage, and percentage calculations. Include memory pressure monitoring.
  - **Disk Metrics Task**: Create `src/facts/disk_facts.rs` with disk space monitoring, I/O statistics, and mount point filtering. Include disk usage thresholds and alerts.
  - **Network Metrics Task**: Create `src/facts/network_facts.rs` with interface statistics, bytes/packets/errors tracking, and network status monitoring.
  - **Process Metrics Task**: Create `src/facts/process_facts.rs` with process counting, CPU/memory per process, and process status monitoring with pattern matching.
  - **Command Facts Task**: Create `src/facts/command_facts.rs` for executing custom commands and parsing their output (text, JSON, key-value formats).
  - **Facts Orchestrator**: Create `src/facts/facts.rs` to orchestrate all fact collectors with scheduling, aggregation, and export capabilities.
- **Log Collection Tasks** - Implement log processing task types:
  - **File Log Source Task**: Create `src/logs/file_log_source.rs` for tailing log files with rotation handling, encoding support, and multiline log processing.
  - **Log Parser Tasks**: Create `src/logs/log_parsers.rs` with parsers for plain text, JSON, key-value, Apache/Nginx logs, syslog, and custom regex patterns.
  - **Log Filter Tasks**: Create `src/logs/log_filters.rs` with include/exclude patterns, field matching, rate limiting, and content-based filtering.
  - **File Log Output Task**: Create `src/logs/file_log_output.rs` with file rotation, compression, and timestamp-based filename patterns.
  - **S3 Log Output Task**: Create `src/logs/s3_log_output.rs` with batched uploads, compression, and configurable prefixes and regions.
  - **HTTP Log Output Task**: Create `src/logs/http_log_output.rs` with batching, authentication (basic/bearer/API key), retry logic, and compression.
  - **Syslog Output Task**: Create `src/logs/syslog_log_output.rs` with RFC 3164/5424 compliance and configurable facilities/priorities.
  - **Console Log Output Task**: Create `src/logs/console_log_output.rs` for stdout/stderr output with structured formatting.
  - **Log Processing Pipeline**: Create `src/logs/log_pipeline.rs` to orchestrate log sources, parsers, filters, and outputs with buffering and error handling.
- Create task prompts in the TODO list that create, in managable pieces, an extensions/plugins system via the `wasmtime` crate that can create and register task types and template filters and functions.
- Review the codebase for usage of Rust best-practices and guidelines
- Review the codebase for safety, security, and production-readiness
- Ensure comprehensive test coverage
