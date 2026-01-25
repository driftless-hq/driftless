//! Filesystem task executor
//!
//! Handles filesystem creation and deletion operations.
//!
//! # Examples
//!
//! ## Create an ext4 filesystem
//!
//! This example creates an ext4 filesystem on a device.
//!
//! **YAML Format:**
//! ```yaml
//! - type: filesystem
//!   description: "Create ext4 filesystem"
//!   dev: /dev/sdb1
//!   state: present
//!   fstype: ext4
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "filesystem",
//!   "description": "Create ext4 filesystem",
//!   "dev": "/dev/sdb1",
//!   "state": "present",
//!   "fstype": "ext4"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "filesystem"
//! description = "Create ext4 filesystem"
//! dev = "/dev/sdb1"
//! state = "present"
//! fstype = "ext4"
//! ```
//!
//! ## Create an XFS filesystem
//!
//! This example creates an XFS filesystem with custom options.
//!
//! **YAML Format:**
//! ```yaml
//! - type: filesystem
//!   description: "Create XFS filesystem"
//!   dev: /dev/sdc1
//!   state: present
//!   fstype: xfs
//!   opts: ["-f", "-i", "size=512"]
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "filesystem",
//!   "description": "Create XFS filesystem",
//!   "dev": "/dev/sdc1",
//!   "state": "present",
//!   "fstype": "xfs",
//!   "opts": ["-f", "-i", "size=512"]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "filesystem"
//! description = "Create XFS filesystem"
//! dev = "/dev/sdc1"
//! state = "present"
//! fstype = "xfs"
//! opts = ["-f", "-i", "size=512"]
//! ```
//!
//! ## Create a Btrfs filesystem
//!
//! This example creates a Btrfs filesystem.
//!
//! **YAML Format:**
//! ```yaml
//! - type: filesystem
//!   description: "Create Btrfs filesystem"
//!   dev: /dev/sdd1
//!   state: present
//!   fstype: btrfs
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "filesystem",
//!   "description": "Create Btrfs filesystem",
//!   "dev": "/dev/sdd1",
//!   "state": "present",
//!   "fstype": "btrfs"
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "filesystem"
//! description = "Create Btrfs filesystem"
//! dev = "/dev/sdd1"
//! state = "present"
//! fstype = "btrfs"
//! ```
//!
//! ## Force create filesystem
//!
//! This example forces the creation of a filesystem (dangerous operation).
//!
//! **YAML Format:**
//! ```yaml
//! - type: filesystem
//!   description: "Force create ext4 filesystem"
//!   dev: /dev/sde1
//!   state: present
//!   fstype: ext4
//!   force: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "filesystem",
//!   "description": "Force create ext4 filesystem",
//!   "dev": "/dev/sde1",
//!   "state": "present",
//!   "fstype": "ext4",
//!   "force": true
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[tasks]]
//! type = "filesystem"
//! description = "Force create ext4 filesystem"
//! dev = "/dev/sde1"
//! state = "present"
//! fstype = "ext4"
//! force = true
//! ```

/// Filesystem state enumeration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemState {
    /// Ensure filesystem exists
    Present,
    /// Ensure filesystem does not exist
    Absent,
}

/// Filesystem creation/deletion task
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilesystemTask {
    /// Optional description of what this task does
    ///
    /// Human-readable description of the task's purpose. Used for documentation
    /// and can be displayed in logs or reports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Device path
    pub dev: String,
    /// Filesystem state
    pub state: FilesystemState,
    /// Filesystem type (ext4, xfs, btrfs, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fstype: Option<String>,
    /// Force filesystem creation (dangerous!)
    #[serde(default)]
    pub force: bool,
    /// Additional mkfs options
    #[serde(default)]
    pub opts: Vec<String>,
}

use anyhow::{Context, Result};
use std::process::Command;

/// Execute a filesystem task
pub async fn execute_filesystem_task(task: &FilesystemTask, dry_run: bool) -> Result<()> {
    match task.state {
        FilesystemState::Present => ensure_filesystem_present(task, dry_run).await,
        FilesystemState::Absent => ensure_filesystem_absent(task, dry_run).await,
    }
}

