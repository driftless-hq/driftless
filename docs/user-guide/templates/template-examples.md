# Enhanced Template System Examples

This document demonstrates the robust template system with variables, filters, and built-in functions.

## Basic Variable Substitution

```yaml
vars:
  user_name: "alice"
  user_count: 42
  config_path: "/etc/myapp/config.yml"

tasks:
  - type: debug
    msg: "Hello {{ user_name }}! There are {{ user_count }} users."

  - type: file
    path: "{{ config_path | dirname }}/backup"
    state: present
```

## Filters

```yaml
vars:
  app_name: "my-application"
  file_path: "/home/user/data.txt"
  description: "hello world example"

tasks:
  - type: debug
    msg: "App name in uppercase: {{ app_name | upper }}"

  - type: debug
    msg: "Filename: {{ file_path | basename }}"

  - type: debug
    msg: "Directory: {{ file_path | dirname }}"

  - type: debug
    msg: "Name length: {{ app_name | length }}"

  - type: debug
    msg: "Capitalized: {{ description | capitalize }}"

  - type: debug
    msg: "Truncated: {{ description | truncate(12) }}"

  - type: debug
    msg: "Truncated with custom end: {{ description | truncate(12, false, '...') }}"
```

## Built-in Functions

```yaml
vars:
  server_list: ["web1", "web2", "db1"]
  config_file: "/etc/nginx/sites-available/default"

tasks:
  - type: debug
    msg: "Server count: {{ length(server_list) }}"

  - type: debug
    msg: "Config filename: {{ basename(config_file) }}"

  - type: debug
    msg: "Config directory: {{ dirname(config_file) }}"
```

## Complex Condition Expressions

```yaml
vars:
  deploy_env: "production"
  server_count: 3
  enable_ssl: true
  regions: ["us-east", "us-west", "eu-central"]

tasks:
  - type: debug
    msg: "Production deployment with SSL"
    when: "{{ deploy_env }} == production and {{ enable_ssl }}"

  - type: debug
    msg: "Multi-region setup"
    when: "{{ length(regions) }} > 1"

  - type: debug
    msg: "Large cluster"
    when: "{{ server_count }} >= 5"

  - type: debug
    msg: "US region included"
    when: "us-east in {{ regions }}"

  - type: fail
    msg: "Cannot deploy to production without SSL"
    when: "{{ deploy_env }} == production and not {{ enable_ssl }}"
```

## Variable Definition Checks

```yaml
tasks:
  - type: assert
    that: "deploy_env is defined"
    success_msg: "Deployment environment is configured"

  - type: fail
    msg: "Required variable 'api_key' is not set"
    when: "api_key is not defined"

  - type: set_fact
    key: "cluster_size"
    value: "{{ server_count | int }}"

  - type: debug
    msg: "Using {{ cluster_size }} servers"
    when: "cluster_size is defined"

## Registered Variables Usage

Registered variables allow you to capture the output of one task and use it in subsequent tasks. This is particularly useful for conditional execution or dynamic configuration based on command results or API responses.

### Command Output Capture

Capture `stdout` and use it in a template.

```yaml
tasks:
  - type: command
    description: "Get uptime"
    command: uptime -p
    register: system_uptime

  - type: debug
    msg: "The system has been up for: {{ system_uptime.stdout }}"
```

### Conditional Execution based on Command Result

Use the exit code (`rc`) to decide whether to run another task.

```yaml
tasks:
  - type: command
    description: "Check if a configuration file is valid"
    command: myapp --check-config /etc/myapp.conf
    register: config_check
    ignore_errors: true

  - type: command
    description: "Apply configuration if valid"
    command: myapp --apply-config /etc/myapp.conf
    when: "{{ config_check.rc }} == 0"

  - type: fail
    msg: "Configuration check failed with error: {{ config_check.stderr }}"
    when: "{{ config_check.rc }} != 0"
```

### API Result Usage

Capture a response from a web service and use its status or content.

```yaml
tasks:
  - type: uri
    description: "Check service health"
    url: https://api.service.local/health
    return_content: true
    register: api_health

  - type: debug
    msg: "Service is healthy. Response status: {{ api_health.status }}"
    when: "{{ api_health.status }} == 200"

  - type: debug
    msg: "Service body: {{ api_health.content }}"
    when: "api_health.content is defined"
