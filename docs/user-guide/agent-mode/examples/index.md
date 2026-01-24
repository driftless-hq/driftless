# Driftless Agent Configuration Examples

This directory contains example configurations for common Driftless agent use cases. Each example includes all necessary configuration files (agent.yml, apply.yml, facts.yml, logs.yml) for a complete setup.

## Available Examples

### Basic Monitoring Agent
**Files**: `agent-basic-monitoring.yml`, `facts-basic-monitoring.yml`
**Purpose**: Minimal agent setup for collecting essential system metrics
**Use Case**: Simple infrastructure monitoring without complex logging or configuration enforcement
**Features**:
- CPU, memory, disk, and network monitoring
- Prometheus metrics endpoint
- 1-minute collection intervals

#### agent-basic-monitoring.yml
```yaml
{{#include agent-basic-monitoring.yml}}
```

#### facts-basic-monitoring.yml
```yaml
{{#include facts-basic-monitoring.yml}}
```

---

### Log Aggregation Agent
**Files**: `agent-log-aggregation.yml`, `logs-comprehensive.yml`
**Purpose**: Comprehensive log collection from multiple sources with forwarding to various destinations
**Use Case**: Centralized logging infrastructure
**Features**:
- Multi-source log collection (nginx, system, application, Docker)
- Multiple output destinations (S3, ELK stack, syslog, local files)
- Compression and batching for efficiency

#### agent-log-aggregation.yml
```yaml
{{#include agent-log-aggregation.yml}}
```

#### logs-comprehensive.yml
```yaml
{{#include logs-comprehensive.yml}}
```

---

### Configuration Enforcement Agent
**Files**: `agent-config-enforcement.yml`, `apply-config-enforcement.yml`
**Purpose**: Continuous enforcement of system configuration and security policies
**Use Case**: Compliance and security hardening
**Features**:
- Security package installation
- Firewall configuration
- SSH hardening
- System security settings
- Automated security monitoring

#### agent-config-enforcement.yml
```yaml
{{#include agent-config-enforcement.yml}}
```

#### apply-config-enforcement.yml
```yaml
{{#include apply-config-enforcement.yml}}
```

---

### Production-Ready Agent
**Files**: `agent-production.yml`, `facts-production.yml`, `logs-production.yml`, `apply-production.yml`
**Purpose**: Complete production deployment combining monitoring, logging, and configuration enforcement
**Use Case**: Enterprise production environments
**Features**:
- All monitoring capabilities
- Enterprise logging with redundancy
- Comprehensive configuration enforcement
- Security hardening
- Backup and disaster recovery
- TLS encryption for log forwarding

#### agent-production.yml
```yaml
{{#include agent-production.yml}}
```

#### facts-production.yml
```yaml
{{#include facts-production.yml}}
```

#### logs-production.yml
```yaml
{{#include logs-production.yml}}
```

#### apply-production.yml
```yaml
{{#include apply-production.yml}}
```

---

## Configuration File Structure

Each example follows the standard Driftless configuration structure:

```
~/.config/driftless/
├── config/
│   ├── agent.yml      # Agent behavior configuration
│   ├── apply.yml      # Configuration operations to enforce
│   ├── facts.yml      # Metrics collection configuration
│   └── logs.yml       # Log collection and forwarding configuration
└── data/              # Runtime data (created automatically)
```

## Getting Started

1. Choose an example that matches your use case
2. Copy the configuration files to `~/.config/driftless/config/`
3. Edit the configurations to match your environment
4. Set any required environment variables or secrets
5. Start the agent: `driftless agent`

## Environment Variables

Many examples use template variables that should be set as environment variables:

```bash
export AWS_ACCESS_KEY_ID="your-key"
export AWS_SECRET_ACCESS_KEY="your-secret"
export ELASTICSEARCH_PASSWORD="your-password"
export MONITORING_API_KEY="your-api-key"
```

## Customization

These examples are starting points. Customize them for your specific needs:

- Adjust collection intervals based on your monitoring requirements
- Modify file paths to match your system layout
- Configure appropriate authentication for external services
- Add additional tasks, collectors, or log sources as needed

## Security Considerations

- Store sensitive configuration in environment variables, not in config files
- Use TLS encryption for log forwarding in production
- Implement proper access controls for metrics endpoints
- Regularly rotate credentials and API keys
- Monitor agent resource usage and adjust limits as needed

## Troubleshooting

If the agent fails to start:

1. Validate configuration syntax: `driftless agent --validate-config`
2. Check file permissions on configuration files
3. Verify network connectivity to external services
4. Review agent logs with `RUST_LOG=debug driftless agent`

For issues with specific components:

- **Apply**: Check task definitions and system permissions
- **Facts**: Verify collector configurations and system access
- **Logs**: Check file paths, permissions, and output destinations