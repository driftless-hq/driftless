# Driftless Configuration Reference

Comprehensive reference for all available configuration components in Driftless.

This documentation is auto-generated from the Rust source code.

## Overview

Driftless provides three main configuration components that work together to manage systems:

- **Configuration Operations** (`apply`): Define and enforce desired system state
- **Facts Collectors** (`facts`): Gather system metrics and inventory information
- **Log Sources/Outputs** (`logs`): Handle log collection and forwarding

## Configuration Operations (`apply`)

Configuration operations define desired system state and are executed idempotently. Each operation corresponds to a specific aspect of system configuration management.

### Task Result Registration and Conditions

All configuration operations support special fields for conditional execution and capturing results:

- **`when`**: An optional expression (usually containing variables) that determines if the task should be executed. If the condition evaluates to `false`, the task is skipped.
- **`register`**: An optional variable name to capture the result of the task execution. The captured data varies by task type and can be used in subsequent tasks using template expansion (e.g., `{{ my_var.stdout }}`). This field only appears in the documentation for tasks that provide output results.

### Command Execution

#### command

**Description**: Command execution task

**Required Fields**:

- `command` (String):
  Command to execute

- `env` (HashMap<String, String>):
  Environment variables

- `exit_code` (i32):
  Expected exit code (default: 0)

- `idempotent` (bool):
  Whether command should be idempotent (only run if not already applied)

**Optional Fields**:

- `cwd` (Option<String>):
  Working directory for command execution

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `group` (Option<String>):
  Whether to run command as a specific group

- `register` (Option<String>):
  Optional variable name to register the task result in

- `user` (Option<String>):
  Whether to run command as a specific user

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Registered Outputs**:

- `changed` (bool): Whether the command was actually run
- `rc` (i32): The exit code of the command
- `stderr` (String): The standard error of the command
- `stdout` (String): The standard output of the command

**Examples**:

**Run a simple command**:

**YAML Format**:

```yaml
- type: command
  description: "Update package list"
  command: apt-get update
```

**JSON Format**:

```json
{
  "type": "command",
  "description": "Update package list",
  "command": "apt-get update"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "command"
description = "Update package list"
command = "apt-get update"
```

**Run command with specific working directory**:

**YAML Format**:

```yaml
- type: command
  description: "Build application in project directory"
  command: make build
  cwd: /opt/myapp
```

**JSON Format**:

```json
{
  "type": "command",
  "description": "Build application in project directory",
  "command": "make build",
  "cwd": "/opt/myapp"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "command"
description = "Build application in project directory"
command = "make build"
cwd = "/opt/myapp"
```

**Run command as specific user**:

**YAML Format**:

```yaml
- type: command
  description: "Restart nginx service"
  command: systemctl restart nginx
  user: root
```

**JSON Format**:

```json
{
  "type": "command",
  "description": "Restart nginx service",
  "command": "systemctl restart nginx",
  "user": "root"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "command"
description = "Restart nginx service"
command = "systemctl restart nginx"
user = "root"
```

**Register command output**:

**YAML Format**:

```yaml
- type: command
  description: "Check system uptime"
  command: uptime
  register: uptime_result
- type: debug
  msg: "The system uptime is: {{ uptime_result.stdout }}"
```

**JSON Format**:

```json
[
  {
    "type": "command",
    "description": "Check system uptime",
    "command": "uptime",
    "register": "uptime_result"
  },
  {
    "type": "debug",
    "msg": "The system uptime is: {{ uptime_result.stdout }}"
  }
]
```

**TOML Format**:

```toml
[[tasks]]
type = "command"
description = "Check system uptime"
command = "uptime"
register = "uptime_result"
[[tasks]]
type = "debug"
msg = "The system uptime is: {{ uptime_result.stdout }}"
```

**Idempotent command**:

**YAML Format**:

```yaml
- type: command
  description: "Initialize database (idempotent)"
  command: /opt/myapp/init-db.sh
  idempotent: true
  exit_code: 0
```

**JSON Format**:

```json
{
  "type": "command",
  "description": "Initialize database (idempotent)",
  "command": "/opt/myapp/init-db.sh",
  "idempotent": true,
  "exit_code": 0
}
```

**TOML Format**:

```toml
[[tasks]]
type = "command"
description = "Initialize database (idempotent)"
command = "/opt/myapp/init-db.sh"
idempotent = true
exit_code = 0
```

#### raw

**Description**: Execute commands without shell processing task

**Required Fields**:

- `args` (Vec<String>):
  Command arguments (argv[1..])

- `creates` (bool):
  Whether the command creates resources

- `environment` (HashMap<String, String>):
  Environment variables

- `executable` (String):
  Command to execute (argv\[0\])

- `exit_codes` (Vec<i32>):
  Expected exit codes (defaults to \[0\])

- `force` (bool):
  Force command execution

- `ignore_errors` (bool):
  Whether to ignore errors

- `removes` (bool):
  Whether the command removes resources

**Optional Fields**:

- `chdir` (Option<String>):
  Working directory for command execution

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `timeout` (Option<u32>):
  Execution timeout in seconds

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Execute a simple command**:

**YAML Format**:

```yaml
- type: raw
  description: "List directory contents"
  executable: ls
  args: ["-la", "/tmp"]
```

**JSON Format**:

```json
{
  "type": "raw",
  "description": "List directory contents",
  "executable": "ls",
  "args": ["-la", "/tmp"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "raw"
description = "List directory contents"
executable = "ls"
args = ["-la", "/tmp"]
```

**Execute command with environment variables**:

**YAML Format**:

```yaml
- type: raw
  description: "Run command with environment"
  executable: /usr/local/bin/myapp
  args: ["--config", "/etc/myapp/config.json"]
  environment:
    DATABASE_URL: "postgresql://localhost/mydb"
    LOG_LEVEL: "debug"
```

**JSON Format**:

```json
{
  "type": "raw",
  "description": "Run command with environment",
  "executable": "/usr/local/bin/myapp",
  "args": ["--config", "/etc/myapp/config.json"],
  "environment": {
    "DATABASE_URL": "postgresql://localhost/mydb",
    "LOG_LEVEL": "debug"
  }
}
```

**TOML Format**:

```toml
[[tasks]]
type = "raw"
description = "Run command with environment"
executable = "/usr/local/bin/myapp"
args = ["--config", "/etc/myapp/config.json"]
environment = { DATABASE_URL = "postgresql://localhost/mydb", LOG_LEVEL = "debug" }
```

**Execute command with timeout**:

**YAML Format**:

```yaml
- type: raw
  description: "Run command with timeout"
  executable: sleep
  args: ["30"]
  timeout: 10
  ignore_errors: true
```

**JSON Format**:

```json
{
  "type": "raw",
  "description": "Run command with timeout",
  "executable": "sleep",
  "args": ["30"],
  "timeout": 10,
  "ignore_errors": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "raw"
description = "Run command with timeout"
executable = "sleep"
args = ["30"]
timeout = 10
ignore_errors = true
```

**Execute command in specific directory**:

**YAML Format**:

```yaml
- type: raw
  description: "Run command in project directory"
  executable: make
  args: ["build"]
  chdir: /opt/myproject
```

**JSON Format**:

```json
{
  "type": "raw",
  "description": "Run command in project directory",
  "executable": "make",
  "args": ["build"],
  "chdir": "/opt/myproject"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "raw"
description = "Run command in project directory"
executable = "make"
args = ["build"]
chdir = "/opt/myproject"
```

#### script

**Description**: Execute local scripts task

**Required Fields**:

- `creates` (bool):
  Whether the script creates resources

- `environment` (HashMap<String, String>):
  Environment variables

- `force` (bool):
  Force script execution

- `params` (Vec<String>):
  Script parameters/arguments

- `path` (String):
  Path to the script file

- `removes` (bool):
  Whether the script removes resources

**Optional Fields**:

- `chdir` (Option<String>):
  Working directory for script execution

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `timeout` (Option<u32>):
  Execution timeout in seconds

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Execute a script**:

**YAML Format**:

```yaml
- type: script
  description: "Run setup script"
  path: /usr/local/bin/setup.sh
```

**JSON Format**:

```json
{
  "type": "script",
  "description": "Run setup script",
  "path": "/usr/local/bin/setup.sh"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "script"
description = "Run setup script"
path = "/usr/local/bin/setup.sh"
```

**Execute script with parameters**:

**YAML Format**:

```yaml
- type: script
  description: "Run deployment script with environment"
  path: /opt/deploy/deploy.sh
  params: ["production", "--verbose"]
  chdir: /opt/deploy
```

**JSON Format**:

```json
{
  "type": "script",
  "description": "Run deployment script with environment",
  "path": "/opt/deploy/deploy.sh",
  "params": ["production", "--verbose"],
  "chdir": "/opt/deploy"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "script"
description = "Run deployment script with environment"
path = "/opt/deploy/deploy.sh"
params = ["production", "--verbose"]
chdir = "/opt/deploy"
```

**Execute script with environment variables**:

**YAML Format**:

```yaml
- type: script
  description: "Run script with environment"
  path: /usr/local/bin/configure.sh
  environment:
    DATABASE_URL: "postgresql://localhost/mydb"
    API_KEY: "secret-key"
  timeout: 300
```

**JSON Format**:

```json
{
  "type": "script",
  "description": "Run script with environment",
  "path": "/usr/local/bin/configure.sh",
  "environment": {
    "DATABASE_URL": "postgresql://localhost/mydb",
    "API_KEY": "secret-key"
  },
  "timeout": 300
}
```

**TOML Format**:

```toml
[[tasks]]
type = "script"
description = "Run script with environment"
path = "/usr/local/bin/configure.sh"
environment = { DATABASE_URL = "postgresql://localhost/mydb", API_KEY = "secret-key" }
timeout = 300
```

**Execute script with creates/removes checks**:

**YAML Format**:

```yaml
- type: script
  description: "Run initialization script"
  path: /usr/local/bin/init.sh
  creates: true
  timeout: 600
```

**JSON Format**:

```json
{
  "type": "script",
  "description": "Run initialization script",
  "path": "/usr/local/bin/init.sh",
  "creates": true,
  "timeout": 600
}
```

**TOML Format**:

```toml
[[tasks]]
type = "script"
description = "Run initialization script"
path = "/usr/local/bin/init.sh"
creates = true
timeout = 600
```

### File Operations

#### archive

**Description**: Archive files task

**Required Fields**:

- `compression` (u32):
  Compression level (1-9)

- `extra_opts` (Vec<String>):
  Extra options for archiving

- `format` (ArchiveFormat):
  Archive format

- `path` (String):
  Archive file path

- `sources` (Vec<String>):
  Files/directories to archive

- `state` (ArchiveState):
  Archive state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `dest` (Option<String>):
  Destination directory (for extraction)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a tar archive**:

**YAML Format**:

```yaml
- type: archive
  description: "Create backup archive"
  path: /tmp/backup.tar
  state: present
  format: tar
  sources:
    - /home/user/documents
    - /home/user/pictures
```

**JSON Format**:

```json
{
  "type": "archive",
  "description": "Create backup archive",
  "path": "/tmp/backup.tar",
  "state": "present",
  "format": "tar",
  "sources": ["/home/user/documents", "/home/user/pictures"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "archive"
description = "Create backup archive"
path = "/tmp/backup.tar"
state = "present"
format = "tar"
sources = ["/home/user/documents", "/home/user/pictures"]
```

**Create a compressed tar archive**:

**YAML Format**:

```yaml
- type: archive
  description: "Create compressed backup"
  path: /tmp/backup.tar.gz
  state: present
  format: tgz
  sources:
    - /var/log
  compression: 9
```

**JSON Format**:

```json
{
  "type": "archive",
  "description": "Create compressed backup",
  "path": "/tmp/backup.tar.gz",
  "state": "present",
  "format": "tgz",
  "sources": ["/var/log"],
  "compression": 9
}
```

**TOML Format**:

```toml
[[tasks]]
type = "archive"
description = "Create compressed backup"
path = "/tmp/backup.tar.gz"
state = "present"
format = "tgz"
sources = ["/var/log"]
compression = 9
```

**Create a zip archive**:

**YAML Format**:

```yaml
- type: archive
  description: "Create zip archive"
  path: /tmp/data.zip
  state: present
  format: zip
  sources:
    - /home/user/data
```

**JSON Format**:

```json
{
  "type": "archive",
  "description": "Create zip archive",
  "path": "/tmp/data.zip",
  "state": "present",
  "format": "zip",
  "sources": ["/home/user/data"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "archive"
description = "Create zip archive"
path = "/tmp/data.zip"
state = "present"
format = "zip"
sources = ["/home/user/data"]
```

**Remove an archive**:

**YAML Format**:

```yaml
- type: archive
  description: "Remove old backup"
  path: /tmp/old-backup.tar.gz
  state: absent
```

**JSON Format**:

```json
{
  "type": "archive",
  "description": "Remove old backup",
  "path": "/tmp/old-backup.tar.gz",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "archive"
description = "Remove old backup"
path = "/tmp/old-backup.tar.gz"
state = "absent"
```

#### blockinfile

**Description**: Insert/update multi-line blocks task

**Required Fields**:

- `backup` (bool):
  Backup file before modification

- `block` (String):
  Block content (multi-line)

- `create` (bool):
  Create file if it doesn't exist

- `marker` (String):
  Marker for block boundaries

- `path` (String):
  Path to the file

- `state` (BlockInFileState):
  Block state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `insertafter` (Option<String>):
  Insert after this line (regex)

- `insertbefore` (Option<String>):
  Insert before this line (regex)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Insert a configuration block**:

**YAML Format**:

```yaml
- type: blockinfile
  description: "Add custom configuration block"
  path: /etc/myapp/config.conf
  state: present
  block: |
    # Custom configuration
    custom_option = true
    custom_value = 42
  marker: "# {mark} Custom Config"
```

**JSON Format**:

```json
{
  "type": "blockinfile",
  "description": "Add custom configuration block",
  "path": "/etc/myapp/config.conf",
  "state": "present",
  "block": "# Custom configuration\ncustom_option = true\ncustom_value = 42\n",
  "marker": "# {mark} Custom Config"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "blockinfile"
description = "Add custom configuration block"
path = "/etc/myapp/config.conf"
state = "present"
block = """
# Custom configuration
custom_option = true
custom_value = 42
"""
marker = "# {mark} Custom Config"
```

**Insert block after specific content**:

**YAML Format**:

```yaml
- type: blockinfile
  description: "Add SSL configuration"
  path: /etc/httpd/httpd.conf
  state: present
  block: |
    SSLEngine on
    SSLCertificateFile /etc/ssl/certs/server.crt
    SSLCertificateKeyFile /etc/ssl/private/server.key
  insertafter: "^# LoadModule ssl_module"
  marker: "# {mark} SSL Config"
```

**JSON Format**:

```json
{
  "type": "blockinfile",
  "description": "Add SSL configuration",
  "path": "/etc/httpd/httpd.conf",
  "state": "present",
  "block": "SSLEngine on\nSSLCertificateFile /etc/ssl/certs/server.crt\nSSLCertificateKeyFile /etc/ssl/private/server.key\n",
  "insertafter": "^# LoadModule ssl_module",
  "marker": "# {mark} SSL Config"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "blockinfile"
description = "Add SSL configuration"
path = "/etc/httpd/httpd.conf"
state = "present"
block = """
SSLEngine on
SSLCertificateFile /etc/ssl/certs/server.crt
SSLCertificateKeyFile /etc/ssl/private/server.key
"""
insertafter = "^# LoadModule ssl_module"
marker = "# {mark} SSL Config"
```

**Insert block with backup**:

**YAML Format**:

