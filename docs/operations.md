# Agent Operations Guide

This guide covers operational procedures for managing the Driftless agent in production.

## Service Management

### Checking Agent Status

```bash
# Systemd service status
sudo systemctl status driftless-agent

# Check if agent is responding
curl http://localhost:8000/health

# View agent logs
sudo journalctl -u driftless-agent -f
# or
tail -f /var/log/driftless/agent.log
```

### Starting/Stopping the Agent

```bash
# Start agent
sudo systemctl start driftless-agent

# Stop agent
sudo systemctl stop driftless-agent

# Restart agent
sudo systemctl restart driftless-agent

# Reload configuration (if supported)
sudo systemctl reload driftless-agent
```

### Manual Agent Execution

For testing or troubleshooting:

```bash
# Run agent in dry-run mode
driftless --config /etc/driftless agent --dry-run

# Run with custom intervals
driftless --config /etc/driftless agent --apply-interval 60 --facts-interval 30

# Run once and exit
driftless --config /etc/driftless agent --single-run
```

## Configuration Management

### Hot Configuration Reload

The agent supports hot reloading of configuration files:

```bash
# Edit configuration
sudo vi /etc/driftless/agent.yml

# The agent will automatically detect changes and reload
# Check logs for confirmation
sudo journalctl -u driftless-agent -n 20
```

### Configuration Validation

```bash
# Validate configuration syntax
driftless --config /etc/driftless agent --validate-config

# Test configuration with dry run
driftless --config /etc/driftless agent --dry-run --apply-interval 1
```

### Backup and Restore

```bash
# Backup configuration
sudo cp -r /etc/driftless /etc/driftless.backup.$(date +%Y%m%d)

# Restore configuration
sudo cp -r /etc/driftless.backup.20231201 /etc/driftless
sudo systemctl restart driftless-agent
```

## Monitoring and Metrics

### Prometheus Metrics

The agent exposes metrics at `http://localhost:8000/metrics`:

```bash
# Available metrics
curl http://localhost:8000/metrics

# Key metrics to monitor:
# - driftless_agent_uptime_seconds
# - driftless_tasks_executed_total
# - driftless_facts_collected_total
# - driftless_config_reload_total
# - driftless_circuit_breaker_state
# - driftless_memory_usage_bytes
# - driftless_cpu_usage_percent
```

### Health Checks

```bash
# Overall health
curl http://localhost:8000/health

# Readiness check
curl http://localhost:8000/ready

# Deep health check (includes subsystem status)
curl http://localhost:8000/health/deep
```

### Log Analysis

```bash
# Search for errors
grep "ERROR" /var/log/driftless/agent.log

# Check recent activity
tail -n 50 /var/log/driftless/agent.log

# Monitor task execution
grep "task.*executed" /var/log/driftless/agent.log | tail -10
```

## Troubleshooting

### Agent Won't Start

1. **Check configuration syntax:**
   ```bash
   driftless --config /etc/driftless agent --validate-config
   ```

2. **Check file permissions:**
   ```bash
   ls -la /etc/driftless/
   sudo chown -R driftless:driftless /etc/driftless/
   ```

3. **Check systemd logs:**
   ```bash
   sudo journalctl -u driftless-agent -n 50 --no-pager
   ```

4. **Test manual execution:**
   ```bash
   sudo -u driftless driftless --config /etc/driftless agent --dry-run
   ```

### Tasks Not Executing

1. **Check agent status:**
   ```bash
   curl http://localhost:8000/health
   ```

2. **Verify configuration:**
   ```bash
   cat /etc/driftless/apply.yml
   ```

3. **Check task execution logs:**
   ```bash
   grep "apply.*task" /var/log/driftless/agent.log
   ```

4. **Test task manually:**
   ```bash
   driftless --config /etc/driftless apply --dry-run
   ```

### High Resource Usage

1. **Check current metrics:**
   ```bash
   curl http://localhost:8000/metrics | grep -E "(memory|cpu)"
   ```

2. **Adjust resource limits:**
   ```yaml
   # In agent.yml
   max_memory_mb: 256
   max_cpu_percent: 25
   ```

3. **Reduce collection intervals:**
   ```yaml
   # In agent.yml
   apply_interval: 600  # 10 minutes
   facts_interval: 300  # 5 minutes
   ```