/// Ensure a filesystem exists on the device
async fn ensure_filesystem_present(task: &FilesystemTask, dry_run: bool) -> Result<()> {
    if has_filesystem(&task.dev)? {
        println!("Filesystem already exists on device: {}", task.dev);

        // Check if it's the correct type
        if let Some(expected_fstype) = &task.fstype {
            let actual_fstype = get_filesystem_type(&task.dev)?;
            if let Some(actual) = actual_fstype {
                if actual != *expected_fstype {
                    if task.force {
                        println!(
                            "Filesystem type mismatch (expected: {}, actual: {}), recreating",
                            expected_fstype, actual
                        );
                        if dry_run {
                            println!(
                                "Would recreate filesystem on {} as {}",
                                task.dev, expected_fstype
                            );
                        } else {
                            remove_filesystem(task)?;
                            create_filesystem(task)?;
                        }
                    } else {
                        return Err(anyhow::anyhow!(
                            "Filesystem type mismatch on {}: expected {}, got {}",
                            task.dev,
                            expected_fstype,
                            actual
                        ));
                    }
                }
            } else {
                return Err(anyhow::anyhow!(
                    "Cannot determine filesystem type on {}",
                    task.dev
                ));
            }
        }

        return Ok(());
    }

    if dry_run {
        println!("Would create filesystem on {}", task.dev);
        if let Some(fstype) = &task.fstype {
            println!("  Type: {}", fstype);
        }
        if !task.opts.is_empty() {
            println!("  Options: {}", task.opts.join(" "));
        }
    } else {
        create_filesystem(task)?;
        println!("Created filesystem on {}", task.dev);
    }

    Ok(())
}

/// Ensure no filesystem exists on the device
async fn ensure_filesystem_absent(task: &FilesystemTask, dry_run: bool) -> Result<()> {
    if !has_filesystem(&task.dev)? {
        println!("No filesystem on device: {}", task.dev);
        return Ok(());
    }

    if dry_run {
        println!("Would remove filesystem from {}", task.dev);
    } else {
        remove_filesystem(task)?;
        println!("Removed filesystem from {}", task.dev);
    }

    Ok(())
}