```yaml
- type: blockinfile
  description: "Add firewall rules with backup"
  path: /etc/iptables/rules.v4
  state: present
  block: |
    -A INPUT -p tcp --dport 80 -j ACCEPT
    -A INPUT -p tcp --dport 443 -j ACCEPT
  marker: "# {mark} Web Rules"
  backup: true
```

**JSON Format**:

```json
{
  "type": "blockinfile",
  "description": "Add firewall rules with backup",
  "path": "/etc/iptables/rules.v4",
  "state": "present",
  "block": "-A INPUT -p tcp --dport 80 -j ACCEPT\n-A INPUT -p tcp --dport 443 -j ACCEPT\n",
  "marker": "# {mark} Web Rules",
  "backup": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "blockinfile"
description = "Add firewall rules with backup"
path = "/etc/iptables/rules.v4"
state = "present"
block = """
-A INPUT -p tcp --dport 80 -j ACCEPT
-A INPUT -p tcp --dport 443 -j ACCEPT
"""
marker = "# {mark} Web Rules"
backup = true
```

**Remove a configuration block**:

**YAML Format**:

```yaml
- type: blockinfile
  description: "Remove old configuration"
  path: /etc/myapp/config.conf
  state: absent
  marker: "# {mark} Old Config"
```

**JSON Format**:

```json
{
  "type": "blockinfile",
  "description": "Remove old configuration",
  "path": "/etc/myapp/config.conf",
  "state": "absent",
  "marker": "# {mark} Old Config"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "blockinfile"
description = "Remove old configuration"
path = "/etc/myapp/config.conf"
state = "absent"
marker = "# {mark} Old Config"
```

#### copy

**Description**: Copy files task

**Required Fields**:

- `backup` (bool):
  Whether to create backup of destination

- `dest` (String):
  Destination file path

- `follow` (bool):
  Whether to follow symlinks

- `force` (bool):
  Force copy even if destination exists

- `mode` (bool):
  Whether to preserve permissions

- `owner` (bool):
  Whether to preserve ownership

- `src` (String):
  Source file path

- `state` (CopyState):
  Copy state

- `timestamp` (bool):
  Whether to preserve timestamps

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Copy a file**:

**YAML Format**:

```yaml
- type: copy
  description: "Copy configuration file"
  src: /etc/nginx/nginx.conf.template
  dest: /etc/nginx/nginx.conf
  state: present
```

**JSON Format**:

```json
{
  "type": "copy",
  "description": "Copy configuration file",
  "src": "/etc/nginx/nginx.conf.template",
  "dest": "/etc/nginx/nginx.conf",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "copy"
description = "Copy configuration file"
src = "/etc/nginx/nginx.conf.template"
dest = "/etc/nginx/nginx.conf"
state = "present"
```

**Copy with backup**:

**YAML Format**:

```yaml
- type: copy
  description: "Copy config with backup"
  src: /tmp/new-config.conf
  dest: /etc/myapp/config.conf
  state: present
  backup: true
```

**JSON Format**:

```json
{
  "type": "copy",
  "description": "Copy config with backup",
  "src": "/tmp/new-config.conf",
  "dest": "/etc/myapp/config.conf",
  "state": "present",
  "backup": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "copy"
description = "Copy config with backup"
src = "/tmp/new-config.conf"
dest = "/etc/myapp/config.conf"
state = "present"
backup = true
```

**Remove a copied file**:

**YAML Format**:

```yaml
- type: copy
  description: "Remove copied configuration"
  src: /etc/nginx/nginx.conf.template
  dest: /etc/nginx/nginx.conf
  state: absent
```

**JSON Format**:

```json
{
  "type": "copy",
  "description": "Remove copied configuration",
  "src": "/etc/nginx/nginx.conf.template",
  "dest": "/etc/nginx/nginx.conf",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "copy"
description = "Remove copied configuration"
src = "/etc/nginx/nginx.conf.template"
dest = "/etc/nginx/nginx.conf"
state = "absent"
```

#### directory

**Description**: Directory management task

**Required Fields**:

- `parents` (bool):
  Whether to create parent directories

- `path` (String):
  Directory path

- `recurse` (bool):
  Whether to recursively set permissions

- `state` (DirectoryState):
  Directory state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `group` (Option<String>):
  Directory group

- `mode` (Option<String>):
  Directory permissions (octal string like "0755")

- `owner` (Option<String>):
  Directory owner

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a directory**:

**YAML Format**:

```yaml
- type: directory
  description: "Create application directory"
  path: /opt/myapp
  state: present
  mode: "0755"
  owner: root
  group: root
```

**JSON Format**:

```json
{
  "type": "directory",
  "description": "Create application directory",
  "path": "/opt/myapp",
  "state": "present",
  "mode": "0755",
  "owner": "root",
  "group": "root"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "directory"
description = "Create application directory"
path = "/opt/myapp"
state = "present"
mode = "0755"
owner = "root"
group = "root"
```

**Create directory with parent directories**:

**YAML Format**:

```yaml
- type: directory
  description: "Create nested directory structure"
  path: /var/log/myapp/subdir
  state: present
  mode: "0750"
  owner: myapp
  group: myapp
  parents: true
```

**JSON Format**:

```json
{
  "type": "directory",
  "description": "Create nested directory structure",
  "path": "/var/log/myapp/subdir",
  "state": "present",
  "mode": "0750",
  "owner": "myapp",
  "group": "myapp",
  "parents": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "directory"
description = "Create nested directory structure"
path = "/var/log/myapp/subdir"
state = "present"
mode = "0750"
owner = "myapp"
group = "myapp"
parents = true
```

**Remove a directory**:

**YAML Format**:

```yaml
- type: directory
  description: "Remove temporary directory"
  path: /tmp/old-data
  state: absent
```

**JSON Format**:

```json
{
  "type": "directory",
  "description": "Remove temporary directory",
  "path": "/tmp/old-data",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "directory"
description = "Remove temporary directory"
path = "/tmp/old-data"
state = "absent"
```

#### fetch

**Description**: Fetch files from remote hosts task

**Required Fields**:

- `dest` (String):
  Destination file path

- `follow_redirects` (bool):
  Follow redirects

- `force` (bool):
  Force download even if file exists

- `headers` (HashMap<String, String>):
  HTTP headers

- `state` (FetchState):
  Fetch state

- `timeout` (u64):
  Timeout in seconds

- `url` (String):
  Source URL

- `validate_certs` (bool):
  Validate SSL certificates

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `password` (Option<String>):
  Password for basic auth

- `username` (Option<String>):
  Username for basic auth

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Download a file**:

**YAML Format**:

```yaml
- type: fetch
  description: "Download configuration file"
  url: http://example.com/config.yml
  dest: /etc/myapp/config.yml
  state: present
```

**JSON Format**:

```json
{
  "type": "fetch",
  "description": "Download configuration file",
  "url": "http://example.com/config.yml",
  "dest": "/etc/myapp/config.yml",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "fetch"
description = "Download configuration file"
url = "http://example.com/config.yml"
dest = "/etc/myapp/config.yml"
state = "present"
```

**Download with authentication**:

**YAML Format**:

```yaml
- type: fetch
  description: "Download private file"
  url: https://private.example.com/file.txt
  dest: /tmp/private.txt
  state: present
  username: myuser
  password: mypassword
```

**JSON Format**:

```json
{
  "type": "fetch",
  "description": "Download private file",
  "url": "https://private.example.com/file.txt",
  "dest": "/tmp/private.txt",
  "state": "present",
  "username": "myuser",
  "password": "mypassword"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "fetch"
description = "Download private file"
url = "https://private.example.com/file.txt"
dest = "/tmp/private.txt"
state = "present"
username = "myuser"
password = "mypassword"
```

**Download with custom headers**:

**YAML Format**:

```yaml
- type: fetch
  description: "Download with custom headers"
  url: https://api.example.com/data.json
  dest: /tmp/data.json
  state: present
  headers:
    Authorization: "Bearer token123"
    X-API-Key: "apikey456"
```

**JSON Format**:

```json
{
  "type": "fetch",
  "description": "Download with custom headers",
  "url": "https://api.example.com/data.json",
  "dest": "/tmp/data.json",
  "state": "present",
  "headers": {
    "Authorization": "Bearer token123",
    "X-API-Key": "apikey456"
  }
}
```

**TOML Format**:

```toml
[[tasks]]
type = "fetch"
description = "Download with custom headers"
url = "https://api.example.com/data.json"
dest = "/tmp/data.json"
state = "present"
headers = { Authorization = "Bearer token123", "X-API-Key" = "apikey456" }
```

**Force download**:

**YAML Format**:

```yaml
- type: fetch
  description: "Force download latest version"
  url: https://example.com/latest.tar.gz
  dest: /tmp/latest.tar.gz
  state: present
  force: true
```

**JSON Format**:

```json
{
  "type": "fetch",
  "description": "Force download latest version",
  "url": "https://example.com/latest.tar.gz",
  "dest": "/tmp/latest.tar.gz",
  "state": "present",
  "force": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "fetch"
description = "Force download latest version"
url = "https://example.com/latest.tar.gz"
dest = "/tmp/latest.tar.gz"
state = "present"
force = true
```

#### file

**Description**: File operation task

Manages files and directories - create, modify, or remove files with content,
permissions, and ownership. Similar to Ansible's `file` module.

**Required Fields**:

- `path` (String):
  Path to the file or directory

  Absolute path to the file or directory to manage.
  Parent directories will not be created automatically.

- `state` (FileState):
  File state (present, absent)

  - `present`: Ensure the file exists with specified properties
  - `absent`: Ensure the file does not exist

**Optional Fields**:

- `content` (Option<String>):
  File content (for present state)

  Content to write to the file when state is `present`.
  Mutually exclusive with `source`.

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `group` (Option<String>):
  File group name

  Group name for the file. Only applied when creating or modifying files.

- `mode` (Option<String>):
  File permissions (octal string like "0644")

  File permissions in octal notation (e.g., "0644", "0755").
  Only applied when creating or modifying files.

- `owner` (Option<String>):
  File owner username

  Username of the file owner. Only applied when creating or modifying files.

- `source` (Option<String>):
  Source file to copy from (alternative to content)

  Path to a source file to copy content from when state is `present`.
  Mutually exclusive with `content`.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a file with content**:

**YAML Format**:

```yaml
- type: file
  description: "Create nginx configuration file"
  path: /etc/nginx/sites-available/default
  state: present
  content: |
    server {
        listen 80;
        root /var/www/html;
        index index.html index.htm;
        location / {
            try_files $uri $uri/ =404;
        }
    }
  mode: "0644"
  owner: root
  group: root
```

**JSON Format**:

```json
{
  "type": "file",
  "description": "Create nginx configuration file",
  "path": "/etc/nginx/sites-available/default",
  "state": "present",
  "content": "server {\n    listen 80;\n    root /var/www/html;\n    index index.html index.htm;\n\n    location / {\n        try_files $uri $uri/ =404;\n    }\n}",
  "mode": "0644",
  "owner": "root",
  "group": "root"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "file"
description = "Create nginx configuration file"
path = "/etc/nginx/sites-available/default"
state = "present"
content = """
server {
    listen 80;
    root /var/www/html;
    index index.html index.htm;
    location / {
        try_files $uri $uri/ =404;
    }
}
"""
mode = "0644"
owner = "root"
group = "root"
```

**Register file creation**:

**YAML Format**:

```yaml
- type: file
  description: "Create marker file"
  path: /tmp/driftless.marker
  state: present
  register: marker_file
```

**JSON Format**:

```json
{
  "type": "file",
  "description": "Create marker file",
  "path": "/tmp/driftless.marker",
  "state": "present",
  "register": "marker_file"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "file"
description = "Create marker file"
path = "/tmp/driftless.marker"
state = "present"
register = "marker_file"
```

#### lineinfile

**Description**: Ensure line in file task

**Required Fields**:

- `backup` (bool):
  Backup file before modification

- `create` (bool):
  Create file if it doesn't exist

- `line` (String):
  The line content

- `path` (String):
  Path to the file

- `state` (LineInFileState):
  Line state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `insertafter` (Option<String>):
  Insert after this line (regex)

- `insertbefore` (Option<String>):
  Insert before this line (regex)

- `regexp` (Option<String>):
  Regular expression to match existing line

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Add a line to a file**:

**YAML Format**:

```yaml
- type: lineinfile
  description: "Add localhost entry to hosts file"
  path: /etc/hosts
  state: present
  line: "127.0.0.1 localhost"
```

**JSON Format**:

```json
{
  "type": "lineinfile",
  "description": "Add localhost entry to hosts file",
  "path": "/etc/hosts",
  "state": "present",
  "line": "127.0.0.1 localhost"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "lineinfile"
description = "Add localhost entry to hosts file"
path = "/etc/hosts"
state = "present"
line = "127.0.0.1 localhost"
```

**Replace a line using regex**:

**YAML Format**:

```yaml
- type: lineinfile
  description: "Update SSH port configuration"
  path: /etc/ssh/sshd_config
  state: present
  regexp: "^#?Port .*"
  line: "Port 22"
```

**JSON Format**:

```json
{
  "type": "lineinfile",
  "description": "Update SSH port configuration",
  "path": "/etc/ssh/sshd_config",
  "state": "present",
  "regexp": "^#?Port .*",
  "line": "Port 22"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "lineinfile"
description = "Update SSH port configuration"
path = "/etc/ssh/sshd_config"
state = "present"
regexp = "^#?Port .*"
line = "Port 22"
```

**Insert line after a pattern**:

**YAML Format**:

```yaml
- type: lineinfile
  description: "Add include directive after main config"
  path: /etc/nginx/nginx.conf
  state: present
  line: "include /etc/nginx/sites-enabled/*;"
  insertafter: "http \{"
```

**JSON Format**:

```json
{
  "type": "lineinfile",
  "description": "Add include directive after main config",
  "path": "/etc/nginx/nginx.conf",
  "state": "present",
  "line": "include /etc/nginx/sites-enabled/*;",
  "insertafter": "http \{"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "lineinfile"
description = "Add include directive after main config"
path = "/etc/nginx/nginx.conf"
state = "present"
line = "include /etc/nginx/sites-enabled/*;"
insertafter = "http \{"
```

#### replace

**Description**: Replace text in files task

**Required Fields**:

- `backup` (bool):
  Backup file before modification

- `path` (String):
  Path to the file

- `replace` (String):
  Replacement string

- `replace_all` (bool):
  Replace all occurrences

- `state` (ReplaceState):
  Replace state

**Optional Fields**:

- `before` (Option<String>):
  String to match (alternative to regexp)

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `regexp` (Option<String>):
  Regular expression to match

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Replace text using regex**:

**YAML Format**:

```yaml
- type: replace
  description: "Update database host"
  path: /etc/myapp/config.ini
  state: present
  regexp: '^db_host\s*=\s*.*$'
  replace: 'db_host = newdb.example.com'
```

**JSON Format**:

```json
{
  "type": "replace",
  "description": "Update database host",
  "path": "/etc/myapp/config.ini",
  "state": "present",
  "regexp": "^db_host\\s*=\\s*.*$",
  "replace": "db_host = newdb.example.com"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "replace"
description = "Update database host"
path = "/etc/myapp/config.ini"
state = "present"
regexp = "^db_host\\s*=\\s*.*$"
replace = "db_host = newdb.example.com"
```

**Replace string literal**:

**YAML Format**:

```yaml
- type: replace
  description: "Update version number"
  path: /opt/myapp/version.txt
  state: present
  before: 'version = "1.0.0"'
  replace: 'version = "1.1.0"'
```

**JSON Format**:

```json
{
  "type": "replace",
  "description": "Update version number",
  "path": "/opt/myapp/version.txt",
  "state": "present",
  "before": "version = \"1.0.0\"",
  "replace": "version = \"1.1.0\""
}
```

**TOML Format**:

