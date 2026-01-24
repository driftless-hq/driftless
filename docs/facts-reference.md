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

**Output**:

```yaml
 cpu_count: 4
 usage_percent: 45.2
 usage_warning: false
 usage_critical: false
 cores:
   - core_id: 0
     usage_percent: 42.1
     frequency_mhz: 2400
   - core_id: 1
     usage_percent: 48.3
     frequency_mhz: 2400
 frequency_mhz: 2400.0
 temperature_celsius: null
 temperature_available: false
 temp_warning: false
 temp_critical: false
 load_average:
   "1m": 1.25
   "5m": 1.15
   "15m": 1.08
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

**Output**:

```yaml
 command: "docker stats --no-stream --format json"
 exit_code: 0
 output:
   - container: "web_server"
     cpu_percent: "5.2"
     memory_usage: "128MiB / 1GiB"
     net_io: "1.2kB / 3.4kB"
   - container: "database"
     cpu_percent: "2.1"
     memory_usage: "256MiB / 2GiB"
     net_io: "500B / 1.2kB"
 labels:
   category: monitoring
```

**Key-value command output parsing**:

**YAML Format**:

```yaml
type: command
name: system_info
command: echo "hostname=$(hostname)\nos_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'=' -f2 | tr -d '\"')\nuptime=$(uptime -p)"
format: key_value
labels:
  category: system
```

**JSON Format**:

```json
{
  "type": "command",
  "name": "system_info",
  "command": "echo \"hostname=$(hostname)\\nos_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'=' -f2 | tr -d '\\\"')\\nuptime=$(uptime -p)\"",
  "format": "key_value",
  "labels": {
    "category": "system"
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "command"
name = "system_info"
command = "echo \"hostname=$(hostname)\\nos_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'=' -f2 | tr -d '\\\"')\\nuptime=$(uptime -p)\""
format = "key_value"
[collectors.labels]
category = "system"
```

**Output**:

```yaml
 command: "echo \"hostname=$(hostname)\\nos_version=$(cat /etc/os-release | grep PRETTY_NAME | cut -d'=' -f2 | tr -d '\\\"')\\nuptime=$(uptime -p)\""
 exit_code: 0
 output:
   hostname: "web-server-01"
   os_version: "Ubuntu 22.04.3 LTS"
   uptime: "up 2 weeks, 3 days, 4 hours"
 labels:
   category: system
```

**Text command output (default)**:

**YAML Format**:

```yaml
type: command
name: disk_usage
command: df -h /
format: text
labels:
  category: storage
```

**JSON Format**:

```json
{
  "type": "command",
  "name": "disk_usage",
  "command": "df -h /",
  "format": "text",
  "labels": {
    "category": "storage"
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "command"
name = "disk_usage"
command = "df -h /"
format = "text"
[collectors.labels]
category = "storage"
```

**Output**:

```yaml
 command: "df -h /"
 exit_code: 0
 stdout: |
   Filesystem      Size  Used Avail Use% Mounted on
   /dev/sda1        50G   15G   33G  31% /
 labels:
   category: storage
```

**Command with environment variables and working directory**:

**YAML Format**:

```yaml
type: command
name: custom_script
command: ./check_service.sh
format: json
cwd: /opt/myapp
env:
  SERVICE_NAME: myapp
  LOG_LEVEL: info
labels:
  category: application
```

**JSON Format**:

```json
{
  "type": "command",
  "name": "custom_script",
  "command": "./check_service.sh",
  "format": "json",
  "cwd": "/opt/myapp",
  "env": {
    "SERVICE_NAME": "myapp",
    "LOG_LEVEL": "info"
  },
  "labels": {
    "category": "application"
  }
}
```

**TOML Format**:

```toml
[[collectors]]
type = "command"
name = "custom_script"
command = "./check_service.sh"
format = "json"
cwd = "/opt/myapp"
[collectors.env]
SERVICE_NAME = "myapp"
LOG_LEVEL = "info"
[collectors.labels]
category = "application"
```

**Output**:

```yaml
 command: "./check_service.sh"
 exit_code: 0
 output:
   service_status: "running"
   uptime_seconds: 3600
   version: "1.2.3"
   health_checks:
     - name: "database"
       status: "ok"
     - name: "cache"
       status: "ok"
 labels:
   category: application
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

**Output**:

```yaml
 disks:
   - device: "/dev/sda1"
     mount_point: "/"
     is_removable: false
     total_bytes: 536870912000
     total_mb: 512000
     total_gb: 500
     used_bytes: 268435456000
     used_mb: 256000
     used_gb: 250
     free_bytes: 134217728000
     free_mb: 128000
     free_gb: 125
     available_bytes: 107374182400
     available_mb: 102400
     available_gb: 100
     usage_percent: 50
     available_percent: 20
     disk_pressure: "medium"
     usage_warning: false
     usage_critical: false
     io_supported: false
 labels:
   storage_type: ssd
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

**Output**:

```yaml
 total_bytes: 8589934592
 total_mb: 8192
 total_gb: 8
 used_bytes: 4294967296
 used_mb: 4096
 used_gb: 4
 free_bytes: 2147483648
 free_mb: 2048
 free_gb: 2
 available_bytes: 3221225472
 available_mb: 3072
 available_gb: 3
 usage_percent: 50
 available_percent: 37
 memory_pressure: "low"
 swap_total_bytes: 2147483648
 swap_used_bytes: 536870912
 swap_free_bytes: 1610612736
 swap_total_mb: 2048
 swap_used_mb: 512
 swap_free_mb: 1536
 swap_usage_percent: 25
 swap_pressure: "low"
 usage_warning: false
 usage_critical: false
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

**Output**:

```yaml
 interfaces:
   - name: "eth0"
     bytes_received: 1234567890
     bytes_transmitted: 987654321
     total_bytes: 2222222211
     packets_received: 1234567
     packets_transmitted: 987654
     total_packets: 2222221
     errors_on_received: 0
     errors_on_transmitted: 0
     total_errors: 0
     status: "up"
   - name: "lo"
     bytes_received: 123456789
     bytes_transmitted: 123456789
     total_bytes: 246913578
     packets_received: 123456
     packets_transmitted: 123456
     total_packets: 246912
     errors_on_received: 0
     errors_on_transmitted: 0
     total_errors: 0
     status: "up"
 labels:
   network_type: corporate
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

**Output**:

```yaml
 total_processes: 150
 matched_processes: 3
 processes:
   - pid: 1234
     name: "nginx"
     cpu_percent: 5
     memory_bytes: 104857600
     memory_mb: 100
     memory_gb: 0
     status: "running"
     command: "/usr/sbin/nginx"
     parent_pid: 1
   - pid: 1235
     name: "nginx"
     cpu_percent: 3
     memory_bytes: 52428800
     memory_mb: 50
     memory_gb: 0
     status: "running"
     command: "/usr/sbin/nginx"
     parent_pid: 1234
   - pid: 5678
     name: "apache2"
     cpu_percent: 2
     memory_bytes: 209715200
     memory_mb: 200
     memory_gb: 0
     status: "sleeping"
     command: "/usr/sbin/apache2"
     parent_pid: 1
 labels:
   process_type: web_servers
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

**Output**:

```yaml
 hostname: "myhost.example.com"
 os: "linux"
 os_family: "unix"
 kernel_version: "5.15.0-91-generic"
 uptime_seconds: 1234567
 boot_time: 1706012345
 cpu_arch: "x86_64"
```