/// Check if a device has a filesystem
fn has_filesystem(device: &str) -> Result<bool> {
    // Use blkid to check for filesystem
    let output = Command::new("blkid")
        .arg("-o")
        .arg("value")
        .arg("-s")
        .arg("TYPE")
        .arg(device)
        .output()
        .with_context(|| format!("Failed to check filesystem on {}", device))?;

    Ok(output.status.success() && !String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

/// Get the filesystem type of a device
fn get_filesystem_type(device: &str) -> Result<Option<String>> {
    let output = Command::new("blkid")
        .arg("-o")
        .arg("value")
        .arg("-s")
        .arg("TYPE")
        .arg(device)
        .output()
        .with_context(|| format!("Failed to get filesystem type for {}", device))?;

    if output.status.success() {
        let fstype = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if fstype.is_empty() {
            Ok(None)
        } else {
            Ok(Some(fstype))
        }
    } else {
        Ok(None)
    }
}

/// Check if a device is currently mounted
fn is_device_mounted(device: &str) -> Result<bool> {
    // Read /proc/mounts to check if the device is mounted
    let mounts_content = std::fs::read_to_string("/proc/mounts")
        .with_context(|| "Failed to read /proc/mounts")?;

    // Check if the device appears in the mounts
    // /proc/mounts format: device mountpoint fstype options dump pass
    for line in mounts_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == device {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Create a filesystem on a device
fn create_filesystem(task: &FilesystemTask) -> Result<()> {
    let fstype = task.fstype.as_deref().unwrap_or("ext4");
    let mut cmd = vec!["mkfs".to_string(), format!("-t{}", fstype)];

    // Add any additional options
    for opt in &task.opts {
        cmd.push(opt.clone());
    }

    cmd.push(task.dev.clone());

    run_command(&cmd)
        .with_context(|| format!("Failed to create {} filesystem on {}", fstype, task.dev))?;

    Ok(())
}

/// Remove a filesystem from a device (WARNING: This destroys data!)
fn remove_filesystem(task: &FilesystemTask) -> Result<()> {
    println!(
        "WARNING: Removing filesystem from {} - this will destroy all data!",
        task.dev
    );

    // Check if the device is currently mounted
    if is_device_mounted(&task.dev)? {
        return Err(anyhow::anyhow!(
            "Cannot remove filesystem: device {} is currently mounted. Unmount it first.",
            task.dev
        ));
    }

    // Try to use wipefs for more sophisticated filesystem removal
    // wipefs can remove filesystem signatures without destroying all data
    let wipefs_cmd = vec![
        "wipefs".to_string(),
        "-a".to_string(), // Remove all signatures
        task.dev.clone(),
    ];

    match run_command(&wipefs_cmd) {
        Ok(()) => {
            println!("Successfully removed filesystem signatures from {}", task.dev);
            return Ok(());
        }
        Err(_) => {
            // wipefs failed, fall back to zeroing the beginning of the device
            println!("wipefs not available or failed, falling back to dd method");
        }
    }

    // Fallback: Zero out the beginning of the device (more destructive)
    let dd_cmd = vec![
        "dd".to_string(),
        "if=/dev/zero".to_string(),
        format!("of={}", task.dev),
        "bs=1M".to_string(),
        "count=1".to_string(),
    ];

    run_command(&dd_cmd).with_context(|| format!("Failed to wipe filesystem on {}", task.dev))?;

    println!("Zeroed beginning of device {}", task.dev);
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
    async fn test_filesystem_create_dry_run() {
        let task = FilesystemTask {
            description: None,
            dev: "/dev/nonexistent_test_device_12345".to_string(),
            state: FilesystemState::Present,
            fstype: Some("ext4".to_string()),
            force: false,
            opts: vec!["-L".to_string(), "data".to_string()],
        };

        let result = execute_filesystem_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_remove_dry_run() {
        let task = FilesystemTask {
            description: None,
            dev: "/dev/sdb1".to_string(),
            state: FilesystemState::Absent,
            fstype: None,
            force: false,
            opts: vec![],
        };

        let result = execute_filesystem_task(&task, true).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_filesystem() {
        // Test with a device that should exist (like /dev/null or similar)
        // This is tricky to test without actual devices, so we'll just ensure the function doesn't crash
        let result = has_filesystem("/dev/null");
        // We don't assert the result since it depends on the system
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_filesystem_type() {
        // Similar to has_filesystem test
        let result = get_filesystem_type("/dev/null");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_empty_device() {
        let task = FilesystemTask {
            description: None,
            dev: "".to_string(), // Empty device path
            state: FilesystemState::Present,
            fstype: Some("ext4".to_string()),
            force: false,
            opts: vec![],
        };

        let result = execute_filesystem_task(&task, true).await;
        // Empty device should still produce output in dry-run mode
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_invalid_device() {
        let task = FilesystemTask {
            description: None,
            dev: "/dev/nonexistent_device_12345".to_string(), // Device that doesn't exist
            state: FilesystemState::Present,
            fstype: Some("ext4".to_string()),
            force: false,
            opts: vec![],
        };

        let result = execute_filesystem_task(&task, true).await;
        // Dry-run should succeed even with invalid device
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_invalid_fstype() {
        let task = FilesystemTask {
            description: None,
            dev: "/dev/nonexistent_test_device_67890".to_string(),
            state: FilesystemState::Present,
            fstype: Some("invalid_filesystem_type_12345".to_string()), // Invalid filesystem type
            force: false,
            opts: vec![],
        };

        let result = execute_filesystem_task(&task, true).await;
        // Dry-run should succeed even with invalid filesystem type
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filesystem_remove_nonexistent() {
        let task = FilesystemTask {
            description: None,
            dev: "/dev/sda1".to_string(),
            state: FilesystemState::Absent,
            fstype: None,
            force: false,
            opts: vec![],
        };

        let result = execute_filesystem_task(&task, true).await;
        assert!(result.is_ok()); // Removing filesystem from device without one should succeed
    }
}