```toml
[[tasks]]
type = "replace"
description = "Update version number"
path = "/opt/myapp/version.txt"
state = "present"
before = 'version = "1.0.0"'
replace = 'version = "1.1.0"'
```

**Replace with backup**:

**YAML Format**:

```yaml
- type: replace
  description: "Update configuration with backup"
  path: /etc/httpd/httpd.conf
  state: present
  regexp: '^Listen 80$'
  replace: 'Listen 8080'
  backup: true
```

**JSON Format**:

```json
{
  "type": "replace",
  "description": "Update configuration with backup",
  "path": "/etc/httpd/httpd.conf",
  "state": "present",
  "regexp": "^Listen 80$",
  "replace": "Listen 8080",
  "backup": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "replace"
description = "Update configuration with backup"
path = "/etc/httpd/httpd.conf"
state = "present"
regexp = "^Listen 80$"
replace = "Listen 8080"
backup = true
```

**Replace all occurrences**:

**YAML Format**:

```yaml
- type: replace
  description: "Update all IP addresses"
  path: /etc/hosts
  state: present
  regexp: '192\.168\.1\.\d+'
  replace: '10.0.0.100'
  replace_all: true
```

**JSON Format**:

```json
{
  "type": "replace",
  "description": "Update all IP addresses",
  "path": "/etc/hosts",
  "state": "present",
  "regexp": "192\\.168\\.1\\.\\d+",
  "replace": "10.0.0.100",
  "replace_all": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "replace"
description = "Update all IP addresses"
path = "/etc/hosts"
state = "present"
regexp = "192\\.168\\.1\\.\\d+"
replace = "10.0.0.100"
replace_all = true
```

#### stat

**Description**: File/directory statistics task

**Required Fields**:

- `checksum` (bool):
  Get checksum of file

- `checksum_algorithm` (ChecksumAlgorithm):
  Checksum algorithm

- `follow` (bool):
  Whether to follow symlinks

- `path` (String):
  Path to check

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `register` (Option<String>):
  Optional variable name to register the task result in

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Registered Outputs**:

- `checksum` (String): The file checksum (if `checksum` is true)
- `exists` (bool): Whether the file or directory exists
- `gid` (u32): The group ID of the owner
- `is_dir` (bool): Whether the path is a directory
- `is_file` (bool): Whether the path is a file
- `mode` (u32): The file mode (permissions)
- `modified` (u64): Last modification time (epoch seconds)
- `size` (u64): The size of the file in bytes
- `uid` (u32): The user ID of the owner

**Examples**:

**Get file statistics**:

**YAML Format**:

```yaml
- type: stat
  description: "Get file statistics"
  path: /etc/passwd
```

**JSON Format**:

```json
{
  "type": "stat",
  "description": "Get file statistics",
  "path": "/etc/passwd"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "stat"
description = "Get file statistics"
path = "/etc/passwd"
```

**Get file checksum**:

**YAML Format**:

```yaml
- type: stat
  description: "Get file checksum"
  path: /etc/hosts
  checksum: true
  checksum_algorithm: sha256
```

**JSON Format**:

```json
{
  "type": "stat",
  "description": "Get file checksum",
  "path": "/etc/hosts",
  "checksum": true,
  "checksum_algorithm": "sha256"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "stat"
description = "Get file checksum"
path = "/etc/hosts"
checksum = true
checksum_algorithm = "sha256"
```

**Follow symlinks**:

**YAML Format**:

```yaml
- type: stat
  description: "Follow symlink for statistics"
  path: /var/log/syslog
  follow: true
```

**JSON Format**:

```json
{
  "type": "stat",
  "description": "Follow symlink for statistics",
  "path": "/var/log/syslog",
  "follow": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "stat"
description = "Follow symlink for statistics"
path = "/var/log/syslog"
follow = true
```

**Register file status**:

**YAML Format**:

```yaml
- type: stat
  description: "Check if nginx config exists"
  path: /etc/nginx/nginx.conf
  register: nginx_conf
- type: debug
  msg: "Nginx config exists: {{ nginx_conf.exists }}"
  when: "{{ nginx_conf.exists }}"
```

**JSON Format**:

```json
[
  {
    "type": "stat",
    "description": "Check if nginx config exists",
    "path": "/etc/nginx/nginx.conf",
    "register": "nginx_conf"
  },
  {
    "type": "debug",
    "msg": "Nginx config exists: {{ nginx_conf.exists }}",
    "when": "{{ nginx_conf.exists }}"
  }
]
```

**TOML Format**:

```toml
[[tasks]]
type = "stat"
description = "Check if nginx config exists"
path = "/etc/nginx/nginx.conf"
register = "nginx_conf"
[[tasks]]
type = "debug"
msg = "Nginx config exists: {{ nginx_conf.exists }}"
when = "{{ nginx_conf.exists }}"
```

**Get directory statistics**:

**YAML Format**:

```yaml
- type: stat
  description: "Get directory statistics"
  path: /home/user
```

**JSON Format**:

```json
{
  "type": "stat",
  "description": "Get directory statistics",
  "path": "/home/user"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "stat"
description = "Get directory statistics"
path = "/home/user"
```

#### template

**Description**: Template rendering task

**Required Fields**:

- `backup` (bool):
  Backup destination before templating

- `dest` (String):
  Destination file

- `force` (bool):
  Force template rendering

- `src` (String):
  Source template file

- `state` (TemplateState):
  Template state

- `vars` (HashMap<String, Value>):
  Variables for template rendering

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Render a template**:

**YAML Format**:

```yaml
- type: template
  description: "Render nginx configuration"
  src: /templates/nginx.conf.j2
  dest: /etc/nginx/sites-available/default
  state: present
  vars:
    server_name: example.com
    port: 80
    root_dir: /var/www/html
```

**JSON Format**:

```json
{
  "type": "template",
  "description": "Render nginx configuration",
  "src": "/templates/nginx.conf.j2",
  "dest": "/etc/nginx/sites-available/default",
  "state": "present",
  "vars": {
    "server_name": "example.com",
    "port": 80,
    "root_dir": "/var/www/html"
  }
}
```

**TOML Format**:

```toml
[[tasks]]
type = "template"
description = "Render nginx configuration"
src = "/templates/nginx.conf.j2"
dest = "/etc/nginx/sites-available/default"
state = "present"
[tasks.vars]
server_name = "example.com"
port = 80
root_dir = "/var/www/html"
```

**Render template with backup**:

**YAML Format**:

```yaml
- type: template
  description: "Update config with backup"
  src: /templates/app.conf.j2
  dest: /etc/myapp/config.conf
  state: present
  backup: true
  vars:
    database_host: localhost
    database_port: 5432
```

**JSON Format**:

```json
{
  "type": "template",
  "description": "Update config with backup",
  "src": "/templates/app.conf.j2",
  "dest": "/etc/myapp/config.conf",
  "state": "present",
  "backup": true,
  "vars": {
    "database_host": "localhost",
    "database_port": 5432
  }
}
```

**TOML Format**:

```toml
[[tasks]]
type = "template"
description = "Update config with backup"
src = "/templates/app.conf.j2"
dest = "/etc/myapp/config.conf"
state = "present"
backup = true
[tasks.vars]
database_host = "localhost"
database_port = 5432
```

**Remove rendered template**:

**YAML Format**:

```yaml
- type: template
  description: "Remove rendered configuration"
  src: /templates/old.conf.j2
  dest: /etc/oldapp/config.conf
  state: absent
```

**JSON Format**:

```json
{
  "type": "template",
  "description": "Remove rendered configuration",
  "src": "/templates/old.conf.j2",
  "dest": "/etc/oldapp/config.conf",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "template"
description = "Remove rendered configuration"
src = "/templates/old.conf.j2"
dest = "/etc/oldapp/config.conf"
state = "absent"
```

#### unarchive

**Description**: Unarchive files task

**Required Fields**:

- `creates` (bool):
  Whether to create destination directory

- `dest` (String):
  Destination directory

- `extra_opts` (Vec<String>):
  Extra options for extraction

- `follow_redirects` (bool):
  Follow redirects for URL downloads

- `headers` (HashMap<String, String>):
  HTTP headers for URL downloads

- `keep_original` (bool):
  Whether to keep the archive after extraction

- `list_files` (Vec<String>):
  List of files to extract (empty = all)

- `src` (String):
  Source archive file (local path) or URL

- `state` (UnarchiveState):
  Unarchive state

- `timeout` (u64):
  Timeout for URL downloads

- `validate_certs` (bool):
  Validate SSL certificates for URL downloads

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `format` (Option<ArchiveFormat>):
  Archive format (auto-detect if not specified)

- `password` (Option<String>):
  Password for basic auth for URL downloads

- `username` (Option<String>):
  Username for basic auth for URL downloads

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Extract a tar archive**:

**YAML Format**:

```yaml
- type: unarchive
  description: "Extract application archive"
  src: /tmp/myapp.tar.gz
  dest: /opt/myapp
  state: present
```

**JSON Format**:

```json
{
  "type": "unarchive",
  "description": "Extract application archive",
  "src": "/tmp/myapp.tar.gz",
  "dest": "/opt/myapp",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "unarchive"
description = "Extract application archive"
src = "/tmp/myapp.tar.gz"
dest = "/opt/myapp"
state = "present"
```

**Extract from URL**:

**YAML Format**:

```yaml
- type: unarchive
  description: "Download and extract software"
  src: https://example.com/software.tar.gz
  dest: /opt/software
  state: present
  creates: true
```

**JSON Format**:

```json
{
  "type": "unarchive",
  "description": "Download and extract software",
  "src": "https://example.com/software.tar.gz",
  "dest": "/opt/software",
  "state": "present",
  "creates": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "unarchive"
description = "Download and extract software"
src = "https://example.com/software.tar.gz"
dest = "/opt/software"
state = "present"
creates = true
```

**Extract specific files**:

**YAML Format**:

```yaml
- type: unarchive
  description: "Extract configuration files"
  src: /tmp/configs.tar.gz
  dest: /etc/myapp
  state: present
  list_files:
    - config.yml
    - settings.json
```

**JSON Format**:

```json
{
  "type": "unarchive",
  "description": "Extract configuration files",
  "src": "/tmp/configs.tar.gz",
  "dest": "/etc/myapp",
  "state": "present",
  "list_files": ["config.yml", "settings.json"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "unarchive"
description = "Extract configuration files"
src = "/tmp/configs.tar.gz"
dest = "/etc/myapp"
state = "present"
list_files = ["config.yml", "settings.json"]
```

**Extract zip archive**:

**YAML Format**:

```yaml
- type: unarchive
  description: "Extract zip archive"
  src: /tmp/data.zip
  dest: /var/data
  state: present
  format: zip
```

**JSON Format**:

```json
{
  "type": "unarchive",
  "description": "Extract zip archive",
  "src": "/tmp/data.zip",
  "dest": "/var/data",
  "state": "present",
  "format": "zip"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "unarchive"
description = "Extract zip archive"
src = "/tmp/data.zip"
dest = "/var/data"
state = "present"
format = "zip"
```

### Monitoring & Logging

#### journald

**Description**: Journald configuration task

Manages systemd journal configuration. Can modify the main /etc/systemd/journald.conf
file or create drop-in configuration files in /etc/systemd/journald.conf.d/.
Supports all journald configuration options like storage settings, size limits,
forwarding options, and compression settings.

**Required Fields**:

- `config` (HashMap<String, String>):
  Journald configuration options

  Key-value pairs of journald configuration options. Required when state is present.
  Common options include:
  - Storage: volatile|persistent|auto|none
  - SystemMaxUse: Maximum disk space to use
  - SystemKeepFree: Disk space to keep free
  - SystemMaxFileSize: Maximum size of individual journal files
  - MaxRetentionSec: Maximum time to retain journal entries
  - ForwardToSyslog: Forward to syslog
  - Compress: Enable compression

- `state` (JournaldState):
  Configuration state (present, absent)

  - `present`: Ensure the journald configuration exists
  - `absent`: Ensure the journald configuration does not exist

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `name` (Option<String>):
  Configuration name (for drop-in configs)

  Name of the drop-in configuration file to create in /etc/systemd/journald.conf.d/.
  If not specified, modifies the main /etc/systemd/journald.conf file.
  This becomes the filename (e.g., "storage" creates /etc/systemd/journald.conf.d/storage.conf).

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Configure journald storage and rotation**:

**YAML Format**:

```yaml
- type: journald
  description: "Configure systemd journal settings"
  config:
    Storage: persistent
    SystemMaxUse: 100M
    SystemKeepFree: 500M
    SystemMaxFileSize: 10M
    MaxRetentionSec: 1week
  state: present
```

**JSON Format**:

```json
{
  "type": "journald",
  "description": "Configure systemd journal settings",
  "config": {
    "Storage": "persistent",
    "SystemMaxUse": "100M",
    "SystemKeepFree": "500M",
    "SystemMaxFileSize": "10M",
    "MaxRetentionSec": "1week"
  },
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "journald"
description = "Configure systemd journal settings"
[tasks.config]
Storage = "persistent"
SystemMaxUse = "100M"
SystemKeepFree = "500M"
SystemMaxFileSize = "10M"
MaxRetentionSec = "1week"
state = "present"
```

#### logrotate

**Description**: Logrotate configuration task

Manages logrotate configuration files in /etc/logrotate.d/.
Creates or removes logrotate configuration snippets for log rotation management.

**Required Fields**:

- `name` (String):
  Configuration name

  Name of the logrotate configuration file to create in /etc/logrotate.d/.
  This becomes the filename (e.g., "nginx" creates /etc/logrotate.d/nginx).

- `options` (Vec<String>):
  Logrotate options

  List of logrotate configuration options. Common options include:
  - "daily", "weekly", "monthly", "yearly"
  - "rotate N" (keep N rotations)
  - "compress", "delaycompress"
  - "missingok", "notifempty"
  - "create MODE OWNER GROUP"

- `state` (LogrotateState):
  Configuration state (present, absent)

  - `present`: Ensure the logrotate configuration exists
  - `absent`: Ensure the logrotate configuration does not exist

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `path` (Option<String>):
  Log file path(s)

  Path or glob pattern for log files to rotate. Required when state is present.
  Examples: "/var/log/app/*.log", "/var/log/nginx/access.log"

- `postrotate` (Option<String>):
  Post-rotate script

  Shell commands to execute after log rotation.
  Commonly used to reload services after log rotation.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a logrotate configuration for nginx**:

**YAML Format**:

```yaml
- type: logrotate
  description: "Configure nginx log rotation"
  name: nginx
  path: /var/log/nginx/*.log
  options:
    - weekly
    - rotate 52
    - compress
    - delaycompress
    - missingok
    - notifempty
    - create 644 www-data www-data
  postrotate: |
    systemctl reload nginx
  state: present
```

**JSON Format**:

```json
{
  "type": "logrotate",
  "description": "Configure nginx log rotation",
  "name": "nginx",
  "path": "/var/log/nginx/*.log",
  "options": [
    "weekly",
    "rotate 52",
    "compress",
    "delaycompress",
    "missingok",
    "notifempty",
    "create 644 www-data www-data"
  ],
  "postrotate": "systemctl reload nginx\n",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "logrotate"
description = "Configure nginx log rotation"
name = "nginx"
path = "/var/log/nginx/*.log"
options = [
  "weekly",
  "rotate 52",
  "compress",
  "delaycompress",
  "missingok",
  "notifempty",
  "create 644 www-data www-data"
]
postrotate = """
systemctl reload nginx
"""
state = "present"
```

#### rsyslog

**Description**: Rsyslog configuration task

Manages rsyslog configuration files in /etc/rsyslog.d/.
Creates or removes rsyslog configuration snippets for log processing and forwarding.

**Required Fields**:

- `name` (String):
  Configuration name

  Name of the rsyslog configuration file to create in /etc/rsyslog.d/.
  This becomes the filename (e.g., "remote-logging" creates /etc/rsyslog.d/remote-logging.conf).

