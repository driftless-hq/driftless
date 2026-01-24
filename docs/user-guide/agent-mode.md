# Agent Mode

Driftless agent mode provides continuous configuration enforcement, metrics collection, and log forwarding for infrastructure automation and monitoring.

### Features

- **Continuous Configuration Enforcement**: Automatically applies configuration changes at specified intervals
- **Metrics Collection**: Gathers system metrics and exposes them via Prometheus-compatible endpoint
- **Log Forwarding**: Collects and forwards logs to various destinations (S3, HTTP, syslog, etc.)
- **Configuration Drift Detection**: Monitors for configuration drift and automatically corrects it
- **Resource Monitoring**: Built-in resource usage monitoring with configurable limits
- **Circuit Breaker Pattern**: Graceful degradation when individual components fail
- **Hot Configuration Reload**: Automatically reloads configuration when files change

### Quick Start

1. Create agent configuration:
```yaml
# ~/.config/driftless/config/agent.yml
config_dir: "~/.config/driftless/config"
apply_interval: 300    # 5 minutes
facts_interval: 60     # 1 minute
apply_dry_run: false
metrics_port: 8000
enabled: true
```

2. Create apply configuration:
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
```

3. Create facts configuration:
```yaml
# ~/.config/driftless/config/facts.yml
collectors:
  - type: cpu
    interval: 60

  - type: memory
    interval: 60

  - type: disk
    interval: 300
    paths: ["/", "/var", "/tmp"]

exporters:
  - type: prometheus
    port: 8000
```

4. Create logs configuration:
```yaml
# ~/.config/driftless/config/logs.yml
sources:
  - type: file
    name: nginx-access
    paths: ["/var/log/nginx/access.log"]
    parser: common

  - type: file
    name: system-auth
    paths: ["/var/log/auth.log"]
    parser: syslog

outputs:
  - type: s3
    name: log-archive
    bucket: my-logs-bucket
    region: us-east-1
    prefix: logs/
    compression:
      algorithm: gzip

  - type: http
    name: elk-forwarder
    url: http://elasticsearch:9200/_bulk
    method: POST
    batch:
      max_size: 100
      max_age: 60
```

5. Start the agent:
```bash
driftless agent
```

### Configuration Options

#### Agent Configuration (`agent.yml`)

```yaml
# Directory containing configuration files to monitor
config_dir: "~/.config/driftless/config"

# Interval for running apply tasks (seconds)
apply_interval: 300

# Interval for collecting facts (seconds)
facts_interval: 60

# Whether to run apply tasks in dry-run mode
apply_dry_run: false

# Port for Prometheus metrics endpoint
metrics_port: 8000

# Whether agent is enabled
enabled: true
```

#### Apply Configuration (`apply.yml`)

Standard apply configuration with additional agent-specific options:

```yaml
# Apply tasks to run continuously
tasks:
  - type: package
    name: nginx
    state: present

  - type: service
    name: nginx
    state: started
    enabled: true

# Agent-specific settings
agent:
  # Maximum execution time per apply cycle (seconds)
  timeout: 300
  # Continue on individual task failures
  continue_on_error: true
```

#### Facts Configuration (`facts.yml`)

```yaml
# Facts collectors to run
collectors:
  - type: cpu
    interval: 60
    enabled: true

  - type: memory
    interval: 60
    enabled: true

  - type: disk
    interval: 300
    paths: ["/", "/var", "/tmp"]
    enabled: true

  - type: network
    interval: 60
    interfaces: ["eth0", "wlan0"]
    enabled: true

# Exporters for collected facts
exporters:
  - type: prometheus
    port: 8000
    path: "/metrics"
    enabled: true

  - type: s3
    bucket: my-metrics-bucket
    region: us-east-1
    prefix: metrics/
    interval: 300
    enabled: true
```

#### Logs Configuration (`logs.yml`)

```yaml
# Log sources to monitor
sources:
  - type: file
    name: nginx-access
    paths: ["/var/log/nginx/access.log", "/var/log/nginx/error.log"]
    parser: common
    multiline:
      pattern: '^\d{4}-\d{2}-\d{2}'
      negate: false
    enabled: true

  - type: file
    name: application
    paths: ["/var/log/application/*.log"]
    parser: json
    enabled: true

