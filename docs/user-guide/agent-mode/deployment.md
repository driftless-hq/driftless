# Agent Deployment Guide

This guide covers deploying the Driftless agent in production environments.

## Prerequisites

- Linux system (Ubuntu 18.04+, CentOS 7+, or equivalent)
- Rust toolchain (for building from source)
- Systemd (for service management)
- Network access for metrics and log shipping

## Installation

### Option 1: Install from Source

```bash
# Clone the repository
git clone https://github.com/driftless-hq/driftless.git
cd driftless

# Build the binary
cargo build --release

# Install the binary
sudo cp target/release/driftless /usr/local/bin/
```

### Option 2: Download Pre-built Binary

```bash
# Download the latest release
wget https://github.com/driftless-hq/driftless/releases/latest/download/driftless-linux-x64.tar.gz
tar -xzf driftless-linux-x64.tar.gz
sudo mv driftless /usr/local/bin/
```

## Configuration

Create a configuration directory and files:

```bash
sudo mkdir -p /etc/driftless
sudo chown -R driftless:driftless /etc/driftless
```

### Agent Configuration (`/etc/driftless/agent.yml`)

```yaml
# Agent operational settings
apply_interval: 300          # Apply tasks every 5 minutes
facts_interval: 60           # Collect facts every minute
apply_dry_run: false         # Enable for production
metrics_port: 8000           # Prometheus metrics port
enabled: true                # Enable agent operation

# Resource limits
max_memory_mb: 512           # Memory limit
max_cpu_percent: 50          # CPU limit

# Circuit breaker settings
circuit_breaker_threshold: 5 # Failures before opening circuit
circuit_breaker_timeout: 300 # Seconds to wait before retry

# Logging
log_level: info
log_file: /var/log/driftless/agent.log
```

### Facts Configuration (`/etc/driftless/facts.yml`)

```yaml
collectors:
  - name: system
    enabled: true
    interval: 60
  - name: network
    enabled: true
    interval: 300
  - name: disk
    enabled: true
    interval: 60
```

### Apply Tasks Configuration (`/etc/driftless/apply.yml`)

```yaml
tasks:
  - name: ensure-ntp
    package:
      name: ntp
      state: present
  - name: configure-firewall
    ufw:
      state: enabled
      rules:
        - port: 22
          proto: tcp
```

### Logs Configuration (`/etc/driftless/logs.yml`)

```yaml
sources:
  - name: system-logs
    type: file
    path: /var/log/syslog
    parser: syslog

destinations:
  - name: elk-stack
    type: elasticsearch
    url: https://elk.example.com:9200
    index: driftless-%{+YYYY.MM.dd}
```

## Systemd Service Setup

Create the systemd service file:

```bash
sudo tee /etc/systemd/system/driftless-agent.service > /dev/null <<EOF
[Unit]
Description=Driftless Configuration Management Agent
After=network.target
Wants=network.target

[Service]
Type=simple
User=driftless
Group=driftless
ExecStart=/usr/local/bin/driftless --config /etc/driftless agent
Restart=always
RestartSec=10
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ReadWritePaths=/etc/driftless /var/log/driftless
ProtectHome=yes

# Resource limits
MemoryLimit=512M
CPUQuota=50%

[Install]
WantedBy=multi-user.target
EOF
```

Create the driftless user:

```bash
sudo useradd --system --shell /bin/false --home /var/lib/driftless --create-home driftless
sudo mkdir -p /var/log/driftless
sudo chown driftless:driftless /var/log/driftless
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable driftless-agent
sudo systemctl start driftless-agent
sudo systemctl status driftless-agent
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.92-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/driftless /usr/local/bin/driftless
USER nobody
ENTRYPOINT ["/usr/local/bin/driftless"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  driftless-agent:
    build: .
    volumes:
      - ./config:/etc/driftless:ro
      - ./logs:/var/log/driftless
    ports:
      - "8000:8000"
    restart: unless-stopped
    environment:
      - RUST_LOG=info
    command: ["--config", "/etc/driftless", "agent"]
```

## Kubernetes Deployment

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: driftless-agent
spec:
  replicas: 1
  selector:
    matchLabels:
      app: driftless-agent
  template:
    metadata:
      labels:
        app: driftless-agent
    spec:
      containers:
      - name: driftless-agent
        image: your-registry/driftless:latest
        args: ["--config", "/etc/driftless", "agent"]
        ports:
        - containerPort: 8000
          name: metrics
        volumeMounts:
        - name: config
          mountPath: /etc/driftless
          readOnly: true
        - name: logs
          mountPath: /var/log/driftless
        resources:
          limits:
            memory: 512Mi
            cpu: "500m"
          requests:
            memory: 256Mi
            cpu: "100m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: driftless-config
      - name: logs
        emptyDir: {}
```

### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: driftless-config
data:
  agent.yml: |
    apply_interval: 300
    facts_interval: 60
    apply_dry_run: false
    metrics_port: 8000
    enabled: true
  facts.yml: |
    collectors:
      - name: system
        enabled: true
  apply.yml: |
    tasks: []
  logs.yml: |
    sources: []
    destinations: []
```

## Monitoring Setup

### Prometheus Configuration

Add to your prometheus.yml:

```yaml
scrape_configs:
  - job_name: 'driftless-agent'
    static_configs:
      - targets: ['localhost:8000']
    scrape_interval: 30s
```

### Grafana Dashboard

Create panels for:

- Agent uptime and status
- Task execution success/failure rates
- Facts collection metrics
- Memory and CPU usage
- Circuit breaker status

## Security Considerations

1. **Run as non-root user**: Always use a dedicated user account
2. **Minimal permissions**: Only grant necessary file system access
3. **Network isolation**: Restrict network access as needed
4. **Configuration encryption**: Store sensitive config in secure locations
5. **Log rotation**: Configure logrotate for agent logs
6. **Updates**: Regularly update the agent binary for security patches

## Troubleshooting

See the [Operations Guide](operations.md) for detailed troubleshooting procedures.