```
```

## Template Inheritance and Composition

```yaml
vars:
  app_name: "webapp"
  base_path: "/opt/{{ app_name }}"
  version: "1.2.3"

tasks:
  # Use include_tasks for modular task composition
  - type: include_tasks
    file: "tasks/setup-directories.yml"
    vars:
      app_base: "{{ base_path }}"
      app_version: "{{ version }}"

  - type: include_tasks
    file: "tasks/deploy-{{ deploy_env }}.yml"
    when: "{{ deploy_env }} is defined"
```

## Advanced Expressions

```yaml
vars:
  ports: [80, 443, 8080]
  server_names: ["web", "api", "admin"]
  memory_gb: 16

tasks:
  - type: debug
    msg: "Server has {{ memory_gb }} GB RAM"
    when: "{{ memory_gb | int }} >= 8"

  - type: debug
    msg: "High availability setup"
    when: "{{ length(server_names) }} > 1 and {{ memory_gb }} >= 32"

  - type: set_fact
    key: "is_production"
    value: "{{ deploy_env == 'production' }}"

  - type: assert
    that: "{{ is_production }} or {{ deploy_env }} == 'staging'"
    success_msg: "Valid deployment environment"
```

## Environment Variables and Env Files

Environment variables are accessible through the `env` fact and env files are automatically loaded.

### Direct Access via env

```yaml
tasks:
  - type: debug
    msg: "User: {{ env.USER }}"

  - type: debug
    msg: "Home directory: {{ env.HOME }}"

  - type: debug
    msg: "Path: {{ env.PATH }}"
```

### Env File Support

Create `~/.config/driftless/env` (user) or `/etc/driftless/env` (system-wide) with:

```bash
API_KEY=your-secret-key
DATABASE_PASSWORD=secret-password
APP_ENV=production
```

Then access in templates:

```yaml
vars:
  app_env: "{{ env.APP_ENV }}"
  api_key: "{{ env.API_KEY }}"

tasks:
  - type: debug
    msg: "Running in {{ app_env }} environment"

  - type: file
    path: "/etc/myapp/config.yml"
    content: |
      api_key: "{{ api_key }}"
      database:
        password: "{{ env.DATABASE_PASSWORD }}"

  - type: fail
    msg: "API_KEY environment variable not set"
    when: "api_key == '' or api_key is not defined"
```

### Notes

- Environment variables are loaded from the system, `/etc/driftless/env` (system-wide), and `~/.config/driftless/env` (user)
- Access via `env.VARIABLE_NAME` syntax
- Variables defined in YAML `vars:` section are processed at load time
- Env file variables override system environment variables

## Built-in Facts

The system provides built-in facts:

```yaml
tasks:
  - type: debug
    msg: "Running Driftless version {{ driftless_version }}"

  - type: debug
    msg: "OS Family: {{ os_family }}"

  - type: debug
    msg: "Architecture: {{ driftless_architecture }}"

  - type: debug
    msg: "Distribution: {{ distribution }}"
```

## Complete Example

```yaml
---
vars:
  app_name: "my-web-app"
  deploy_env: "production"
  server_count: 3
  enable_ssl: true
  base_domain: "example.com"
  config_dir: "/etc/{{ app_name }}"

tasks:
  # Validation
  - type: assert
    that: "{{ deploy_env }} in ['development', 'staging', 'production']"
    success_msg: "Valid deployment environment: {{ deploy_env }}"

  - type: assert
    that: "{{ server_count | int }} > 0"
    success_msg: "Server count is valid: {{ server_count }}"

  # Setup
  - type: set_fact
    key: "full_domain"
    value: "{{ app_name }}.{{ base_domain }}"

  - type: set_fact
    key: "is_https"
    value: "{{ enable_ssl and deploy_env == 'production' }}"

  # Directory creation with templating
  - type: directory
    path: "{{ config_dir }}"
    state: present
    mode: "0755"

  - type: directory
    path: "{{ config_dir }}/ssl"
    state: present
    mode: "0700"
    when: "{{ is_https }}"

  # Configuration file
  - type: file
    path: "{{ config_dir }}/app.yml"
    state: present
    content: |
      app:
        name: {{ app_name | upper }}
        environment: {{ deploy_env }}
        servers: {{ server_count }}
        domain: {{ full_domain }}
        ssl_enabled: {{ is_https }}
        config_path: {{ config_dir }}

  # Deployment
  - type: debug
    msg: "Deploying {{ app_name }} to {{ deploy_env }} environment"

  - type: debug
    msg: "Using {{ server_count }} servers for high availability"
    when: "{{ server_count | int }} > 1"

  - type: debug
    msg: "SSL will be configured for {{ full_domain }}"
    when: "{{ is_https }}"

  # Include environment-specific tasks
  - type: include_tasks
    file: "tasks/deploy-{{ deploy_env }}.yml"
    vars:
      app_config: "{{ config_dir }}/app.yml"
      ssl_enabled: "{{ is_https }}"
```