- `state` (RsyslogState):
  Configuration state (present, absent)

  - `present`: Ensure the rsyslog configuration exists
  - `absent`: Ensure the rsyslog configuration does not exist

**Optional Fields**:

- `config` (Option<String>):
  Rsyslog configuration content

  The rsyslog configuration directives. Required when state is present.
  Examples include log forwarding rules, custom log files, filters, etc.

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create an rsyslog configuration for remote logging**:

**YAML Format**:

```yaml
- type: rsyslog
  description: "Configure remote log forwarding"
  name: remote-logging
  config: |
    # Forward all logs to remote server
    *.* @@logserver.example.com:514
    # Forward auth logs with TCP
    auth.* @@logserver.example.com:514
  state: present
```

**JSON Format**:

```json
{
  "type": "rsyslog",
  "description": "Configure remote log forwarding",
  "name": "remote-logging",
  "config": "*.* @@logserver.example.com:514\n\nauth.* @@logserver.example.com:514\n",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "rsyslog"
description = "Configure remote log forwarding"
name = "remote-logging"
config = """
*.* @@logserver.example.com:514
auth.* @@logserver.example.com:514
"""
state = "present"
```

### Network Operations

#### geturl

**Description**: Download files from HTTP/HTTPS/FTP task

Downloads files from web servers or FTP servers. Supports authentication,
checksum validation, and file permission management. Similar to Ansible's `get_url` module.

**Required Fields**:

- `backup` (bool):
  Backup destination before download

- `dest` (String):
  Destination file path

- `follow_redirects` (bool):
  Follow redirects

- `force` (bool):
  Force download even if file exists

- `headers` (HashMap<String, String>):
  HTTP headers

- `state` (GetUrlState):
  Get URL state

- `timeout` (u64):
  Timeout in seconds

- `url` (String):
  Source URL

- `validate_certs` (bool):
  Validate SSL certificates

**Optional Fields**:

- `checksum` (Option<String>):
  Checksum validation

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `group` (Option<String>):
  File group

- `mode` (Option<String>):
  File permissions (octal string like "0644")

- `owner` (Option<String>):
  File owner

- `password` (Option<String>):
  Password for basic auth

- `username` (Option<String>):
  Username for basic auth

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Download a file**:

**YAML Format**:

```yaml
- type: get_url
  description: "Download configuration file"
  url: https://example.com/config.yml
  dest: /etc/myapp/config.yml
  state: present
```

**JSON Format**:

```json
{
  "type": "get_url",
  "description": "Download configuration file",
  "url": "https://example.com/config.yml",
  "dest": "/etc/myapp/config.yml",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "get_url"
description = "Download configuration file"
url = "https://example.com/config.yml"
dest = "/etc/myapp/config.yml"
state = "present"
```

**Download with checksum validation**:

**YAML Format**:

```yaml
- type: get_url
  description: "Download software with checksum validation"
  url: https://example.com/software.tar.gz
  dest: /tmp/software.tar.gz
  state: present
  checksum: sha256:abc123def456...
```

**JSON Format**:

```json
{
  "type": "get_url",
  "description": "Download software with checksum validation",
  "url": "https://example.com/software.tar.gz",
  "dest": "/tmp/software.tar.gz",
  "state": "present",
  "checksum": "sha256:abc123def456..."
}
```

**TOML Format**:

```toml
[[tasks]]
type = "get_url"
description = "Download software with checksum validation"
url = "https://example.com/software.tar.gz"
dest = "/tmp/software.tar.gz"
state = "present"
checksum = "sha256:abc123def456..."
```

**Download with authentication**:

**YAML Format**:

```yaml
- type: get_url
  description: "Download private file"
  url: https://private.example.com/file.txt
  dest: /tmp/private.txt
  state: present
  username: myuser
  password: mypassword
```

**JSON Format**:

```json
{
  "type": "get_url",
  "description": "Download private file",
  "url": "https://private.example.com/file.txt",
  "dest": "/tmp/private.txt",
  "state": "present",
  "username": "myuser",
  "password": "mypassword"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "get_url"
description = "Download private file"
url = "https://private.example.com/file.txt"
dest = "/tmp/private.txt"
state = "present"
username = "myuser"
password = "mypassword"
```

**Download and set permissions**:

**YAML Format**:

```yaml
- type: get_url
  description: "Download script with proper permissions"
  url: https://example.com/script.sh
  dest: /usr/local/bin/myscript.sh
  state: present
  mode: "0755"
  owner: root
  group: root
```

**JSON Format**:

```json
{
  "type": "get_url",
  "description": "Download script with proper permissions",
  "url": "https://example.com/script.sh",
  "dest": "/usr/local/bin/myscript.sh",
  "state": "present",
  "mode": "0755",
  "owner": "root",
  "group": "root"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "get_url"
description = "Download script with proper permissions"
url = "https://example.com/script.sh"
dest = "/usr/local/bin/myscript.sh"
state = "present"
mode = "0755"
owner = "root"
group = "root"
```

#### uri

**Description**: Interact with web services task

Makes HTTP requests to web services and APIs. Validates responses and can
return content. Similar to Ansible's `uri` module.

**Required Fields**:

- `follow_redirects` (bool):
  Follow redirects

- `force` (bool):
  Force execution even if idempotent

- `headers` (HashMap<String, String>):
  HTTP headers

- `method` (HttpMethod):
  HTTP method

- `return_content` (bool):
  Return content in result

- `state` (UriState):
  URI state

- `status_code` (Vec<u16>):
  Expected status codes

- `timeout` (u64):
  Timeout in seconds

- `url` (String):
  Target URL

- `validate_certs` (bool):
  Validate SSL certificates

**Optional Fields**:

- `body` (Option<String>):
  Request body

- `content_type` (Option<String>):
  Content type for request body

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `password` (Option<String>):
  Password for basic auth

- `register` (Option<String>):
  Optional variable name to register the task result in

- `username` (Option<String>):
  Username for basic auth

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Registered Outputs**:

- `changed` (bool): Whether the request was successfully made
- `content` (String): The body of the response (if `return_content` is true)
- `status` (u16): The HTTP status code of the response

**Examples**:

**Simple GET request**:

**YAML Format**:

```yaml
- type: uri
  description: "Check API health endpoint"
  url: https://api.example.com/health
  method: GET
  status_code: 200
  return_content: true
```

**JSON Format**:

```json
{
  "type": "uri",
  "description": "Check API health endpoint",
  "url": "https://api.example.com/health",
  "method": "GET",
  "status_code": 200,
  "return_content": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "uri"
description = "Check API health endpoint"
url = "https://api.example.com/health"
method = "GET"
status_code = 200
return_content = true
```

**POST request with JSON body**:

**YAML Format**:

```yaml
- type: uri
  description: "Create a new user via API"
  url: https://api.example.com/users
  method: POST
  body: "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}"
  headers:
    Content-Type: application/json
  status_code: 201
```

**JSON Format**:

```json
{
  "type": "uri",
  "description": "Create a new user via API",
  "url": "https://api.example.com/users",
  "method": "POST",
  "body": "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}",
  "headers": {
    "Content-Type": "application/json"
  },
  "status_code": 201
}
```

**TOML Format**:

```toml
[[tasks]]
type = "uri"
description = "Create a new user via API"
url = "https://api.example.com/users"
method = "POST"
body = "{\"name\": \"John Doe\", \"email\": \"john@example.com\"}"
[tasks.headers]
Content-Type = "application/json"
[tasks]]
status_code = 201
```

**Request with authentication**:

**YAML Format**:

```yaml
- type: uri
  description: "Get user profile with authentication"
  url: https://api.example.com/profile
  method: GET
  username: myuser
  password: mypassword
  return_content: true
```

**JSON Format**:

```json
{
  "type": "uri",
  "description": "Get user profile with authentication",
  "url": "https://api.example.com/profile",
  "method": "GET",
  "username": "myuser",
  "password": "mypassword",
  "return_content": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "uri"
description = "Get user profile with authentication"
url = "https://api.example.com/profile"
method = "GET"
username = "myuser"
password = "mypassword"
return_content = true
```

**Register URI response**:

**YAML Format**:

```yaml
- type: uri
  description: "Get health status"
  url: https://api.example.com/health
  register: health_response
  return_content: true
- type: debug
  msg: "The API status code is: {{ health_response.status }}"
```

**JSON Format**:

```json
[
  {
    "type": "uri",
    "description": "Get health status",
    "url": "https://api.example.com/health",
    "register": "health_response",
    "return_content": true
  },
  {
    "type": "debug",
    "msg": "The API status code is: {{ health_response.status }}"
  }
]
```

**TOML Format**:

```toml
[[tasks]]
type = "uri"
description = "Get health status"
url = "https://api.example.com/health"
register = "health_response"
return_content = true
[[tasks]]
type = "debug"
msg = "The API status code is: {{ health_response.status }}"
```

### Package Management

#### apt

**Description**: Debian/Ubuntu package management task

**Required Fields**:

- `allow_downgrades` (bool):
  Allow downgrades

- `allow_unauthenticated` (bool):
  Allow unauthenticated packages

- `autoclean` (bool):
  Autoclean package cache

- `autoremove` (bool):
  Autoremove unused packages

- `cache_valid_time` (u32):
  Cache validity time in seconds

- `force` (bool):
  Force installation

- `name` (String):
  Package name

- `state` (PackageState):
  Package state

- `update_cache` (bool):
  Update package cache

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install a package**:

**YAML Format**:

```yaml
- type: apt
  description: "Install curl package"
  name: curl
  state: present
```

**JSON Format**:

```json
{
  "type": "apt",
  "description": "Install curl package",
  "name": "curl",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "apt"
description = "Install curl package"
name = "curl"
state = "present"
```

**Install package with cache update**:

**YAML Format**:

```yaml
- type: apt
  description: "Install nginx with cache update"
  name: nginx
  state: present
  update_cache: true
```

**JSON Format**:

```json
{
  "type": "apt",
  "description": "Install nginx with cache update",
  "name": "nginx",
  "state": "present",
  "update_cache": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "apt"
description = "Install nginx with cache update"
name = "nginx"
state = "present"
update_cache = true
```

**Remove a package**:

**YAML Format**:

```yaml
- type: apt
  description: "Remove apache2 package"
  name: apache2
  state: absent
```

**JSON Format**:

```json
{
  "type": "apt",
  "description": "Remove apache2 package",
  "name": "apache2",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "apt"
description = "Remove apache2 package"
name = "apache2"
state = "absent"
```

**Update package to latest version**:

**YAML Format**:

```yaml
- type: apt
  description: "Update vim to latest version"
  name: vim
  state: latest
  update_cache: true
```

**JSON Format**:

```json
{
  "type": "apt",
  "description": "Update vim to latest version",
  "name": "vim",
  "state": "latest",
  "update_cache": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "apt"
description = "Update vim to latest version"
name = "vim"
state = "latest"
update_cache = true
```

#### gem

**Description**: Ruby gem management task

**Required Fields**:

- `executable` (String):
  Ruby executable path

- `extra_args` (Vec<String>):
  Extra arguments

- `force` (bool):
  Force installation

- `gem_executable` (String):
  Gem executable path

- `install_doc` (bool):
  Install documentation

- `name` (String):
  Gem name

- `state` (PackageState):
  Gem state

- `user_install` (bool):
  User installation

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `version` (Option<String>):
  Version specification

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install a gem**:

**YAML Format**:

```yaml
- type: gem
  description: "Install bundler gem"
  name: bundler
  state: present
```

**JSON Format**:

```json
{
  "type": "gem",
  "description": "Install bundler gem",
  "name": "bundler",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "gem"
description = "Install bundler gem"
name = "bundler"
state = "present"
```

**Install gem with specific version**:

**YAML Format**:

```yaml
- type: gem
  description: "Install Rails 7.0"
  name: rails
  state: present
  version: "7.0.0"
```

**JSON Format**:

```json
{
  "type": "gem",
  "description": "Install Rails 7.0",
  "name": "rails",
  "state": "present",
  "version": "7.0.0"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "gem"
description = "Install Rails 7.0"
name = "rails"
state = "present"
version = "7.0.0"
```

**Install gem for specific user**:

**YAML Format**:

```yaml
- type: gem
  description: "Install jekyll for user"
  name: jekyll
  state: present
  user_install: true
```

**JSON Format**:

```json
{
  "type": "gem",
  "description": "Install jekyll for user",
  "name": "jekyll",
  "state": "present",
  "user_install": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "gem"
description = "Install jekyll for user"
name = "jekyll"
state = "present"
user_install = true
```

**Remove a gem**:

**YAML Format**:

```yaml
- type: gem
  description: "Remove bundler gem"
  name: bundler
  state: absent
```

**JSON Format**:

```json
{
  "type": "gem",
  "description": "Remove bundler gem",
  "name": "bundler",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "gem"
description = "Remove bundler gem"
name = "bundler"
state = "absent"
```

#### npm

**Description**: Node.js package management task

**Required Fields**:

- `executable` (String):
  NPM executable path

- `extra_args` (Vec<String>):
  Extra arguments

- `force` (bool):
  Force installation

- `global` (bool):
  Global installation

- `name` (String):
  Package name

- `production` (bool):
  Production only

- `state` (PackageState):
  Package state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `registry` (Option<String>):
  Registry URL

- `version` (Option<String>):
  Version specification

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install an npm package**:

**YAML Format**:

```yaml
- type: npm
  description: "Install express package"
  name: express
  state: present
```

**JSON Format**:

```json
{
  "type": "npm",
  "description": "Install express package",
  "name": "express",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "npm"
description = "Install express package"
name = "express"
state = "present"
```

**Install package globally**:

**YAML Format**:

```yaml
- type: npm
  description: "Install PM2 globally"
  name: pm2
  state: present
  global: true
```

**JSON Format**:

```json
{
  "type": "npm",
  "description": "Install PM2 globally",
  "name": "pm2",
  "state": "present",
  "global": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "npm"
description = "Install PM2 globally"
name = "pm2"
state = "present"
global = true
```

**Install specific version**:

**YAML Format**:

```yaml
- type: npm
  description: "Install React 18"
  name: react
  state: present
  version: "18.2.0"
```

**JSON Format**:

```json
{
  "type": "npm",
  "description": "Install React 18",
  "name": "react",
  "state": "present",
  "version": "18.2.0"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "npm"
description = "Install React 18"
name = "react"
state = "present"
version = "18.2.0"
```

**Remove an npm package**:

**YAML Format**:

```yaml
- type: npm
  description: "Remove express package"
  name: express
  state: absent
```

**JSON Format**:

```json
{
  "type": "npm",
  "description": "Remove express package",
  "name": "express",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "npm"
description = "Remove express package"
name = "express"
state = "absent"
```

#### package

**Description**: Package management task

**Required Fields**:

- `name` (String):
  Package name

- `state` (PackageState):
  Package state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `manager` (Option<String>):
  Package manager to use (auto-detect if not specified)

- `register` (Option<String>):
  Optional variable name to register the task result in

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Registered Outputs**:

- `changed` (bool): Whether any packages were installed or removed
- `packages` (Vec<String>): List of packages affected

**Examples**:

**Install a package**:

**YAML Format**:

```yaml
- type: package
  description: "Install nginx web server"
  name: nginx
  state: present
```

**JSON Format**:

```json
{
  "type": "package",
  "description": "Install nginx web server",
  "name": "nginx",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "package"
description = "Install nginx web server"
name = "nginx"
state = "present"
```

**Install with specific package manager**:

**YAML Format**:

```yaml
- type: package
  description: "Install curl using apt"
  name: curl
  state: present
  manager: apt
```

**JSON Format**:

