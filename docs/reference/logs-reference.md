# Driftless Logs Reference

Comprehensive reference for all available log sources and outputs in Driftless.

This documentation is auto-generated from the Rust source code.

## Overview

Log processors handle log collection and forwarding. Each processor corresponds to a specific log source or output destination.

## Log Sources/Outputs (`logs`)

Log processors handle log collection and forwarding. Each processor corresponds to a specific log source or output destination.

### Processor Configuration

All log processors support common configuration fields for controlling processing behavior:

- **`enabled`**: Whether this processor is enabled (default: true)
- **`name`**: Processor name for identification

### Log Outputs

#### console

**Description**: Output logs to stdout/stderr for debugging

**Required Fields**:

- `name` (String):
  Processor name for identification

**Optional Fields**:

- `enabled` (bool):
  Whether this processor is enabled (default: true)

**Examples**:

**File log output**:

**YAML Format**:

```yaml
logs:
  - type: file
    path: /var/log/app.log
    format: json
    rotation:
      size: 10MB
      count: 5
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "file",
      "path": "/var/log/app.log",
      "format": "json",
      "rotation": {
        "size": "10MB",
        "count": 5
      }
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "file"
path = "/var/log/app.log"
format = "json"
[logs.rotation]
size = "10MB"
count = 5
```

**Console log output**:

**YAML Format**:

```yaml
logs:
  - type: console
    format: text
    level: info
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "console",
      "format": "text",
      "level": "info"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "console"
format = "text"
level = "info"
```

**Syslog log output**:

**YAML Format**:

```yaml
logs:
  - type: syslog
    facility: local0
    severity: info
    tag: driftless
    server: 127.0.0.1:514
    protocol: udp
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "syslog",
      "facility": "local0",
      "severity": "info",
      "tag": "driftless",
      "server": "127.0.0.1:514",
      "protocol": "udp"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "syslog"
facility = "local0"
severity = "info"
tag = "driftless"
server = "127.0.0.1:514"
protocol = "udp"
```

#### file

**Description**: Write logs to files with rotation and compression

**Required Fields**:

- `name` (String):
  Processor name for identification

**Optional Fields**:

- `enabled` (bool):
  Whether this processor is enabled (default: true)

**Examples**:

**File log output**:

**YAML Format**:

```yaml
logs:
  - type: file
    path: /var/log/app.log
    format: json
    rotation:
      size: 10MB
      count: 5
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "file",
      "path": "/var/log/app.log",
      "format": "json",
      "rotation": {
        "size": "10MB",
        "count": 5
      }
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "file"
path = "/var/log/app.log"
format = "json"
[logs.rotation]
size = "10MB"
count = 5
```

**Console log output**:

**YAML Format**:

```yaml
logs:
  - type: console
    format: text
    level: info
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "console",
      "format": "text",
      "level": "info"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "console"
format = "text"
level = "info"
```

**Syslog log output**:

**YAML Format**:

```yaml
logs:
  - type: syslog
    facility: local0
    severity: info
    tag: driftless
    server: 127.0.0.1:514
    protocol: udp
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "syslog",
      "facility": "local0",
      "severity": "info",
      "tag": "driftless",
      "server": "127.0.0.1:514",
      "protocol": "udp"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "syslog"
facility = "local0"
severity = "info"
tag = "driftless"
server = "127.0.0.1:514"
protocol = "udp"
```

#### http

**Description**: Send logs to HTTP endpoints with authentication and retry

**Required Fields**:

- `name` (String):
  Processor name for identification

**Optional Fields**:

- `enabled` (bool):
  Whether this processor is enabled (default: true)

**Examples**:

**File log output**:

**YAML Format**:

```yaml
logs:
  - type: file
    path: /var/log/app.log
    format: json
    rotation:
      size: 10MB
      count: 5
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "file",
      "path": "/var/log/app.log",
      "format": "json",
      "rotation": {
        "size": "10MB",
        "count": 5
      }
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "file"
path = "/var/log/app.log"
format = "json"
[logs.rotation]
size = "10MB"
count = 5
```

**Console log output**:

**YAML Format**:

```yaml
logs:
  - type: console
    format: text
    level: info
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "console",
      "format": "text",
      "level": "info"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "console"
format = "text"
level = "info"
```

**Syslog log output**:

**YAML Format**:

```yaml
logs:
  - type: syslog
    facility: local0
    severity: info
    tag: driftless
    server: 127.0.0.1:514
    protocol: udp
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "syslog",
      "facility": "local0",
      "severity": "info",
      "tag": "driftless",
      "server": "127.0.0.1:514",
      "protocol": "udp"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "syslog"
facility = "local0"
severity = "info"
tag = "driftless"
server = "127.0.0.1:514"
protocol = "udp"
```

#### s3

**Description**: Upload logs to S3 with batching and compression

**Required Fields**:

- `name` (String):
  Processor name for identification

**Optional Fields**:

- `enabled` (bool):
  Whether this processor is enabled (default: true)

**Examples**:

**File log output**:

**YAML Format**:

```yaml
logs:
  - type: file
    path: /var/log/app.log
    format: json
    rotation:
      size: 10MB
      count: 5
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "file",
      "path": "/var/log/app.log",
      "format": "json",
      "rotation": {
        "size": "10MB",
        "count": 5
      }
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "file"
path = "/var/log/app.log"
format = "json"
[logs.rotation]
size = "10MB"
count = 5
```

**Console log output**:

**YAML Format**:

```yaml
logs:
  - type: console
    format: text
    level: info
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "console",
      "format": "text",
      "level": "info"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "console"
format = "text"
level = "info"
```

**Syslog log output**:

**YAML Format**:

```yaml
logs:
  - type: syslog
    facility: local0
    severity: info
    tag: driftless
    server: 127.0.0.1:514
    protocol: udp
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "syslog",
      "facility": "local0",
      "severity": "info",
      "tag": "driftless",
      "server": "127.0.0.1:514",
      "protocol": "udp"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "syslog"
facility = "local0"
severity = "info"
tag = "driftless"
server = "127.0.0.1:514"
protocol = "udp"
```

#### syslog

**Description**: Send logs to syslog with RFC compliance

**Required Fields**:

- `name` (String):
  Processor name for identification

**Optional Fields**:

- `enabled` (bool):
  Whether this processor is enabled (default: true)

**Examples**:

**File log output**:

**YAML Format**:

```yaml
logs:
  - type: file
    path: /var/log/app.log
    format: json
    rotation:
      size: 10MB
      count: 5
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "file",
      "path": "/var/log/app.log",
      "format": "json",
      "rotation": {
        "size": "10MB",
        "count": 5
      }
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "file"
path = "/var/log/app.log"
format = "json"
[logs.rotation]
size = "10MB"
count = 5
```

**Console log output**:

**YAML Format**:

```yaml
logs:
  - type: console
    format: text
    level: info
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "console",
      "format": "text",
      "level": "info"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "console"
format = "text"
level = "info"
```

**Syslog log output**:

**YAML Format**:

```yaml
logs:
  - type: syslog
    facility: local0
    severity: info
    tag: driftless
    server: 127.0.0.1:514
    protocol: udp
```

**JSON Format**:

```json
{
  "logs": [
    {
      "type": "syslog",
      "facility": "local0",
      "severity": "info",
      "tag": "driftless",
      "server": "127.0.0.1:514",
      "protocol": "udp"
    }
  ]
}
```

**TOML Format**:

```toml
[[logs]]
type = "syslog"
facility = "local0"
severity = "info"
tag = "driftless"
server = "127.0.0.1:514"
protocol = "udp"
```