## Using Jinja2 Template Files with Task Chaining

Driftless supports rendering external Jinja2 template files (`.j2` extension) using the `template` task. This allows for complex templating with access to all variables, including outputs from previous tasks, demonstrating the system's ability to chain tasks together.

### Example: Dynamic Configuration Based on System Facts

First, create a template file `nginx.conf.j2` in your configuration directory:

```nginx
# nginx.conf.j2
server {
    listen {{ nginx_port }};
    server_name {{ server_name }};
    
    root {{ web_root }};
    index index.html;
    
    # Dynamic upstream based on registered command output
    upstream app_backend {
        {% for server in app_servers %}
        server {{ server }};
        {% endfor %}
    }
    
    location / {
        proxy_pass http://app_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
    
    # SSL configuration if enabled
    {% if enable_ssl %}
    listen 443 ssl;
    ssl_certificate {{ ssl_cert }};
    ssl_certificate_key {{ ssl_key }};
    {% endif %}
    
    # Custom error page with system info
    error_page 500 502 503 504 /50x.html;
    location = /50x.html {
        root {{ web_root }};
    }
    
    # System uptime from previous command
    add_header X-System-Uptime "{{ system_uptime.stdout | trim }}" always;
}
```

Then, use the following task configuration to render it:

```yaml
vars:
  nginx_port: 80
  server_name: "myapp.example.com"
  web_root: "/var/www/html"
  enable_ssl: false
  ssl_cert: "/etc/ssl/certs/myapp.crt"
  ssl_key: "/etc/ssl/private/myapp.key"

tasks:
  # First task: Gather system information
  - type: command
    description: "Get system uptime"
    command: "uptime -p"
    register: system_uptime

  # Second task: Get list of application servers (simulated)
  - type: command
    description: "Get list of backend servers"
    command: "echo -e '192.168.1.10:8080\n192.168.1.11:8080'"
    register: backend_servers

  # Third task: Process the server list into a variable
  - type: set_fact
    key: "app_servers"
    value: "{{ backend_servers.stdout_lines }}"

  # Fourth task: Render the template using outputs from previous tasks
  - type: template
    description: "Render nginx configuration with dynamic backend"
    src: "nginx.conf.j2"
    dest: "/etc/nginx/sites-available/myapp"
    state: present
    vars:
      nginx_port: "{{ nginx_port }}"
      server_name: "{{ server_name }}"
      web_root: "{{ web_root }}"
      enable_ssl: "{{ enable_ssl }}"
      ssl_cert: "{{ ssl_cert }}"
      ssl_key: "{{ ssl_key }}"
      # app_servers and system_uptime are automatically available

  # Fifth task: Enable the site
  - type: file
    path: "/etc/nginx/sites-enabled/myapp"
    src: "/etc/nginx/sites-available/myapp"
    state: link

  # Sixth task: Reload nginx
  - type: service
    name: nginx
    state: reloaded
```

This example demonstrates:

- **Task Chaining**: The output of the `command` task (registered as `system_uptime`) is used in the template.
- **Variable Processing**: The `set_fact` task processes the command output into a list (`app_servers`) used in the template loop.
- **Complex Templating**: The `.j2` file includes conditionals (`{% if %}`), loops (`{% for %}`), and variable access.
- **Full Integration**: Templates have access to all variables, including those set by previous tasks.

The rendered output would include the actual system uptime in the HTTP header and dynamically configure the upstream servers based on the command output.