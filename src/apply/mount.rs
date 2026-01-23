//! Mount task executor
//!
//! Handles filesystem mounting operations: mount, unmount, fstab management.
//!
//! # Examples
//!
//! ## Mount a filesystem
//!
//! This example mounts a device to a mount point.
//!
//! **YAML Format:**
//! ```yaml
//! - type: mount
//!   description: "Mount data partition"
//!   path: /mnt/data
//!   state: mounted
//!   src: /dev/sdb1
//!   fstype: ext4
//!   opts: ["defaults"]
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "mount",
//!   "description": "Mount data partition",
//!   "path": "/mnt/data",
//!   "state": "mounted",
//!   "src": "/dev/sdb1",
//!   "fstype": "ext4",
//!   "opts": ["defaults"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "mount"
//! description = "Mount data partition"
//! path = "/mnt/data"
//! state = "mounted"
//! src = "/dev/sdb1"
//! fstype = "ext4"
//! opts = ["defaults"]
//! ```
//!
//! ## Mount with fstab entry
//!
//! This example mounts a filesystem and adds it to /etc/fstab for persistence.
//!
//! **YAML Format:**
//! ```yaml
//! - type: mount
//!   description: "Mount NFS share with fstab entry"
//!   path: /mnt/nfs
//!   state: present
//!   src: 192.168.1.100:/export/data
//!   fstype: nfs
//!   opts: ["defaults", "vers=4"]
//!   fstab: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "mount",
//!   "description": "Mount NFS share with fstab entry",
//!   "path": "/mnt/nfs",
//!   "state": "present",
//!   "src": "192.168.1.100:/export/data",
//!   "fstype": "nfs",
//!   "opts": ["defaults", "vers=4"],
//!   "fstab": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "mount"
//! description = "Mount NFS share with fstab entry"
//! path = "/mnt/nfs"
//! state = "present"
//! src = "192.168.1.100:/export/data"
//! fstype = "nfs"
//! opts = ["defaults", "vers=4"]
//! fstab = true
//! ```
//!
//! ## Unmount a filesystem
//!
//! This example unmounts a filesystem.
//!
//! **YAML Format:**
//! ```yaml
//! - type: mount
//!   description: "Unmount temporary mount"
//!   path: /mnt/temp
//!   state: unmounted
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "mount",
//!   "description": "Unmount temporary mount",
//!   "path": "/mnt/temp",
//!   "state": "unmounted"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "mount"
//! description = "Unmount temporary mount"
//! path = "/mnt/temp"
//! state = "unmounted"
//! ```
//!
//! ## Remove fstab entry
//!
//! This example removes a filesystem entry from /etc/fstab.
//!
//! **YAML Format:**
//! ```yaml
//! - type: mount
//!   description: "Remove fstab entry"
//!   path: /mnt/old
//!   state: absent
//!   fstab: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "mount",
//!   "description": "Remove fstab entry",
//!   "path": "/mnt/old",
//!   "state": "absent",
//!   "fstab": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "mount"
//! description = "Remove fstab entry"
//! path = "/mnt/old"
//! state = "absent"
//! fstab = true
//! ```

/// Mount state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountState {
    /// Ensure filesystem is mounted
    Mounted,
    /// Ensure filesystem is not mounted
    Unmounted,
    /// Ensure filesystem is mounted and in fstab
    Present,
    /// Ensure filesystem is not in fstab
    Absent,
}

/// Filesystem mounting task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MountTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Mount point path
    pub path: String,
    /// Mount state
    pub state: MountState,
    /// Device to mount (device path, UUID, LABEL, etc.)
    pub src: String,
    /// Filesystem type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fstype: Option<String>,
    /// Mount options
    #[serde(default)]
    pub opts: Vec<String>,
    /// Whether to update /etc/fstab
    #[serde(default)]
    pub fstab: bool,
    /// Whether to mount recursively
    #[serde(default)]
    pub recursive: bool,
}

use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

/// Execute a mount task
pub async fn execute_mount_task(task: &MountTask, dry_run: bool) -> Result<()> {
    match task.state {
        MountState::Mounted => ensure_mounted(task, dry_run).await,
        MountState::Unmounted => ensure_unmounted(task, dry_run).await,
        MountState::Present => {
            ensure_in_fstab(task, dry_run).await?;
            ensure_mounted(task, dry_run).await
        }
        MountState::Absent => {
            ensure_unmounted(task, dry_run).await?;
            ensure_not_in_fstab(task, dry_run).await
        }
    }
}