# Log outputs for forwarding
outputs:
  - type: s3
    name: log-archive
    bucket: my-logs-bucket
    region: us-east-1
    prefix: logs/
    compression:
      algorithm: gzip
    batch:
      max_size: 1000
      max_age: 300
    enabled: true

  - type: http
    name: elk-forwarder
    url: http://elasticsearch:9200/_bulk
    method: POST
    headers:
      Content-Type: "application/x-ndjson"
    auth:
      type: basic
      username: elastic
      password: "{{ elasticsearch_password }}"
    batch:
      max_size: 100
      max_age: 60
    enabled: true

  - type: syslog
    name: local-syslog
    facility: user
    severity: info
    enabled: true

  - type: file
    name: local-archive
    path: "/var/log/driftless/archive"
    rotation:
      max_size: "100MB"
      max_age: "7d"
      max_files: 10
    enabled: true
```

### Monitoring and Metrics

The agent exposes Prometheus-compatible metrics at `http://localhost:8000/metrics`:

```
# HELP driftless_agent_apply_execution_count_total Total number of apply executions
# TYPE driftless_agent_apply_execution_count_total counter
driftless_agent_apply_execution_count_total 42

# HELP driftless_agent_apply_execution_duration_seconds Duration of apply executions
# TYPE driftless_agent_apply_execution_duration_seconds histogram
driftless_agent_apply_execution_duration_seconds_bucket{le="0.1"} 0
driftless_agent_apply_execution_duration_seconds_bucket{le="0.5"} 2
driftless_agent_apply_execution_duration_seconds_bucket{le="1"} 5
driftless_agent_apply_execution_duration_seconds_bucket{le="5"} 40
driftless_agent_apply_execution_duration_seconds_bucket{le="10"} 42
driftless_agent_apply_execution_duration_seconds_bucket{le="+Inf"} 42

# HELP driftless_agent_facts_collection_count_total Total number of facts collections
# TYPE driftless_agent_facts_collection_count_total counter
driftless_agent_facts_collection_count_total 120

# HELP driftless_agent_logs_processed_entries_total Total number of log entries processed
# TYPE driftless_agent_logs_processed_entries_total counter
driftless_agent_logs_processed_entries_total 15432
```

### Operational Commands

```bash
# Start agent in foreground
driftless agent

# Start agent with specific config directory
driftless agent --config-dir /etc/driftless/config

# Start agent with custom log level
RUST_LOG=debug driftless agent

# Check agent status (when running)
curl http://localhost:8000/status

# View agent metrics
curl http://localhost:8000/metrics

# Stop agent gracefully (send SIGTERM)
kill $(pgrep driftless)
```

### Common Use Cases

#### Infrastructure Monitoring Agent
```yaml
# agent.yml
config_dir: "/etc/driftless/config"
apply_interval: 3600  # 1 hour
facts_interval: 60    # 1 minute
metrics_port: 9090

# facts.yml
collectors:
  - type: system
  - type: cpu
  - type: memory
  - type: disk
  - type: network

exporters:
  - type: prometheus
    port: 9090
```

#### Log Aggregation Agent
```yaml
# agent.yml
config_dir: "/etc/driftless/config"
facts_interval: 300   # 5 minutes (reduced frequency)

# logs.yml
sources:
  - type: file
    name: all-logs
    paths: ["/var/log/**/*.log"]
    parser: auto

outputs:
  - type: s3
    bucket: centralized-logs
    region: us-east-1
    compression: {algorithm: gzip}
```

#### Configuration Enforcement Agent
```yaml
# agent.yml
config_dir: "/etc/driftless/config"
apply_interval: 600   # 10 minutes
apply_dry_run: false

# apply.yml
tasks:
  - type: package
    name: security-tools
    state: present

  - type: file
    path: "/etc/security/policy.conf"
    state: present
    content: |
      # Security policy enforced by driftless
      enforce_password_policy = true
```

### Troubleshooting

#### Agent Won't Start
```bash
# Check configuration syntax
driftless agent --validate-config

# Check file permissions
ls -la ~/.config/driftless/config/

# Check logs
RUST_LOG=debug driftless agent 2>&1 | head -50
```

#### High Resource Usage
```bash
# Check metrics endpoint
curl http://localhost:8000/metrics | grep driftless_agent

# Reduce collection intervals
# Edit agent.yml and restart agent
```

#### Configuration Not Reloading
```bash
# Check file permissions
ls -la ~/.config/driftless/config/agent.yml

# Verify configuration syntax
driftless agent --validate-config

# Check agent logs for reload messages
```