```json
{
  "type": "package",
  "description": "Install curl using apt",
  "name": "curl",
  "state": "present",
  "manager": "apt"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "package"
description = "Install curl using apt"
name = "curl"
state = "present"
manager = "apt"
```

**Update a package to latest version**:

**YAML Format**:

```yaml
- type: package
  description: "Update vim to latest version"
  name: vim
  state: latest
```

**JSON Format**:

```json
{
  "type": "package",
  "description": "Update vim to latest version",
  "name": "vim",
  "state": "latest"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "package"
description = "Update vim to latest version"
name = "vim"
state = "latest"
```

**Remove a package**:

**YAML Format**:

```yaml
- type: package
  description: "Remove telnet client"
  name: telnet
  state: absent
```

**JSON Format**:

```json
{
  "type": "package",
  "description": "Remove telnet client",
  "name": "telnet",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "package"
description = "Remove telnet client"
name = "telnet"
state = "absent"
```

**Register package installation**:

**YAML Format**:

```yaml
- type: package
  description: "Install git and check if changed"
  name: git
  state: present
  register: git_install
- type: debug
  msg: "Git was newly installed"
  when: "{{ git_install.changed }}"
```

**JSON Format**:

```json
[
  {
    "type": "package",
    "description": "Install git and check if changed",
    "name": "git",
    "state": "present",
    "register": "git_install"
  },
  {
    "type": "debug",
    "msg": "Git was newly installed",
    "when": "{{ git_install.changed }}"
  }
]
```

**TOML Format**:

```toml
[[tasks]]
type = "package"
description = "Install git and check if changed"
name = "git"
state = "present"
register = "git_install"
[[tasks]]
type = "debug"
msg = "Git was newly installed"
when = "{{ git_install.changed }}"
```

#### pacman

**Description**: Arch Linux package management task

**Required Fields**:

- `force` (bool):
  Force installation/removal

- `name` (String):
  Package name

- `reinstall` (bool):
  Force reinstallation

- `remove_config` (bool):
  Remove configuration files

- `remove_dependencies` (bool):
  Remove dependencies

- `state` (PackageState):
  Package state

- `update_cache` (bool):
  Update package database

- `upgrade` (bool):
  Upgrade system

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install a package**:

**YAML Format**:

```yaml
- type: pacman
  description: "Install vim package"
  name: vim
  state: present
```

**JSON Format**:

```json
{
  "type": "pacman",
  "description": "Install vim package",
  "name": "vim",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pacman"
description = "Install vim package"
name = "vim"
state = "present"
```

**Install with cache update**:

**YAML Format**:

```yaml
- type: pacman
  description: "Install nginx with cache update"
  name: nginx
  state: present
  update_cache: true
```

**JSON Format**:

```json
{
  "type": "pacman",
  "description": "Install nginx with cache update",
  "name": "nginx",
  "state": "present",
  "update_cache": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pacman"
description = "Install nginx with cache update"
name = "nginx"
state = "present"
update_cache = true
```

**Remove a package**:

**YAML Format**:

```yaml
- type: pacman
  description: "Remove vim package"
  name: vim
  state: absent
```

**JSON Format**:

```json
{
  "type": "pacman",
  "description": "Remove vim package",
  "name": "vim",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pacman"
description = "Remove vim package"
name = "vim"
state = "absent"
```

**Remove package with dependencies**:

**YAML Format**:

```yaml
- type: pacman
  description: "Remove package with dependencies"
  name: old-package
  state: absent
  remove_dependencies: true
```

**JSON Format**:

```json
{
  "type": "pacman",
  "description": "Remove package with dependencies",
  "name": "old-package",
  "state": "absent",
  "remove_dependencies": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pacman"
description = "Remove package with dependencies"
name = "old-package"
state = "absent"
remove_dependencies = true
```

#### pip

**Description**: Python package management task

**Required Fields**:

- `executable` (String):
  Python executable path

- `extra_args` (Vec<String>):
  Extra arguments

- `force` (bool):
  Force installation

- `name` (String):
  Package name

- `state` (PackageState):
  Package state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `requirements` (Option<String>):
  Requirements file

- `version` (Option<String>):
  Version specification

- `virtualenv` (Option<String>):
  Virtual environment path

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install a Python package**:

**YAML Format**:

```yaml
- type: pip
  description: "Install requests package"
  name: requests
  state: present
```

**JSON Format**:

```json
{
  "type": "pip",
  "description": "Install requests package",
  "name": "requests",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pip"
description = "Install requests package"
name = "requests"
state = "present"
```

**Install package in virtual environment**:

**YAML Format**:

```yaml
- type: pip
  description: "Install Django in virtualenv"
  name: django
  state: present
  virtualenv: /opt/myapp/venv
```

**JSON Format**:

```json
{
  "type": "pip",
  "description": "Install Django in virtualenv",
  "name": "django",
  "state": "present",
  "virtualenv": "/opt/myapp/venv"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pip"
description = "Install Django in virtualenv"
name = "django"
state = "present"
virtualenv = "/opt/myapp/venv"
```

**Install specific version**:

**YAML Format**:

```yaml
- type: pip
  description: "Install Flask 2.0"
  name: flask
  state: present
  version: "2.0.0"
```

**JSON Format**:

```json
{
  "type": "pip",
  "description": "Install Flask 2.0",
  "name": "flask",
  "state": "present",
  "version": "2.0.0"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pip"
description = "Install Flask 2.0"
name = "flask"
state = "present"
version = "2.0.0"
```

**Remove a Python package**:

**YAML Format**:

```yaml
- type: pip
  description: "Remove requests package"
  name: requests
  state: absent
```

**JSON Format**:

```json
{
  "type": "pip",
  "description": "Remove requests package",
  "name": "requests",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "pip"
description = "Remove requests package"
name = "requests"
state = "absent"
```

#### yum

**Description**: RHEL/CentOS/Fedora package management task

**Required Fields**:

- `allow_downgrades` (bool):
  Allow downgrades

- `disable_excludes` (bool):
  Disable excludes

- `disable_gpg_check` (bool):
  Disable GPG check

- `force` (bool):
  Force installation

- `install_recommended` (bool):
  Install recommended packages

- `install_suggested` (bool):
  Install suggested packages

- `name` (String):
  Package name

- `state` (PackageState):
  Package state

- `update_cache` (bool):
  Update package cache

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install a package**:

**YAML Format**:

```yaml
- type: yum
  description: "Install nginx web server"
  name: nginx
  state: present
```

**JSON Format**:

```json
{
  "type": "yum",
  "description": "Install nginx web server",
  "name": "nginx",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "yum"
description = "Install nginx web server"
name = "nginx"
state = "present"
```

**Install with cache update**:

**YAML Format**:

```yaml
- type: yum
  description: "Install curl with cache update"
  name: curl
  state: present
  update_cache: true
```

**JSON Format**:

```json
{
  "type": "yum",
  "description": "Install curl with cache update",
  "name": "curl",
  "state": "present",
  "update_cache": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "yum"
description = "Install curl with cache update"
name = "curl"
state = "present"
update_cache = true
```

**Remove a package**:

**YAML Format**:

```yaml
- type: yum
  description: "Remove telnet package"
  name: telnet
  state: absent
```

**JSON Format**:

```json
{
  "type": "yum",
  "description": "Remove telnet package",
  "name": "telnet",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "yum"
description = "Remove telnet package"
name = "telnet"
state = "absent"
```

#### zypper

**Description**: SUSE package management task

**Required Fields**:

- `allow_downgrades` (bool):
  Allow downgrades

- `allow_vendor_change` (bool):
  Allow vendor changes

- `disable_gpg_check` (bool):
  Disable GPG check

- `force` (bool):
  Force installation

- `name` (String):
  Package name

- `state` (PackageState):
  Package state

- `update_cache` (bool):
  Update package cache

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Install a package**:

**YAML Format**:

```yaml
- type: zypper
  description: "Install apache web server"
  name: apache2
  state: present
```

**JSON Format**:

```json
{
  "type": "zypper",
  "description": "Install apache web server",
  "name": "apache2",
  "state": "present"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "zypper"
description = "Install apache web server"
name = "apache2"
state = "present"
```

**Install with cache update**:

**YAML Format**:

```yaml
- type: zypper
  description: "Install vim with repository refresh"
  name: vim
  state: present
  update_cache: true
```

**JSON Format**:

```json
{
  "type": "zypper",
  "description": "Install vim with repository refresh",
  "name": "vim",
  "state": "present",
  "update_cache": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "zypper"
description = "Install vim with repository refresh"
name = "vim"
state = "present"
update_cache = true
```

**Remove a package**:

**YAML Format**:

```yaml
- type: zypper
  description: "Remove telnet package"
  name: telnet
  state: absent
```

**JSON Format**:

```json
{
  "type": "zypper",
  "description": "Remove telnet package",
  "name": "telnet",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "zypper"
description = "Remove telnet package"
name = "telnet"
state = "absent"
```

### Security & Access

#### authorizedkey

**Description**: SSH authorized key management task

**Required Fields**:

- `create_ssh_dir` (bool):
  Whether to create .ssh directory if it doesn't exist

- `manage_dir` (bool):
  Whether to manage SSH directory permissions

- `state` (AuthorizedKeyState):
  SSH state (present/absent)

- `unique` (bool):
  Whether to deduplicate keys

- `user` (String):
  Target user for SSH key management

- `validate_key` (bool):
  Whether to validate key format

**Optional Fields**:

- `comment` (Option<String>):
  Comment to identify this key

- `description` (Option<String>):
  Optional description of what this task does

- `key` (Option<String>):
  SSH public key content (inline)

- `key_file` (Option<String>):
  Path to SSH public key file

- `key_options` (Option<String>):
  Key options (comma-separated list)

- `path` (Option<String>):
  Path to authorized_keys file (defaults to ~/.ssh/authorized_keys)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Add SSH public key**:

**YAML Format**:

```yaml
- type: authorized_key
  description: "Add SSH key for admin user"
  user: admin
  state: present
  key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... user@host"
```

**JSON Format**:

```json
{
  "type": "authorized_key",
  "description": "Add SSH key for admin user",
  "user": "admin",
  "state": "present",
  "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... user@host"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "authorized_key"
description = "Add SSH key for admin user"
user = "admin"
state = "present"
key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... user@host"
```

**Add SSH key from file**:

**YAML Format**:

```yaml
- type: authorized_key
  description: "Add SSH key from file"
  user: deploy
  state: present
  key_file: /tmp/id_rsa.pub
  comment: "Deployment key"
```

**JSON Format**:

```json
{
  "type": "authorized_key",
  "description": "Add SSH key from file",
  "user": "deploy",
  "state": "present",
  "key_file": "/tmp/id_rsa.pub",
  "comment": "Deployment key"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "authorized_key"
description = "Add SSH key from file"
user = "deploy"
state = "present"
key_file = "/tmp/id_rsa.pub"
comment = "Deployment key"
```

**Add SSH key with restrictions**:

**YAML Format**:

```yaml
- type: authorized_key
  description: "Add restricted SSH key"
  user: backup
  state: present
  key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... backup@host"
  key_options: "command=\"/usr/local/bin/backup.sh\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding"
```

**JSON Format**:

```json
{
  "type": "authorized_key",
  "description": "Add restricted SSH key",
  "user": "backup",
  "state": "present",
  "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... backup@host",
  "key_options": "command=\"/usr/local/bin/backup.sh\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "authorized_key"
description = "Add restricted SSH key"
user = "backup"
state = "present"
key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... backup@host"
key_options = "command=\"/usr/local/bin/backup.sh\",no-port-forwarding,no-X11-forwarding,no-agent-forwarding"
```

**Remove SSH key**:

**YAML Format**:

```yaml
- type: authorized_key
  description: "Remove SSH key"
  user: olduser
  state: absent
  key: "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... olduser@host"
```

**JSON Format**:

```json
{
  "type": "authorized_key",
  "description": "Remove SSH key",
  "user": "olduser",
  "state": "absent",
  "key": "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... olduser@host"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "authorized_key"
description = "Remove SSH key"
user = "olduser"
state = "absent"
key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC7vbqajDhS... olduser@host"
```

#### firewalld

**Description**: Firewalld firewall management task

**Required Fields**:

- `check_running` (bool):
  Whether to check if firewalld is running

- `permanent` (bool):
  Whether to make changes permanent

- `reload` (bool):
  Whether to reload firewall after changes

- `state` (FirewalldState):
  Firewall state (present/absent)

- `zone` (String):
  Zone to manage (defaults to "public")

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

- `port` (Option<String>):
  Port to manage (e.g., "8080/tcp", "53/udp")

- `rich_rule` (Option<String>):
  Rich rule to manage

- `service` (Option<String>):
  Service to manage (e.g., "http", "ssh")

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Allow SSH service**:

**YAML Format**:

```yaml
- type: firewalld
  description: "Allow SSH access"
  state: present
  service: ssh
  zone: public
  permanent: true
```

**JSON Format**:

```json
{
  "type": "firewalld",
  "description": "Allow SSH access",
  "state": "present",
  "service": "ssh",
  "zone": "public",
  "permanent": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "firewalld"
description = "Allow SSH access"
state = "present"
service = "ssh"
zone = "public"
permanent = true
```

**Allow custom port**:

**YAML Format**:

```yaml
- type: firewalld
  description: "Allow web traffic on port 8080"
  state: present
  port: "8080/tcp"
  zone: public
  permanent: true
```

**JSON Format**:

```json
{
  "type": "firewalld",
  "description": "Allow web traffic on port 8080",
  "state": "present",
  "port": "8080/tcp",
  "zone": "public",
  "permanent": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "firewalld"
description = "Allow web traffic on port 8080"
state = "present"
port = "8080/tcp"
zone = "public"
permanent = true
```

**Add rich rule**:

**YAML Format**:

```yaml
- type: firewalld
  description: "Allow traffic from specific IP"
  state: present
  rich_rule: 'rule family="ipv4" source address="192.168.1.100" accept'
  zone: public
  permanent: true
```

**JSON Format**:

```json
{
  "type": "firewalld",
  "description": "Allow traffic from specific IP",
  "state": "present",
  "rich_rule": "rule family=\"ipv4\" source address=\"192.168.1.100\" accept",
  "zone": "public",
  "permanent": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "firewalld"
description = "Allow traffic from specific IP"
state = "present"
rich_rule = 'rule family="ipv4" source address="192.168.1.100" accept'
zone = "public"
permanent = true
```

**Remove firewall rule**:

**YAML Format**:

```yaml
- type: firewalld
  description: "Remove SSH access"
  state: absent
  service: ssh
  zone: public
  permanent: true
```

**JSON Format**:

```json
{
  "type": "firewalld",
  "description": "Remove SSH access",
  "state": "absent",
  "service": "ssh",
  "zone": "public",
  "permanent": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "firewalld"
description = "Remove SSH access"
state = "absent"
service = "ssh"
zone = "public"
permanent = true
```

#### iptables

**Description**: iptables firewall management task

**Required Fields**:

- `chain` (String):
  Chain to manage (INPUT/OUTPUT/FORWARD/PREROUTING/POSTROUTING)

- `check_available` (bool):
  Whether to check if iptables is available

- `extra_args` (Vec<String>):
  Additional iptables arguments

- `ipv6` (bool):
  IPv6 mode (use ip6tables instead of iptables)

- `protocol` (String):
  Protocol (tcp/udp/icmp/all)

- `state` (IptablesState):
  iptables state (present/absent)

- `table` (String):
  Table to manage (filter/nat/mangle/raw/security)

- `target` (String):
  Target/jump action (ACCEPT/DROP/REJECT/LOG/MASQUERADE)

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

- `destination` (Option<String>):
  Destination IP/network (with optional mask)

- `dport` (Option<String>):
  Destination port (for tcp/udp)

- `in_interface` (Option<String>):
  Input interface