/// Ensure filesystem is mounted
async fn ensure_mounted(task: &MountTask, dry_run: bool) -> Result<()> {
    if is_mounted(&task.path)? {
        println!("Filesystem already mounted at: {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would mount {} at {}", task.src, task.path);
        if let Some(fstype) = &task.fstype {
            println!("  Filesystem type: {}", fstype);
        }
        if !task.opts.is_empty() {
            println!("  Options: {}", task.opts.join(","));
        }
    } else {
        mount_filesystem(task)?;
        println!("Mounted {} at {}", task.src, task.path);
    }

    Ok(())
}

/// Ensure filesystem is not mounted
async fn ensure_unmounted(task: &MountTask, dry_run: bool) -> Result<()> {
    if !is_mounted(&task.path)? {
        println!("Filesystem not mounted at: {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would unmount {}", task.path);
    } else {
        unmount_filesystem(&task.path)?;
        println!("Unmounted {}", task.path);
    }

    Ok(())
}

/// Ensure mount is present in fstab
async fn ensure_in_fstab(task: &MountTask, dry_run: bool) -> Result<()> {
    if is_in_fstab(task)? {
        println!("Mount entry already exists in fstab: {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would add mount entry to fstab: {} {}", task.src, task.path);
    } else {
        add_to_fstab(task)?;
        println!("Added mount entry to fstab: {}", task.path);
    }

    Ok(())
}

/// Ensure mount is not present in fstab
async fn ensure_not_in_fstab(task: &MountTask, dry_run: bool) -> Result<()> {
    if !is_in_fstab(task)? {
        println!("Mount entry not in fstab: {}", task.path);
        return Ok(());
    }

    if dry_run {
        println!("Would remove mount entry from fstab: {}", task.path);
    } else {
        remove_from_fstab(task)?;
        println!("Removed mount entry from fstab: {}", task.path);
    }

    Ok(())
}

/// Check if a path is currently mounted
fn is_mounted(path: &str) -> Result<bool> {
    let output = Command::new("mountpoint")
        .arg(path)
        .output()
        .with_context(|| format!("Failed to check if {} is mounted", path))?;

    Ok(output.status.success())
}

/// Mount a filesystem
fn mount_filesystem(task: &MountTask) -> Result<()> {
    let mut cmd = vec!["mount".to_string()];

    if let Some(fstype) = &task.fstype {
        cmd.push("-t".to_string());
        cmd.push(fstype.clone());
    }

    if !task.opts.is_empty() {
        cmd.push("-o".to_string());
        cmd.push(task.opts.join(","));
    }

    if task.recursive {
        cmd.push("-r".to_string());
    }

    cmd.push(task.src.clone());
    cmd.push(task.path.clone());

    run_command(&cmd).with_context(|| format!("Failed to mount {} at {}", task.src, task.path))?;

    Ok(())
}

/// Unmount a filesystem
fn unmount_filesystem(path: &str) -> Result<()> {
    let cmd = vec!["umount".to_string(), path.to_string()];

    run_command(&cmd).with_context(|| format!("Failed to unmount {}", path))?;

    Ok(())
}

