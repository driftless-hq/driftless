# Apply Task Types - Ansible Built-in Parity Checklist

This document tracks the implementation status of various task types for the Driftless apply system, aiming for parity with Ansible's built-in modules (ansible.builtin collection).

## Current Implementation Status

### ✅ Implemented (47/47 built-in modules)

#### System Administration
- [x] **user** - User and group management (`UserTask`)
- [x] **service/systemd** - Service management (`ServiceTask`)
- [x] **group** - Group management (`GroupTask`)
- [x] **cron** - Cron job management (`CronTask`)
- [x] **mount** - Mount filesystem (`MountTask`)
- [x] **filesystem** - Create/delete filesystems (`FilesystemTask`)
- [x] **sysctl** - Kernel parameter management (`SysctlTask`)
- [x] **hostname** - System hostname management (`HostnameTask`)
- [x] **timezone** - System timezone management (`TimezoneTask`)
- [x] **reboot** - System reboot (`RebootTask`)
- [x] **shutdown** - System shutdown (`ShutdownTask`)

#### File Operations
- [x] **file** - File/directory operations (`FileTask`)
- [x] **directory** - Directory creation/management (`DirectoryTask`)
- [x] **copy** - Copy files (similar to file with source)
- [x] **template** - Template file rendering
- [x] **lineinfile** - Ensure line in file
- [x] **blockinfile** - Insert/update multi-line blocks
- [x] **replace** - Replace text in files
- [x] **fetch** - Fetch files from remote hosts
- [x] **unarchive** - Unarchive files
- [x] **archive** - Archive files
- [x] **stat** - File/directory statistics

#### Package Management
- [x] **package** - Generic package management (`PackageTask`)
- [x] **apt** - Debian/Ubuntu package management
- [x] **yum/dnf** - RHEL/CentOS/Fedora package management
- [x] **pacman** - Arch Linux package management
- [x] **zypper** - SUSE package management
- [x] **pip** - Python package management
- [x] **npm** - Node.js package management
- [x] **gem** - Ruby gem management

#### Command Execution
- [x] **command/shell** - Command execution (`CommandTask`)
- [x] **script** - Execute local scripts
- [x] **raw** - Execute commands without shell processing

#### Network Operations
- [x] **uri** - Interact with web services
- [x] **get_url** - Download files from HTTP/HTTPS/FTP
- [x] **unarchive** - Extract archives from URLs


#### Utility/Control
- [x] **debug** - Print statements for debugging
- [x] **assert** - Assert given conditions
- [x] **fail** - Fail with custom message
- [x] **wait_for** - Wait for conditions
- [x] **pause** - Pause execution
- [x] **set_fact** - Set facts for later use
- [x] **include_role** - Include roles
- [x] **include_tasks** - Include task files

#### Security & Access
- [x] **authorized_key** - SSH authorized keys
- [x] **sudoers** - Sudo configuration
- [x] **firewalld** - Firewall management
- [x] **ufw** - Ubuntu firewall
- [x] **iptables** - Linux firewall
- [x] **selinux** - SELinux policy management

#### Source Control
- [x] **git** - Git repository management

#### Monitoring & Logging
- [x] **logrotate** - Log rotation configuration
- [x] **rsyslog** - Syslog configuration
- [x] **journald** - systemd journal configuration

## Nix Integration Opportunities

The [nix crate](https://github.com/nix-rust/nix) provides Rust bindings to *nix APIs. We can leverage this for:

### High Priority (System-level operations)
- [ ] **Process management** - Enhanced process monitoring/control
- [ ] **Signal handling** - Send signals to processes
- [ ] **File permissions** - More robust Unix permission handling
- [ ] **User/group operations** - Lower-level user/group management
- [ ] **Mount operations** - Filesystem mounting
- [ ] **Network interfaces** - Network interface management
- [ ] **System information** - Detailed system/hardware info

### Medium Priority (Infrastructure automation)
- [ ] **Sysctl operations** - Kernel parameter management
- [ ] **Capability management** - Linux capabilities
- [ ] **Namespace operations** - Container/namespace management
- [ ] **Cgroup management** - Control groups
- [ ] **Inotify monitoring** - File system monitoring
- [ ] **Socket operations** - Unix domain sockets

### Low Priority (Advanced features)
- [ ] **ACL management** - Access control lists
- [ ] **Extended attributes** - File extended attributes
- [ ] **Audit operations** - System audit logging
- [ ] **KVM operations** - Kernel-based virtual machines

## Implementation Notes

### Task Type Naming Convention
- Use lowercase names matching Ansible module names where possible
- Use descriptive names for complex operations
- Maintain backward compatibility with existing schemas

### Schema Design Principles
- Keep task schemas simple and focused
- Support dry-run operations
- Include proper validation
- Provide sensible defaults
- Allow for platform-specific customizations

### Testing Requirements
- Unit tests for each task type
- Integration tests with real systems (where safe)
- Dry-run validation tests
- Error handling tests
- Cross-platform compatibility tests

### Security Considerations
- Validate file paths to prevent directory traversal
- Sanitize command inputs
- Handle sensitive data appropriately
- Implement proper privilege escalation patterns