- `out_interface` (Option<String>):
  Output interface

- `source` (Option<String>):
  Source IP/network (with optional mask)

- `sport` (Option<String>):
  Source port (for tcp/udp)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Allow SSH access**:

**YAML Format**:

```yaml
- type: iptables
  description: "Allow SSH access"
  state: present
  table: filter
  chain: INPUT
  protocol: tcp
  dport: "22"
  target: ACCEPT
```

**JSON Format**:

```json
{
  "type": "iptables",
  "description": "Allow SSH access",
  "state": "present",
  "table": "filter",
  "chain": "INPUT",
  "protocol": "tcp",
  "dport": "22",
  "target": "ACCEPT"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "iptables"
description = "Allow SSH access"
state = "present"
table = "filter"
chain = "INPUT"
protocol = "tcp"
dport = "22"
target = "ACCEPT"
```

**Block specific IP address**:

**YAML Format**:

```yaml
- type: iptables
  description: "Block specific IP address"
  state: present
  table: filter
  chain: INPUT
  source: 192.168.1.100
  target: DROP
```

**JSON Format**:

```json
{
  "type": "iptables",
  "description": "Block specific IP address",
  "state": "present",
  "table": "filter",
  "chain": "INPUT",
  "source": "192.168.1.100",
  "target": "DROP"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "iptables"
description = "Block specific IP address"
state = "present"
table = "filter"
chain = "INPUT"
source = "192.168.1.100"
target = "DROP"
```

**Allow HTTP and HTTPS traffic**:

**YAML Format**:

```yaml
- type: iptables
  description: "Allow web traffic"
  state: present
  table: filter
  chain: INPUT
  protocol: tcp
  dport: "80,443"
  target: ACCEPT
```

**JSON Format**:

```json
{
  "type": "iptables",
  "description": "Allow web traffic",
  "state": "present",
  "table": "filter",
  "chain": "INPUT",
  "protocol": "tcp",
  "dport": "80,443",
  "target": "ACCEPT"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "iptables"
description = "Allow web traffic"
state = "present"
table = "filter"
chain = "INPUT"
protocol = "tcp"
dport = "80,443"
target = "ACCEPT"
```

**Remove iptables rule**:

**YAML Format**:

```yaml
- type: iptables
  description: "Remove SSH blocking rule"
  state: absent
  table: filter
  chain: INPUT
  protocol: tcp
  dport: "22"
  target: DROP
```

**JSON Format**:

```json
{
  "type": "iptables",
  "description": "Remove SSH blocking rule",
  "state": "absent",
  "table": "filter",
  "chain": "INPUT",
  "protocol": "tcp",
  "dport": "22",
  "target": "DROP"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "iptables"
description = "Remove SSH blocking rule"
state = "absent"
table = "filter"
chain = "INPUT"
protocol = "tcp"
dport = "22"
target = "DROP"
```

#### selinux

**Description**: SELinux policy management task

**Required Fields**:

- `follow` (bool):
  Whether to follow symlinks

- `ignore_missing` (bool):
  Whether to ignore missing files

- `persistent` (bool):
  Whether to make changes persistent

- `recurse` (bool):
  Whether to recurse into directories

- `state` (SelinuxState):
  SELinux state (present/absent/enforcing/permissive/disabled)

**Optional Fields**:

- `boolean` (Option<String>):
  SELinux boolean to manage

- `context` (Option<String>):
  SELinux context to set

- `description` (Option<String>):
  Optional description of what this task does

- `policy` (Option<String>):
  Policy type (targeted/mls)

- `serange` (Option<String>):
  SELinux level/range to set

- `serole` (Option<String>):
  SELinux role to set

- `setype` (Option<String>):
  SELinux type to set

- `seuser` (Option<String>):
  SELinux user to set

- `target` (Option<String>):
  File/directory to set context on

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Enable SELinux boolean**:

**YAML Format**:

```yaml
- type: selinux
  description: "Enable httpd_can_network_connect"
  state: on
  boolean: httpd_can_network_connect
```

**JSON Format**:

```json
{
  "type": "selinux",
  "description": "Enable httpd_can_network_connect",
  "state": "on",
  "boolean": "httpd_can_network_connect"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "selinux"
description = "Enable httpd_can_network_connect"
state = "on"
boolean = "httpd_can_network_connect"
```

**Set file context**:

**YAML Format**:

```yaml
- type: selinux
  description: "Set httpd context for web directory"
  state: context
  target: /var/www/html
  setype: httpd_sys_content_t
  recurse: true
```

**JSON Format**:

```json
{
  "type": "selinux",
  "description": "Set httpd context for web directory",
  "state": "context",
  "target": "/var/www/html",
  "setype": "httpd_sys_content_t",
  "recurse": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "selinux"
description = "Set httpd context for web directory"
state = "context"
target = "/var/www/html"
setype = "httpd_sys_content_t"
recurse = true
```

**Set SELinux to enforcing mode**:

**YAML Format**:

```yaml
- type: selinux
  description: "Set SELinux to enforcing mode"
  state: enforcing
```

**JSON Format**:

```json
{
  "type": "selinux",
  "description": "Set SELinux to enforcing mode",
  "state": "enforcing"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "selinux"
description = "Set SELinux to enforcing mode"
state = "enforcing"
```

**Restore file contexts**:

**YAML Format**:

```yaml
- type: selinux
  description: "Restore SELinux contexts"
  state: restorecon
  target: /etc/httpd
  recurse: true
```

**JSON Format**:

```json
{
  "type": "selinux",
  "description": "Restore SELinux contexts",
  "state": "restorecon",
  "target": "/etc/httpd",
  "recurse": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "selinux"
description = "Restore SELinux contexts"
state = "restorecon"
target = "/etc/httpd"
recurse = true
```

#### sudoers

**Description**: Sudoers configuration management task

**Required Fields**:

- `backup` (bool):
  Backup file before modification

- `commands` (Vec<String>):
  Commands to allow (defaults to ALL)

- `group` (bool):
  Whether this is a group (prefix with %)

- `hosts` (Vec<String>):
  Hosts to allow (defaults to ALL)

- `name` (String):
  User or group to grant sudo privileges

- `noexec` (bool):
  NOEXEC option (prevent shell escapes)