/// Check if mount entry exists in fstab
fn is_in_fstab(task: &MountTask) -> Result<bool> {
    let fstab_content =
        fs::read_to_string("/etc/fstab").with_context(|| "Failed to read /etc/fstab")?;

    // Look for line containing both source and path
    for line in fstab_content.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue; // Skip comments
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == task.src && parts[1] == task.path {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Add mount entry to fstab
fn add_to_fstab(task: &MountTask) -> Result<()> {
    let fstab_entry = format!(
        "{} {} {} {} {} {}\n",
        task.src,
        task.path,
        task.fstype.as_deref().unwrap_or("auto"),
        task.opts.join(","),
        "0",
        "2"
    );

    let mut fstab_content =
        fs::read_to_string("/etc/fstab").with_context(|| "Failed to read /etc/fstab")?;

    fstab_content.push_str(&fstab_entry);

    fs::write("/etc/fstab", fstab_content).with_context(|| "Failed to write /etc/fstab")?;

    Ok(())
}

/// Remove mount entry from fstab
fn remove_from_fstab(task: &MountTask) -> Result<()> {
    let fstab_content =
        fs::read_to_string("/etc/fstab").with_context(|| "Failed to read /etc/fstab")?;

    let mut new_content = String::new();

    for line in fstab_content.lines() {
        let line_str = line.trim();
        if line_str.starts_with('#') {
            new_content.push_str(line);
            new_content.push('\n');
            continue;
        }

        let parts: Vec<&str> = line_str.split_whitespace().collect();
        if !(parts.len() >= 2 && parts[0] == task.src && parts[1] == task.path) {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    fs::write("/etc/fstab", new_content).with_context(|| "Failed to write /etc/fstab")?;

    Ok(())
}

/// Run a command and return the result
fn run_command(cmd: &[String]) -> Result<()> {
    if cmd.is_empty() {
        return Ok(());
    }

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .status()
        .with_context(|| format!("Failed to execute command: {}", cmd.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mount_dry_run() {
        let task = MountTask {
            description: None,
            path: "/mnt/test".to_string(),
            state: MountState::Mounted,
            src: "/dev/sda1".to_string(),
            fstype: Some("ext4".to_string()),
            opts: vec!["defaults".to_string()],
            fstab: false,
            recursive: false,
        };

        let result = execute_mount_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unmount_dry_run() {
        let task = MountTask {
            description: None,
            path: "/mnt/test".to_string(),
            state: MountState::Unmounted,
            src: "/dev/sda1".to_string(),
            fstype: None,
            opts: vec![],
            fstab: false,
            recursive: false,
        };

        let result = execute_mount_task(&task, false).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_mounted() {
        // Test with root filesystem (should be mounted)
        let mounted = is_mounted("/");
        assert!(mounted.is_ok());
        // We can't assert the actual value since it depends on the system
    }

    #[test]
    fn test_format_fstab_entry() {
        let _task = MountTask {
            description: None,
            path: "/mnt/data".to_string(),
            state: MountState::Present,
            src: "UUID=1234-5678".to_string(),
            fstype: Some("ext4".to_string()),
            opts: vec!["defaults".to_string(), "noatime".to_string()],
            fstab: true,
            recursive: false,
        };

        // This would create: "UUID=1234-5678 /mnt/data ext4 defaults,noatime 0 2"
        // We can't easily test the fstab functions without mocking the filesystem
    }

    #[tokio::test]
    async fn test_mount_empty_path() {
        let task = MountTask {
            description: None,
            path: "".to_string(), // Empty path
            state: MountState::Mounted,
            src: "/dev/sda1".to_string(),
            fstype: None,
            opts: vec![],
            fstab: false,
            recursive: false,
        };

        let result = execute_mount_task(&task, true).await;
        // Dry-run should succeed even with empty path
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mount_empty_src() {
        let task = MountTask {
            description: None,
            path: "/mnt/test".to_string(),
            state: MountState::Mounted,
            src: "".to_string(), // Empty source
            fstype: None,
            opts: vec![],
            fstab: false,
            recursive: false,
        };

        let result = execute_mount_task(&task, true).await;
        // Dry-run should succeed even with empty source
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mount_invalid_fstype() {
        let task = MountTask {
            description: None,
            path: "/mnt/test".to_string(),
            state: MountState::Mounted,
            src: "/dev/sda1".to_string(),
            fstype: Some("invalid_filesystem_type".to_string()), // Invalid filesystem type
            opts: vec![],
            fstab: false,
            recursive: false,
        };

        let result = execute_mount_task(&task, true).await;
        // Dry-run should succeed even with invalid filesystem type
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unmount_nonexistent_mount() {
        let task = MountTask {
            description: None,
            path: "/mnt/nonexistent_mount_point_12345".to_string(),
            state: MountState::Unmounted,
            src: "/dev/sda1".to_string(),
            fstype: None,
            opts: vec![],
            fstab: false,
            recursive: false,
        };

        let result = execute_mount_task(&task, true).await;
        assert!(result.is_ok()); // Unmounting non-existent mount should succeed
    }
}