### Circuit Breaker Tripped

1. **Check circuit breaker status:**
   ```bash
   curl http://localhost:8000/metrics | grep circuit_breaker
   ```

2. **Review recent failures:**
   ```bash
   grep "circuit.*open" /var/log/driftless/agent.log
   ```

3. **Investigate root cause:**
   - Check network connectivity
   - Verify external service availability
   - Review task configurations

4. **Manual reset (if needed):**
   ```bash
   sudo systemctl restart driftless-agent
   ```

### Configuration Not Reloading

1. **Check file permissions:**
   ```bash
   ls -la /etc/driftless/
   ```

2. **Verify file watcher:**
   ```bash
   grep "config.*reload" /var/log/driftless/agent.log
   ```

3. **Manual reload:**
   ```bash
   sudo systemctl reload driftless-agent
   # or
   sudo systemctl restart driftless-agent
   ```

## Performance Tuning

### Memory Optimization

```yaml
# agent.yml
max_memory_mb: 256
circuit_breaker_threshold: 3
```

### CPU Optimization

```yaml
# agent.yml
max_cpu_percent: 25
apply_interval: 600
facts_interval: 300
```

### Network Optimization

```yaml
# agent.yml
# Reduce metrics collection frequency
metrics_interval: 60

# Configure timeouts
http_timeout: 30
```

## Log Management

### Log Rotation

Create `/etc/logrotate.d/driftless`:

```
/var/log/driftless/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 644 driftless driftless
    postrotate
        systemctl reload driftless-agent
    endscript
}
```

### Log Levels

Adjust log verbosity:

```yaml
# agent.yml
log_level: warn  # error, warn, info, debug, trace
```

Or via environment:

```bash
export RUST_LOG=driftless=debug
sudo systemctl restart driftless-agent
```

## Backup and Recovery

### Configuration Backup

```bash
#!/bin/bash
# Daily backup script
BACKUP_DIR="/var/backups/driftless"
mkdir -p $BACKUP_DIR
tar -czf $BACKUP_DIR/config-$(date +%Y%m%d).tar.gz -C /etc driftless
find $BACKUP_DIR -name "config-*.tar.gz" -mtime +30 -delete
```

### Full Recovery

```bash
# Stop agent
sudo systemctl stop driftless-agent

# Restore configuration
sudo tar -xzf /var/backups/driftless/config-20231201.tar.gz -C /etc

# Restore logs (if needed)
# sudo tar -xzf /var/backups/driftless/logs-20231201.tar.gz -C /var/log

# Start agent
sudo systemctl start driftless-agent
```

## Security Maintenance

### Regular Updates

```bash
# Check for updates
curl -s https://api.github.com/repos/your-org/driftless/releases/latest | grep "browser_download_url.*linux"

# Update binary
sudo systemctl stop driftless-agent
sudo cp new-driftless-binary /usr/local/bin/driftless
sudo systemctl start driftless-agent
```

### Security Audits

```bash
# Check running processes
ps aux | grep driftless

# Verify file permissions
find /etc/driftless -type f -exec ls -la {} \;

# Check network connections
ss -tlnp | grep :8000
```

## Emergency Procedures

### Emergency Stop

```bash
# Immediate stop
sudo systemctl stop driftless-agent

# Kill all processes
sudo pkill -9 driftless

# Disable service
sudo systemctl disable driftless-agent
```

### Emergency Recovery

```bash
# Restore from backup
sudo tar -xzf /var/backups/driftless/emergency-backup.tar.gz -C /

# Verify configuration
driftless --config /etc/driftless agent --validate-config

# Start in dry-run mode first
driftless --config /etc/driftless agent --dry-run

# Enable and start service
sudo systemctl enable driftless-agent
sudo systemctl start driftless-agent
```

## Support and Escalation

1. **Check documentation:** This operations guide and README.md
2. **Review logs:** Complete log analysis as described above
3. **Community support:** GitHub issues and discussions
4. **Commercial support:** Contact your support provider

For critical issues, gather:
- Agent version: `driftless --version`
- Configuration files (sanitized)
- Recent logs: `journalctl -u driftless-agent -n 100`
- System information: `uname -a`, `free -h`, `df -h`