- `nopasswd` (bool):
  NOPASSWD option (don't require password)

- `setenv` (bool):
  SETENV option (allow environment variable setting)

- `state` (SudoersState):
  Sudoers state (present/absent)

- `validate` (bool):
  Whether to validate sudoers syntax after changes

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

- `path` (Option<String>):
  Path to sudoers file (defaults to /etc/sudoers)

- `runas` (Option<String>):
  Run as user (defaults to ALL)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Grant sudo access to user**:

**YAML Format**:

```yaml
- type: sudoers
  description: "Grant sudo access to admin user"
  state: present
  name: admin
  commands: ["ALL"]
  hosts: ["ALL"]
```

**JSON Format**:

```json
{
  "type": "sudoers",
  "description": "Grant sudo access to admin user",
  "state": "present",
  "name": "admin",
  "commands": ["ALL"],
  "hosts": ["ALL"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sudoers"
description = "Grant sudo access to admin user"
state = "present"
name = "admin"
commands = ["ALL"]
hosts = ["ALL"]
```

**Grant sudo access to group**:

**YAML Format**:

```yaml
- type: sudoers
  description: "Grant sudo access to wheel group"
  state: present
  name: wheel
  group: true
  commands: ["ALL"]
  hosts: ["ALL"]
```

**JSON Format**:

```json
{
  "type": "sudoers",
  "description": "Grant sudo access to wheel group",
  "state": "present",
  "name": "wheel",
  "group": true,
  "commands": ["ALL"],
  "hosts": ["ALL"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sudoers"
description = "Grant sudo access to wheel group"
state = "present"
name = "wheel"
group = true
commands = ["ALL"]
hosts = ["ALL"]
```

**Grant passwordless sudo for specific commands**:

**YAML Format**:

```yaml
- type: sudoers
  description: "Grant passwordless sudo for service management"
  state: present
  name: deploy
  commands: ["/usr/bin/systemctl", "/usr/bin/service"]
  hosts: ["ALL"]
  nopasswd: true
```

**JSON Format**:

```json
{
  "type": "sudoers",
  "description": "Grant passwordless sudo for service management",
  "state": "present",
  "name": "deploy",
  "commands": ["/usr/bin/systemctl", "/usr/bin/service"],
  "hosts": ["ALL"],
  "nopasswd": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sudoers"
description = "Grant passwordless sudo for service management"
state = "present"
name = "deploy"
commands = ["/usr/bin/systemctl", "/usr/bin/service"]
hosts = ["ALL"]
nopasswd = true
```

**Remove sudo privileges**:

**YAML Format**:

```yaml
- type: sudoers
  description: "Remove sudo access from user"
  state: absent
  name: olduser
```

**JSON Format**:

```json
{
  "type": "sudoers",
  "description": "Remove sudo access from user",
  "state": "absent",
  "name": "olduser"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sudoers"
description = "Remove sudo access from user"
state = "absent"
name = "olduser"
```

#### ufw

**Description**: UFW firewall management task

**Required Fields**:

- `state` (UfwState):
  UFW state

**Optional Fields**:

- `default` (Option<String>):
  Default policy for chains

- `description` (Option<String>):
  Optional description of what this task does

- `direction` (Option<String>):
  Direction (in/out)

- `from` (Option<String>):
  Source IP/network (for from parameter)

- `interface` (Option<String>):
  Interface to apply rule to

- `logging` (Option<String>):
  Logging level

- `port` (Option<String>):
  Port to manage (e.g., "80", "443/tcp", "53/udp")

- `proto` (Option<String>):
  Protocol (tcp/udp)

- `rule` (Option<String>):
  Rule to manage

- `to` (Option<String>):
  Destination IP/network (for to parameter)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Enable UFW firewall**:

**YAML Format**:

```yaml
- type: ufw
  description: "Enable UFW firewall"
  state: enabled
```

**JSON Format**:

```json
{
  "type": "ufw",
  "description": "Enable UFW firewall",
  "state": "enabled"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "ufw"
description = "Enable UFW firewall"
state = "enabled"
```

**Allow SSH access**:

**YAML Format**:

```yaml
- type: ufw
  description: "Allow SSH access"
  state: allow
  port: "22"
  proto: tcp
```

**JSON Format**:

```json
{
  "type": "ufw",
  "description": "Allow SSH access",
  "state": "allow",
  "port": "22",
  "proto": "tcp"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "ufw"
description = "Allow SSH access"
state = "allow"
port = "22"
proto = "tcp"
```

**Allow HTTP and HTTPS**:

**YAML Format**:

```yaml
- type: ufw
  description: "Allow web traffic"
  state: allow
  port: "80,443"
  proto: tcp
```

**JSON Format**:

```json
{
  "type": "ufw",
  "description": "Allow web traffic",
  "state": "allow",
  "port": "80,443",
  "proto": "tcp"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "ufw"
description = "Allow web traffic"
state = "allow"
port = "80,443"
proto = "tcp"
```

**Deny specific IP address**:

**YAML Format**:

```yaml
- type: ufw
  description: "Block specific IP address"
  state: deny
  from: 192.168.1.100
```

**JSON Format**:

```json
{
  "type": "ufw",
  "description": "Block specific IP address",
  "state": "deny",
  "from": "192.168.1.100"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "ufw"
description = "Block specific IP address"
state = "deny"
from = "192.168.1.100"
```

### Source Control

#### git

**Description**: Git repository management task

**Required Fields**:

- `accept_hostkey` (bool):
  Accept host key

  Will ensure or not that -o StrictHostKeyChecking=no is present as an ssh option.

- `clone` (bool):
  Whether to clone if repository doesn't exist

  If false, do not clone the repository even if it does not exist locally.

- `dest` (String):
  Destination directory

  The path of where the repository should be checked out.

- `force` (bool):
  Whether to force checkout

  If true, any modified files in the working repository will be discarded.

- `recursive` (bool):
  Whether to clone recursively (include submodules)

  If false, repository will be cloned without the --recursive option.

- `remote` (String):
  Remote name

  Name of the remote.

- `repo` (String):
  Git repository URL

  The git, SSH, or HTTP(S) protocol address of the git repository.

- `update` (bool):
  Whether to update the repository

  If false, do not retrieve new revisions from the origin repository.

- `version` (String):
  Version to check out

  What version of the repository to check out. This can be the literal string HEAD,
  a branch name, a tag name, or a SHA-1 hash.

**Optional Fields**:

- `depth` (Option<usize>):
  Depth for shallow clone

  Create a shallow clone with a history truncated to the specified number of revisions.

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `key_file` (Option<String>):
  SSH key file

  Specify an optional private key file path to use for the checkout.

- `register` (Option<String>):
  Optional variable name to register the task result in

- `ssh_opts` (Option<String>):
  SSH options

  Options git will pass to ssh when used as protocol.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Registered Outputs**:

- `after` (String): The SHA-1 hash after the task has run
- `before` (String): The SHA-1 hash before the task has run
- `changed` (bool): Whether the repository was updated or cloned

**Examples**:

**Clone a repository**:

**YAML Format**:

```yaml
- type: git
  description: "Clone application repository"
  repo: https://github.com/user/myapp.git
  dest: /opt/myapp
  version: main
```

**JSON Format**:

```json
{
  "type": "git",
  "description": "Clone application repository",
  "repo": "https://github.com/user/myapp.git",
  "dest": "/opt/myapp",
  "version": "main"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "git"
description = "Clone application repository"
repo = "https://github.com/user/myapp.git"
dest = "/opt/myapp"
version = "main"
```

**Clone specific branch**:

**YAML Format**:

```yaml
- type: git
  description: "Clone development branch"
  repo: https://github.com/user/myapp.git
  dest: /opt/myapp-dev
  version: develop
```

**JSON Format**:

```json
{
  "type": "git",
  "description": "Clone development branch",
  "repo": "https://github.com/user/myapp.git",
  "dest": "/opt/myapp-dev",
  "version": "develop"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "git"
description = "Clone development branch"
repo = "https://github.com/user/myapp.git"
dest = "/opt/myapp-dev"
version = "develop"
```

**Clone with submodules**:

**YAML Format**:

```yaml
- type: git
  description: "Clone repository with submodules"
  repo: https://github.com/user/myapp.git
  dest: /opt/myapp
  recursive: true
```

**JSON Format**:

```json
{
  "type": "git",
  "description": "Clone repository with submodules",
  "repo": "https://github.com/user/myapp.git",
  "dest": "/opt/myapp",
  "recursive": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "git"
description = "Clone repository with submodules"
repo = "https://github.com/user/myapp.git"
dest = "/opt/myapp"
recursive = true
```

**Clone specific commit**:

**YAML Format**:

```yaml
- type: git
  description: "Clone specific commit"
  repo: https://github.com/user/myapp.git
  dest: /opt/myapp
  version: abc123def456
```

**JSON Format**:

```json
{
  "type": "git",
  "description": "Clone specific commit",
  "repo": "https://github.com/user/myapp.git",
  "dest": "/opt/myapp",
  "version": "abc123def456"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "git"
description = "Clone specific commit"
repo = "https://github.com/user/myapp.git"
dest = "/opt/myapp"
version = "abc123def456"
```

**Register repository state**:

**YAML Format**:

```yaml
- type: git
  description: "Clone rust source"
  repo: https://github.com/rust-lang/rust.git
  dest: /opt/rust
  register: rust_repo
- type: debug
  msg: "Rust repo is at {{ rust_repo.after }}"
```

**JSON Format**:

```json
[
  {
    "type": "git",
    "description": "Clone rust source",
    "repo": "https://github.com/rust-lang/rust.git",
    "dest": "/opt/rust",
    "register": "rust_repo"
  },
  {
    "type": "debug",
    "msg": "Rust repo is at {{ rust_repo.after }}"
  }
]
```

**TOML Format**:

```toml
[[tasks]]
type = "git"
description = "Clone rust source"
repo = "https://github.com/rust-lang/rust.git"
dest = "/opt/rust"
register = "rust_repo"
[[tasks]]
type = "debug"
msg = "Rust repo is at {{ rust_repo.after }}"
```

### System Administration

#### cron

**Description**: Cron job management task

**Required Fields**:

- `day` (String):
  Day of month (1-31, or * for any)

- `hour` (String):
  Hour (0-23, or * for any)

- `job` (String):
  Command to execute

- `minute` (String):
  Minute (0-59, or * for any)

- `month` (String):
  Month (1-12, or * for any)

- `name` (String):
  Unique name for this cron job

- `state` (CronState):
  Cron job state

- `user` (String):
  User to run the job as

- `weekday` (String):
  Day of week (0-7, or * for any)

**Optional Fields**:

- `comment` (Option<String>):
  Optional comment/description

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a cron job**:

**YAML Format**:

```yaml
- type: cron
  description: "Create daily backup cron job"
  name: daily-backup
  state: present
  user: root
  minute: "0"
  hour: "2"
  day: "*"
  month: "*"
  weekday: "*"
  job: "/usr/local/bin/backup.sh"
```

**JSON Format**:

```json
{
  "type": "cron",
  "description": "Create daily backup cron job",
  "name": "daily-backup",
  "state": "present",
  "user": "root",
  "minute": "0",
  "hour": "2",
  "day": "*",
  "month": "*",
  "weekday": "*",
  "job": "/usr/local/bin/backup.sh"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "cron"
description = "Create daily backup cron job"
name = "daily-backup"
state = "present"
user = "root"
minute = "0"
hour = "2"
day = "*"
month = "*"
weekday = "*"
job = "/usr/local/bin/backup.sh"
```

**Create cron job with specific schedule**:

**YAML Format**:

```yaml
- type: cron
  description: "Weekly maintenance on Mondays"
  name: weekly-maintenance
  state: present
  user: root
  minute: "0"
  hour: "9"
  day: "*"
  month: "*"
  weekday: "1"
  job: "/usr/local/bin/maintenance.sh"
  comment: "Weekly system maintenance"
```

**JSON Format**:

```json
{
  "type": "cron",
  "description": "Weekly maintenance on Mondays",
  "name": "weekly-maintenance",
  "state": "present",
  "user": "root",
  "minute": "0",
  "hour": "9",
  "day": "*",
  "month": "*",
  "weekday": "1",
  "job": "/usr/local/bin/maintenance.sh",
  "comment": "Weekly system maintenance"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "cron"
description = "Weekly maintenance on Mondays"
name = "weekly-maintenance"
state = "present"
user = "root"
minute = "0"
hour = "9"
day = "*"
month = "*"
weekday = "1"
job = "/usr/local/bin/maintenance.sh"
comment = "Weekly system maintenance"
```

**Remove a cron job**:

**YAML Format**:

```yaml
- type: cron
  description: "Remove daily backup cron job"
  name: daily-backup
  state: absent
```

**JSON Format**:

```json
{
  "type": "cron",
  "description": "Remove daily backup cron job",
  "name": "daily-backup",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "cron"
description = "Remove daily backup cron job"
name = "daily-backup"
state = "absent"
```

**Cron job with complex schedule**:

**YAML Format**:

```yaml
- type: cron
  description: "Monitor service every 15 minutes during business hours"
  name: service-monitor
  state: present
  user: monitor
  minute: "*/15"
  hour: "9-17"
  day: "1-5"
  month: "*"
  weekday: "*"
  job: "/usr/local/bin/check-service.sh"
  comment: "Business hours service monitoring"
```

**JSON Format**:

```json
{
  "type": "cron",
  "description": "Monitor service every 15 minutes during business hours",
  "name": "service-monitor",
  "state": "present",
  "user": "monitor",
  "minute": "*/15",
  "hour": "9-17",
  "day": "1-5",
  "month": "*",
  "weekday": "*",
  "job": "/usr/local/bin/check-service.sh",
  "comment": "Business hours service monitoring"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "cron"
description = "Monitor service every 15 minutes during business hours"
name = "service-monitor"
state = "present"
user = "monitor"
minute = "*/15"
hour = "9-17"
day = "1-5"
month = "*"
weekday = "*"
job = "/usr/local/bin/check-service.sh"
comment = "Business hours service monitoring"
```

#### filesystem

**Description**: Filesystem creation/deletion task

**Required Fields**:

- `dev` (String):
  Device path

- `force` (bool):
  Force filesystem creation (dangerous!)

- `opts` (Vec<String>):
  Additional mkfs options

- `state` (FilesystemState):
  Filesystem state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `fstype` (Option<String>):
  Filesystem type (ext4, xfs, btrfs, etc.)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create an ext4 filesystem**:

**YAML Format**:

```yaml
- type: filesystem
  description: "Create ext4 filesystem"
  dev: /dev/sdb1
  state: present
  fstype: ext4
```

**JSON Format**:

```json
{
  "type": "filesystem",
  "description": "Create ext4 filesystem",
  "dev": "/dev/sdb1",
  "state": "present",
  "fstype": "ext4"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "filesystem"
description = "Create ext4 filesystem"
dev = "/dev/sdb1"
state = "present"
fstype = "ext4"
```

**Create an XFS filesystem**:

**YAML Format**:

```yaml
- type: filesystem
  description: "Create XFS filesystem"
  dev: /dev/sdc1
  state: present
  fstype: xfs
  opts: ["-f", "-i", "size=512"]
```

**JSON Format**:

```json
{
  "type": "filesystem",
  "description": "Create XFS filesystem",
  "dev": "/dev/sdc1",
  "state": "present",
  "fstype": "xfs",
  "opts": ["-f", "-i", "size=512"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "filesystem"
description = "Create XFS filesystem"
dev = "/dev/sdc1"
state = "present"
fstype = "xfs"
opts = ["-f", "-i", "size=512"]
```

**Create a Btrfs filesystem**:

**YAML Format**:

```yaml
- type: filesystem
  description: "Create Btrfs filesystem"
  dev: /dev/sdd1
  state: present
  fstype: btrfs
```

**JSON Format**:

```json
{
  "type": "filesystem",
  "description": "Create Btrfs filesystem",
  "dev": "/dev/sdd1",
  "state": "present",
  "fstype": "btrfs"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "filesystem"
description = "Create Btrfs filesystem"
dev = "/dev/sdd1"
state = "present"
fstype = "btrfs"
```

**Force create filesystem**:

**YAML Format**:

```yaml
- type: filesystem
  description: "Force create ext4 filesystem"
  dev: /dev/sde1
  state: present
  fstype: ext4
  force: true
```

**JSON Format**:

```json
{
  "type": "filesystem",
  "description": "Force create ext4 filesystem",
  "dev": "/dev/sde1",
  "state": "present",
  "fstype": "ext4",
  "force": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "filesystem"
description = "Force create ext4 filesystem"
dev = "/dev/sde1"
state = "present"
fstype = "ext4"
force = true
```

#### group

**Description**: Group management task

**Required Fields**:

- `name` (String):
  Group name

- `state` (GroupState):
  Group state

- `system` (bool):
  Whether group is a system group

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `gid` (Option<u32>):
  Group ID

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a group**:

**YAML Format**:

```yaml
- type: group
  description: "Create a web application group"
  name: webapp
  state: present
  gid: 1001
```

**JSON Format**:

```json
{
  "type": "group",
  "description": "Create a web application group",
  "name": "webapp",
  "state": "present",
  "gid": 1001
}
```

**TOML Format**:

```toml
[[tasks]]
type = "group"
description = "Create a web application group"
name = "webapp"
state = "present"
gid = 1001
```

**Create a system group**:

**YAML Format**:

```yaml
- type: group
  description: "Create a system group for nginx"
  name: nginx
  state: present
  system: true
```

**JSON Format**:

```json
{
  "type": "group",
  "description": "Create a system group for nginx",
  "name": "nginx",
  "state": "present",
  "system": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "group"
description = "Create a system group for nginx"
name = "nginx"
state = "present"
system = true
```

**Remove a group**:

**YAML Format**:

```yaml
- type: group
  description: "Remove the old group"
  name: oldgroup
  state: absent
```

**JSON Format**:

```json
{
  "type": "group",
  "description": "Remove the old group",
  "name": "oldgroup",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "group"
description = "Remove the old group"
name = "oldgroup"
state = "absent"
```

#### hostname

**Description**: System hostname management task

**Required Fields**:

- `name` (String):
  Desired hostname

- `persist` (bool):
  Whether to persist hostname to /etc/hostname

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Set system hostname**:

**YAML Format**:

```yaml
- type: hostname
  description: "Set system hostname"
  name: web-server-01
  persist: true
```

**JSON Format**:

```json
{
  "type": "hostname",
  "description": "Set system hostname",
  "name": "web-server-01",
  "persist": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "hostname"
description = "Set system hostname"
name = "web-server-01"
persist = true
```

**Set hostname temporarily**:

**YAML Format**:

```yaml
- type: hostname
  description: "Set temporary hostname"
  name: temp-server
  persist: false
```

**JSON Format**:

```json
{
  "type": "hostname",
  "description": "Set temporary hostname",
  "name": "temp-server",
  "persist": false
}
```

**TOML Format**:

```toml
[[tasks]]
type = "hostname"
description = "Set temporary hostname"
name = "temp-server"
persist = false
```

**Set hostname with domain**:

**YAML Format**:

```yaml
- type: hostname
  description: "Set fully qualified hostname"
  name: app.example.com
  persist: true
```

**JSON Format**:

```json
{
  "type": "hostname",
  "description": "Set fully qualified hostname",
  "name": "app.example.com",
  "persist": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "hostname"
description = "Set fully qualified hostname"
name = "app.example.com"
persist = true
```

#### mount

**Description**: Filesystem mounting task

**Required Fields**:

- `fstab` (bool):
  Whether to update /etc/fstab

- `opts` (Vec<String>):
  Mount options

- `path` (String):
  Mount point path

- `recursive` (bool):
  Whether to mount recursively

- `src` (String):
  Device to mount (device path, UUID, LABEL, etc.)

- `state` (MountState):
  Mount state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `fstype` (Option<String>):
  Filesystem type

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Mount a filesystem**:

**YAML Format**:

```yaml
- type: mount
  description: "Mount data partition"
  path: /mnt/data
  state: mounted
  src: /dev/sdb1
  fstype: ext4
  opts: ["defaults"]
```

**JSON Format**:

```json
{
  "type": "mount",
  "description": "Mount data partition",
  "path": "/mnt/data",
  "state": "mounted",
  "src": "/dev/sdb1",
  "fstype": "ext4",
  "opts": ["defaults"]
}
```

**TOML Format**:

```toml
[[tasks]]
type = "mount"
description = "Mount data partition"
path = "/mnt/data"
state = "mounted"
src = "/dev/sdb1"
fstype = "ext4"
opts = ["defaults"]
```

**Mount with fstab entry**:

**YAML Format**:

```yaml
- type: mount
  description: "Mount NFS share with fstab entry"
  path: /mnt/nfs
  state: present
  src: 192.168.1.100:/export/data
  fstype: nfs
  opts: ["defaults", "vers=4"]
  fstab: true
```

**JSON Format**:

```json
{
  "type": "mount",
  "description": "Mount NFS share with fstab entry",
  "path": "/mnt/nfs",
  "state": "present",
  "src": "192.168.1.100:/export/data",
  "fstype": "nfs",
  "opts": ["defaults", "vers=4"],
  "fstab": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "mount"
description = "Mount NFS share with fstab entry"
path = "/mnt/nfs"
state = "present"
src = "192.168.1.100:/export/data"
fstype = "nfs"
opts = ["defaults", "vers=4"]
fstab = true
```

**Unmount a filesystem**:

**YAML Format**:

```yaml
- type: mount
  description: "Unmount temporary mount"
  path: /mnt/temp
  state: unmounted
```

**JSON Format**:

```json
{
  "type": "mount",
  "description": "Unmount temporary mount",
  "path": "/mnt/temp",
  "state": "unmounted"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "mount"
description = "Unmount temporary mount"
path = "/mnt/temp"
state = "unmounted"
```

**Remove fstab entry**:

**YAML Format**:

```yaml
- type: mount
  description: "Remove fstab entry"
  path: /mnt/old
  state: absent
  fstab: true
```

**JSON Format**:

```json
{
  "type": "mount",
  "description": "Remove fstab entry",
  "path": "/mnt/old",
  "state": "absent",
  "fstab": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "mount"
description = "Remove fstab entry"
path = "/mnt/old"
state = "absent"
fstab = true
```

#### reboot

**Description**: System reboot task

**Required Fields**:

- `delay` (u32):
  Delay before reboot (seconds)

- `force` (bool):
  Whether to force reboot (don't wait for clean shutdown)

- `test` (bool):
  Test mode (don't actually reboot)

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `msg` (Option<String>):
  Message to display before reboot

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Reboot system with delay**:

**YAML Format**:

```yaml
- type: reboot
  description: "Reboot system after kernel update"
  delay: 60
  msg: "System will reboot in 60 seconds for kernel update"
  force: false
```

**JSON Format**:

```json
{
  "type": "reboot",
  "description": "Reboot system after kernel update",
  "delay": 60,
  "msg": "System will reboot in 60 seconds for kernel update",
  "force": false
}
```

**TOML Format**:

```toml
[[tasks]]
type = "reboot"
description = "Reboot system after kernel update"
delay = 60
msg = "System will reboot in 60 seconds for kernel update"
force = false
```

**Immediate reboot**:

**YAML Format**:

```yaml
- type: reboot
  description: "Immediate system reboot"
  delay: 0
  force: true
```

**JSON Format**:

```json
{
  "type": "reboot",
  "description": "Immediate system reboot",
  "delay": 0,
  "force": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "reboot"
description = "Immediate system reboot"
delay = 0
force = true
```

**Test reboot (dry run)**:

**YAML Format**:

```yaml
- type: reboot
  description: "Test reboot configuration"
  delay: 30
  msg: "This is a test reboot - system will not actually reboot"
  force: false
  test: true
```

**JSON Format**:

```json
{
  "type": "reboot",
  "description": "Test reboot configuration",
  "delay": 30,
  "msg": "This is a test reboot - system will not actually reboot",
  "force": false,
  "test": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "reboot"
description = "Test reboot configuration"
delay = 30
msg = "This is a test reboot - system will not actually reboot"
force = false
test = true
```

#### service

**Description**: Service management task

**Required Fields**:

- `name` (String):
  Service name

- `state` (ServiceState):
  Service state

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `enabled` (Option<bool>):
  Whether to enable service at boot

- `manager` (Option<String>):
  Service manager to use (auto-detect if not specified)

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Start and enable a service**:

**YAML Format**:

```yaml
- type: service
  description: "Start and enable nginx service"
  name: nginx
  state: started
  enabled: true
```

**JSON Format**:

```json
{
  "type": "service",
  "description": "Start and enable nginx service",
  "name": "nginx",
  "state": "started",
  "enabled": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "service"
description = "Start and enable nginx service"
name = "nginx"
state = "started"
enabled = true
```

#### shutdown

**Description**: System shutdown task

**Required Fields**:

- `delay` (u32):
  Delay before shutdown (seconds)

- `force` (bool):
  Whether to force shutdown (don't wait for clean shutdown)

- `test` (bool):
  Test mode (don't actually shutdown)

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `msg` (Option<String>):
  Message to display before shutdown

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Shutdown system with delay**:

**YAML Format**:

```yaml
- type: shutdown
  description: "Shutdown system for maintenance"
  delay: 30
  msg: "System will shutdown in 30 seconds for maintenance"
  force: false
```

**JSON Format**:

```json
{
  "type": "shutdown",
  "description": "Shutdown system for maintenance",
  "delay": 30,
  "msg": "System will shutdown in 30 seconds for maintenance",
  "force": false
}
```

**TOML Format**:

```toml
[[tasks]]
type = "shutdown"
description = "Shutdown system for maintenance"
delay = 30
msg = "System will shutdown in 30 seconds for maintenance"
force = false
```

**Immediate shutdown**:

**YAML Format**:

```yaml
- type: shutdown
  description: "Immediate system shutdown"
  delay: 0
  force: true
```

**JSON Format**:

```json
{
  "type": "shutdown",
  "description": "Immediate system shutdown",
  "delay": 0,
  "force": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "shutdown"
description = "Immediate system shutdown"
delay = 0
force = true
```

**Test shutdown (dry run)**:

**YAML Format**:

```yaml
- type: shutdown
  description: "Test shutdown configuration"
  delay: 60
  msg: "This is a test shutdown - system will not actually shutdown"
  force: false
  test: true
```

**JSON Format**:

```json
{
  "type": "shutdown",
  "description": "Test shutdown configuration",
  "delay": 60,
  "msg": "This is a test shutdown - system will not actually shutdown",
  "force": false,
  "test": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "shutdown"
description = "Test shutdown configuration"
delay = 60
msg = "This is a test shutdown - system will not actually shutdown"
force = false
test = true
```

#### sysctl

**Description**: Kernel parameter management task

**Required Fields**:

- `name` (String):
  Parameter name (e.g., "net.ipv4.ip_forward")

- `persist` (bool):
  Whether to persist changes to /etc/sysctl.conf

- `reload` (bool):
  Whether to reload immediately

- `state` (SysctlState):
  Parameter state

- `value` (String):
  Parameter value

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Set kernel parameter**:

**YAML Format**:

```yaml
- type: sysctl
  description: "Enable IP forwarding"
  name: net.ipv4.ip_forward
  state: present
  value: "1"
  persist: true
```

**JSON Format**:

```json
{
  "type": "sysctl",
  "description": "Enable IP forwarding",
  "name": "net.ipv4.ip_forward",
  "state": "present",
  "value": "1",
  "persist": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sysctl"
description = "Enable IP forwarding"
name = "net.ipv4.ip_forward"
state = "present"
value = "1"
persist = true
```

**Configure network buffer sizes**:

**YAML Format**:

```yaml
- type: sysctl
  description: "Increase network buffer sizes"
  name: net.core.rmem_max
  state: present
  value: "16777216"
  persist: true
```

**JSON Format**:

```json
{
  "type": "sysctl",
  "description": "Increase network buffer sizes",
  "name": "net.core.rmem_max",
  "state": "present",
  "value": "16777216",
  "persist": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sysctl"
description = "Increase network buffer sizes"
name = "net.core.rmem_max"
state = "present"
value = "16777216"
persist = true
```

**Disable IPv6**:

**YAML Format**:

```yaml
- type: sysctl
  description: "Disable IPv6"
  name: net.ipv6.conf.all.disable_ipv6
  state: present
  value: "1"
  persist: true
```

**JSON Format**:

```json
{
  "type": "sysctl",
  "description": "Disable IPv6",
  "name": "net.ipv6.conf.all.disable_ipv6",
  "state": "present",
  "value": "1",
  "persist": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sysctl"
description = "Disable IPv6"
name = "net.ipv6.conf.all.disable_ipv6"
state = "present"
value = "1"
persist = true
```

**Remove sysctl parameter**:

**YAML Format**:

```yaml
- type: sysctl
  description: "Remove custom sysctl parameter"
  name: net.ipv4.tcp_tw_reuse
  state: absent
```

**JSON Format**:

```json
{
  "type": "sysctl",
  "description": "Remove custom sysctl parameter",
  "name": "net.ipv4.tcp_tw_reuse",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "sysctl"
description = "Remove custom sysctl parameter"
name = "net.ipv4.tcp_tw_reuse"
state = "absent"
```

#### timezone

**Description**: System timezone management task

**Required Fields**:

- `name` (String):
  Timezone name (e.g., "America/New_York", "UTC")

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Set system timezone to UTC**:

**YAML Format**:

```yaml
- type: timezone
  description: "Set system timezone to UTC"
  name: UTC
```

**JSON Format**:

```json
{
  "type": "timezone",
  "description": "Set system timezone to UTC",
  "name": "UTC"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "timezone"
description = "Set system timezone to UTC"
name = "UTC"
```

**Set timezone to Eastern Time**:

**YAML Format**:

```yaml
- type: timezone
  description: "Set timezone to Eastern Time"
  name: America/New_York
```

**JSON Format**:

```json
{
  "type": "timezone",
  "description": "Set timezone to Eastern Time",
  "name": "America/New_York"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "timezone"
description = "Set timezone to Eastern Time"
name = "America/New_York"
```

**Set timezone to Pacific Time**:

**YAML Format**:

```yaml
- type: timezone
  description: "Set timezone to Pacific Time"
  name: America/Los_Angeles
```

**JSON Format**:

```json
{
  "type": "timezone",
  "description": "Set timezone to Pacific Time",
  "name": "America/Los_Angeles"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "timezone"
description = "Set timezone to Pacific Time"
name = "America/Los_Angeles"
```

#### user

**Description**: User and group management task

**Required Fields**:

- `groups` (Vec<String>):
  Additional groups

- `name` (String):
  Username

- `state` (UserState):
  User state

**Optional Fields**:

- `create_home` (Option<bool>):
  Whether to create home directory

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `gid` (Option<u32>):
  Group ID

- `home` (Option<String>):
  Home directory

- `password` (Option<String>):
  Password (hashed)

- `shell` (Option<String>):
  Shell

- `uid` (Option<u32>):
  User ID

- `when` (Option<String>):
  Optional condition to determine if the task should run

**Examples**:

**Create a user with basic settings**:

**YAML Format**:

```yaml
- type: user
  description: "Create a web application user"
  name: webapp
  state: present
  uid: 1001
  gid: 1001
  home: /home/webapp
  shell: /bin/bash
  create_home: true
```

**JSON Format**:

```json
{
  "type": "user",
  "description": "Create a web application user",
  "name": "webapp",
  "state": "present",
  "uid": 1001,
  "gid": 1001,
  "home": "/home/webapp",
  "shell": "/bin/bash",
  "create_home": true
}
```

**TOML Format**:

```toml
[[tasks]]
type = "user"
description = "Create a web application user"
name = "webapp"
state = "present"
uid = 1001
gid = 1001
home = "/home/webapp"
shell = "/bin/bash"
create_home = true
```

**Create a system user**:

**YAML Format**:

```yaml
- type: user
  description: "Create a system user for nginx"
  name: nginx
  state: present
  uid: 33
  gid: 33
  home: /var/lib/nginx
  shell: /usr/sbin/nologin
  create_home: false
```

**JSON Format**:

```json
{
  "type": "user",
  "description": "Create a system user for nginx",
  "name": "nginx",
  "state": "present",
  "uid": 33,
  "gid": 33,
  "home": "/var/lib/nginx",
  "shell": "/usr/sbin/nologin",
  "create_home": false
}
```

**TOML Format**:

```toml
[[tasks]]
type = "user"
description = "Create a system user for nginx"
name = "nginx"
state = "present"
uid = 33
gid = 33
home = "/var/lib/nginx"
shell = "/usr/sbin/nologin"
create_home = false
```

**Remove a user**:

**YAML Format**:

```yaml
- type: user
  description: "Remove the old user account"
  name: olduser
  state: absent
```

**JSON Format**:

```json
{
  "type": "user",
  "description": "Remove the old user account",
  "name": "olduser",
  "state": "absent"
}
```

**TOML Format**:

```toml
[[tasks]]
type = "user"
description = "Remove the old user account"
name = "olduser"
state = "absent"
```

### Utility/Control

#### assert

**Description**: Assert task for validating conditions

**Required Fields**:

- `quiet` (bool):
  Quiet mode

  Don't show success messages.

- `that` (String):
  Condition to assert

  Boolean expression that must evaluate to true.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `fail_msg` (Option<String>):
  Failure message

  Message to display when assertion fails.

- `success_msg` (Option<String>):
  Success message

  Message to display when assertion passes.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### debug

**Description**: Debug task for displaying information

**Required Fields**:

- `msg` (String):
  Message to display

  The message to print. Can be a string or variable reference.

- `verbosity` (DebugVerbosity):
  Verbosity level

  Control when this debug message is shown (normal/verbose).

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `var` (Option<String>):
  Variable to debug

  Variable name to display value of. Alternative to msg.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### fail

**Description**: Fail task for forcing execution failure

**Required Fields**:

- `msg` (String):
  Failure message

  Message to display when failing.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### includerole

**Description**: Include role task for reusable configurations

**Required Fields**:

- `defaults` (HashMap<String, Value>):
  Default variables

  Default variables for the role.

- `name` (String):
  Role name

  Name of the role to include.

- `vars` (HashMap<String, Value>):
  Variable overrides

  Variables to pass to the role.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### includetasks

**Description**: Include tasks task for modular configurations

**Required Fields**:

- `file` (String):
  File to include

  Path to the task file to include.

- `vars` (HashMap<String, Value>):
  Variable overrides

  Variables to pass to the included tasks.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### pause

**Description**: Pause task to halt execution

**Required Fields**:

- `minutes` (u64):
  Minutes to pause

  Duration to pause execution in minutes.

- `prompt` (String):
  Message to display during pause

  Message shown to user during pause.

- `seconds` (u64):
  Seconds to pause

  Duration to pause execution in seconds.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### setfact

**Description**: Set fact task for variable management

**Required Fields**:

- `cacheable` (bool):
  Cacheable flag

  Whether this fact can be cached between runs.

- `key` (String):
  Variable name

  Name of the fact/variable to set.

- `value` (Value):
  Variable value

  Value to assign to the variable.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `when` (Option<String>):
  Optional condition to determine if the task should run

#### waitfor

**Description**: Wait for task for synchronization

**Required Fields**:

- `active_connection` (bool):
  Active connection check

  Perform active connection attempt instead of just port scan.

- `delay` (u64):
  Delay between checks

  Time to wait between connectivity checks.

- `state` (ConnectionState):
  Connection state to wait for

  Whether to wait for connection to be started or stopped.

- `timeout` (u64):
  Timeout in seconds

  Maximum time to wait for condition.

**Optional Fields**:

- `description` (Option<String>):
  Optional description of what this task does

  Human-readable description of the task's purpose. Used for documentation
  and can be displayed in logs or reports.

- `host` (Option<String>):
  Host to wait for connectivity

  Hostname or IP address to check connectivity.

- `path` (Option<String>):
  Path to file to wait for

  File path to wait for existence or non-existence.

- `port` (Option<u16>):
  Port to check

  Port number to check for connectivity.

- `when` (Option<String>):
  Optional condition to determine if the task should run

## Facts Collectors (`facts`)

Facts collectors gather system metrics and inventory information. These collectors run at specified intervals to provide monitoring data.

**Note**: Facts collectors are not yet implemented. This section will be populated when facts collection functionality is added.

### Planned Collectors

- **system**: System information (OS, kernel, hardware)
- **cpu**: CPU usage and performance metrics
- **memory**: Memory usage statistics
- **disk**: Disk usage and I/O metrics
- **network**: Network interface statistics
- **process**: Process information and metrics
- **command**: Custom command output collection

## Log Sources/Outputs (`logs`)

Log sources and outputs handle log collection and forwarding. Sources tail log files while outputs forward logs to various destinations.

**Note**: Log collection is not yet implemented. This section will be populated when log collection functionality is added.

### Planned Components

#### Sources
- **file**: Tail local log files
- **journald**: Systemd journal collection
- **syslog**: Syslog message collection

#### Outputs
- **file**: Write to local files
- **http**: Forward via HTTP/HTTPS
- **s3**: Store in Amazon S3
- **elasticsearch**: Send to Elasticsearch
- **syslog**: Forward to syslog

## Comprehensive Examples

This section provides complete examples showing how to use Driftless for common configuration management tasks.

### Complete Configuration Example

Here's a complete example showing a typical web server setup:

**YAML Format**:

```yaml
vars:
  web_user: www-data
  web_root: /var/www/html
  nginx_config: /etc/nginx/sites-available/default

tasks:
  # Install required packages
  - type: package
    name: nginx
    state: present

  # Create web directory
  - type: file
    path: "{{ web_root }}"
    state: present
    mode: "0755"
    owner: "{{ web_user }}"
    group: "{{ web_user }}"

  # Configure nginx
  - type: file
    path: "{{ nginx_config }}"
    state: present
    content: |
      server {
          listen 80;
          root {{ web_root }};
          index index.html index.htm;

          location / {
              try_files $uri $uri/ =404;
          }
      }
    mode: "0644"
    owner: root
    group: root

  # Create index page
  - type: file
    path: "{{ web_root }}/index.html"
    state: present
    content: |
      <!DOCTYPE html>
      <html>
      <head><title>Welcome to Driftless</title></head>
      <body><h1>Hello from Driftless!</h1></body>
      </html>
    mode: "0644"
    owner: "{{ web_user }}"
    group: "{{ web_user }}"

  # Start and enable nginx service
  - type: service
    name: nginx
    state: started
    enabled: true
```

**JSON Format**:

```json
{
  "vars": {
    "web_user": "www-data",
    "web_root": "/var/www/html",
    "nginx_config": "/etc/nginx/sites-available/default"
  },
  "tasks": [
    {
      "type": "package",
      "name": "nginx",
      "state": "present"
    },
    {
      "type": "file",
      "path": "{{ web_root }}",
      "state": "present",
      "mode": "0755",
      "owner": "{{ web_user }}",
      "group": "{{ web_user }}"
    },
    {
      "type": "file",
      "path": "{{ nginx_config }}",
      "state": "present",
      "content": "server {\n    listen 80;\n    root {{ web_root }};\n    index index.html index.htm;\n\n    location / {\n        try_files $uri $uri/ =404;\n    }\n}",
      "mode": "0644",
      "owner": "root",
      "group": "root"
    },
    {
      "type": "file",
      "path": "{{ web_root }}/index.html",
      "state": "present",
      "content": "<!DOCTYPE html>\n<html>\n<head><title>Welcome to Driftless</title></head>\n<body><h1>Hello from Driftless!</h1></body>\n</html>",
      "mode": "0644",
      "owner": "{{ web_user }}",
      "group": "{{ web_user }}"
    },
    {
      "type": "service",
      "name": "nginx",
      "state": "started",
      "enabled": true
    }
  ]
}
```

**TOML Format**:

```toml
[vars]
web_user = "www-data"
web_root = "/var/www/html"
nginx_config = "/etc/nginx/sites-available/default"

[[tasks]]
type = "package"
name = "nginx"
state = "present"

[[tasks]]
type = "file"
path = "{{ web_root }}"
state = "present"
mode = "0755"
owner = "{{ web_user }}"
group = "{{ web_user }}"

[[tasks]]
type = "file"
path = "{{ nginx_config }}"
state = "present"
content = """
server {
    listen 80;
    root {{ web_root }};
    index index.html index.htm;

    location / {
    try_files $uri $uri/ =404;
    }
}
"""
mode = "0644"
owner = "root"
group = "root"

[[tasks]]
type = "file"
path = "{{ web_root }}/index.html"
state = "present"
content = """
<!DOCTYPE html>
<html>
<head><title>Welcome to Driftless</title></head>
<body><h1>Hello from Driftless!</h1></body>
</html>
"""
mode = "0644"
owner = "{{ web_user }}"
group = "{{ web_user }}"

[[tasks]]
type = "service"
name = "nginx"
state = "started"
enabled = true
```

