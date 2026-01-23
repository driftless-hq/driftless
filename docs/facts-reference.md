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

