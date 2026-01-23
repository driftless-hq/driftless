# Driftless Facts Reference

Comprehensive reference for all available facts collectors in Driftless.

This documentation is auto-generated from the Rust source code.

## Overview

Facts collectors gather system metrics and inventory information. Each collector corresponds to a specific type of system information or metric.

## Facts Collectors (`facts`)

Facts collectors gather system metrics and inventory information. Each collector corresponds to a specific type of system information or metric.

### Collector Configuration

All facts collectors support common configuration fields for controlling collection behavior:

- **`name`**: Collector name (used for metric names)
- **`enabled`**: Whether this collector is enabled (default: true)
- **`poll_interval`**: Poll interval in seconds (how often to collect this metric)
- **`labels`**: Additional labels for this collector

### CPU Metrics

#### cpu

**Description**: Collect CPU usage, frequency, temperature, and load average metrics

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `collect` (CpuCollectOptions):
  CPU metrics to collect

- `name` (String):
  Collector name (used for metric names)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

- `thresholds` (CpuThresholds):
  Thresholds for alerts

**Optional Fields**:

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic CPU metrics collection**:

**YAML Format**:

```yaml
type: cpu
name: cpu
poll_interval: 30
collect:
  usage: true
  per_core: true
  frequency: true
  temperature: true
  load_average: true
thresholds:
  usage_warning: 80.0
  usage_critical: 95.0
  temp_warning: 70.0
  temp_critical: 85.0
```

**JSON Format**:

```json
{
  "type": "cpu",
  "name": "cpu",
  "poll_interval": 30,
  "collect": {
    "usage": true,
    "per_core": true,
    "frequency": true,
    "temperature": true,
    "load_average": true
  },
  "thresholds": {
    "usage_warning": 80.0,
    "usage_critical": 95.0,
    "temp_warning": 70.0,
    "temp_critical": 85.0
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "cpu"
name = "cpu"
poll_interval = 30
[collectors.collect]
usage = true
per_core = true
frequency = true
temperature = true
load_average = true
[collectors.thresholds]
usage_warning = 80.0
usage_critical = 95.0
temp_warning = 70.0
temp_critical = 85.0
```

### Command Output

#### command

**Description**: Execute custom commands and collect their output as facts

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `command` (String):
  Command to execute

- `env` (HashMap<String, String>):
  Environment variables

- `format` (CommandOutputFormat):
  Expected output format

- `name` (String):
  Collector name (used for metric names)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

**Optional Fields**:

- `cwd` (Option<String>):
  Working directory for command

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic command output collection**:

**YAML Format**:

```yaml
type: command
name: uptime
command: uptime -p
format: text
labels:
  category: system
```

**JSON Format**:

```json
{
  "type": "command",
  "name": "uptime",
  "command": "uptime -p",
  "format": "text",
  "labels": {
    "category": "system"
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "command"
name = "uptime"
command = "uptime -p"
format = "text"
[collectors.labels]
category = "system"
```

**JSON command output parsing**:

**YAML Format**:

```yaml
type: command
name: docker_stats
command: docker stats --no-stream --format json
format: json
cwd: /tmp
env:
  DOCKER_HOST: unix:///var/run/docker.sock
```

**JSON Format**:

```json
{
  "type": "command",
  "name": "docker_stats",
  "command": "docker stats --no-stream --format json",
  "format": "json",
  "cwd": "/tmp",
  "env": {
    "DOCKER_HOST": "unix:///var/run/docker.sock"
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "command"
name = "docker_stats"
command = "docker stats --no-stream --format json"
format = "json"
cwd = "/tmp"
[collectors.env]
DOCKER_HOST = "unix:///var/run/docker.sock"
```

### Disk Metrics

#### disk

**Description**: Collect disk space and I/O statistics for mounted filesystems

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `collect` (DiskCollectOptions):
  Disk metrics to collect

- `devices` (Vec<String>):
  Disk devices to monitor (empty = all)

- `mount_points` (Vec<String>):
  Mount points to monitor (empty = all)

- `name` (String):
  Collector name (used for metric names)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

- `thresholds` (DiskThresholds):
  Thresholds for alerts

**Optional Fields**:

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic disk metrics collection**:

**YAML Format**:

```yaml
type: disk
name: disk
devices: ["/dev/sda", "/dev/sdb"]
mount_points: ["/", "/home", "/var"]
collect:
  total: true
  used: true
  free: true
  available: true
  percentage: true
  io: true
thresholds:
  usage_warning: 80.0
  usage_critical: 90.0
```

**JSON Format**:

```json
{
  "type": "disk",
  "name": "disk",
  "devices": ["/dev/sda", "/dev/sdb"],
  "mount_points": ["/", "/home", "/var"],
  "collect": {
    "total": true,
    "used": true,
    "free": true,
    "available": true,
    "percentage": true,
    "io": true
  },
  "thresholds": {
    "usage_warning": 80.0,
    "usage_critical": 90.0
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "disk"
name = "disk"
devices = ["/dev/sda", "/dev/sdb"]
mount_points = ["/", "/home", "/var"]
[collectors.collect]
total = true
used = true
free = true
available = true
percentage = true
io = true
[collectors.thresholds]
usage_warning = 80.0
usage_critical = 90.0
```

### Memory Metrics

#### memory

**Description**: Collect memory usage statistics including total, used, free, and swap

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `collect` (MemoryCollectOptions):
  Memory metrics to collect

- `name` (String):
  Collector name (used for metric names)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

- `thresholds` (MemoryThresholds):
  Thresholds for alerts

**Optional Fields**:

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic memory metrics collection**:

**YAML Format**:

```yaml
type: memory
name: memory
collect:
  total: true
  used: true
  free: true
  available: true
  swap: true
  percentage: true
thresholds:
  usage_warning: 85.0
  usage_critical: 95.0
```

**JSON Format**:

```json
{
  "type": "memory",
  "name": "memory",
  "collect": {
    "total": true,
    "used": true,
    "free": true,
    "available": true,
    "swap": true,
    "percentage": true
  },
  "thresholds": {
    "usage_warning": 85.0,
    "usage_critical": 95.0
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "memory"
name = "memory"
[collectors.collect]
total = true
used = true
free = true
available = true
swap = true
percentage = true
[collectors.thresholds]
usage_warning = 85.0
usage_critical = 95.0
```

### Network Metrics

#### network

**Description**: Collect network interface statistics and status information

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `collect` (NetworkCollectOptions):
  Network metrics to collect

- `interfaces` (Vec<String>):
  Network interfaces to monitor (empty = all)

- `name` (String):
  Collector name (used for metric names)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

**Optional Fields**:

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic network metrics collection**:

**YAML Format**:

```yaml
type: network
name: network
interfaces: ["eth0", "wlan0"]
collect:
  bytes: true
  packets: true
  errors: true
  status: true
```

**JSON Format**:

```json
{
  "type": "network",
  "name": "network",
  "interfaces": ["eth0", "wlan0"],
  "collect": {
    "bytes": true,
    "packets": true,
    "errors": true,
    "status": true
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "network"
name = "network"
interfaces = ["eth0", "wlan0"]
[collectors.collect]
bytes = true
packets = true
errors = true
status = true
```

### Process Metrics

#### process

**Description**: Collect process information and resource usage statistics

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `collect` (ProcessCollectOptions):
  Process metrics to collect

- `name` (String):
  Collector name (used for metric names)

- `patterns` (Vec<String>):
  Process name patterns to monitor (empty = all processes)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

**Optional Fields**:

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic process metrics collection**:

**YAML Format**:

```yaml
type: process
name: process
patterns: ["nginx", "apache", "sshd"]
collect:
  count: true
  cpu: true
  memory: true
  status: true
```

**JSON Format**:

```json
{
  "type": "process",
  "name": "process",
  "patterns": ["nginx", "apache", "sshd"],
  "collect": {
    "count": true,
    "cpu": true,
    "memory": true,
    "status": true
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "process"
name = "process"
patterns = ["nginx", "apache", "sshd"]
[collectors.collect]
count = true
cpu = true
memory = true
status = true
```

### System Information

#### system

**Description**: Collect system information including hostname, OS, kernel, uptime, and architecture

**Required Fields**:

- `base` (BaseCollector):
  No description available

- `collect` (SystemCollectOptions):
  What system information to collect

- `name` (String):
  Collector name (used for metric names)

- `poll_interval` (u64):
  Poll interval in seconds (how often to collect this metric)

**Optional Fields**:

- `enabled` (bool):
  Whether this collector is enabled (default: true)

- `labels` (HashMap<String, String>):
  Additional labels for this collector

**Examples**:

**Basic system information collection**:

**YAML Format**:

```yaml
type: system
name: system
collect:
  hostname: true
  os: true
  kernel: true
  uptime: true
  boot_time: true
  arch: true
```

**JSON Format**:

```json
{
  "type": "system",
  "name": "system",
  "collect": {
    "hostname": true,
    "os": true,
    "kernel": true,
    "uptime": true,
    "boot_time": true,
    "arch": true
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "system"
name = "system"
[collectors.collect]
hostname = true
os = true
kernel = true
uptime = true
boot_time = true
arch = true
```

