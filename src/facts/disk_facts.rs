//! Disk facts collector
//!
//! Collects disk space and I/O statistics for mounted filesystems.
//!
//! # Examples
//!
//! ## Basic disk metrics collection
//!
//! **YAML Format:**
//! ```yaml
//! type: disk
//! name: disk
//! devices: ["/dev/sda", "/dev/sdb"]
//! mount_points: ["/", "/home", "/var"]
//! collect:
//!   total: true
//!   used: true
//!   free: true
//!   available: true
//!   percentage: true
//!   io: true
//! thresholds:
//!   usage_warning: 80.0
//!   usage_critical: 90.0
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "disk",
//!   "name": "disk",
//!   "devices": ["/dev/sda", "/dev/sdb"],
//!   "mount_points": ["/", "/home", "/var"],
//!   "collect": {
//!     "total": true,
//!     "used": true,
//!     "free": true,
//!     "available": true,
//!     "percentage": true,
//!     "io": true
//!   },
//!   "thresholds": {
//!     "usage_warning": 80.0,
//!     "usage_critical": 90.0
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[collectors]]
//! type = "disk"
//! name = "disk"
//! devices = ["/dev/sda", "/dev/sdb"]
//! mount_points = ["/", "/home", "/var"]
//!
//! [collectors.collect]
//! total = true
//! used = true
//! free = true
//! available = true
//! percentage = true
//! io = true
//!
//! [collectors.thresholds]
//! usage_warning = 80.0
//! usage_critical = 90.0
//! ```
//!
//! **Output:**
//! ```yaml
//! disks:
//!   - device: "/dev/sda1"
//!     mount_point: "/"
//!     is_removable: false
//!     total_bytes: 536870912000
//!     total_mb: 512000
//!     total_gb: 500
//!     used_bytes: 268435456000
//!     used_mb: 256000
//!     used_gb: 250
//!     free_bytes: 134217728000
//!     free_mb: 128000
//!     free_gb: 125
//!     available_bytes: 107374182400
//!     available_mb: 102400
//!     available_gb: 100
//!     usage_percent: 50
//!     available_percent: 20
//!     disk_pressure: "medium"
//!     usage_warning: false
//!     usage_critical: false
//!     io_supported: false
//! labels:
//!   storage_type: ssd
//! ```

use crate::facts::DiskCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::{Disks, System};

/// Execute disk facts collection
pub fn collect_disk_facts(collector: &DiskCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut disks = Disks::new();
    disks.refresh();

    let mut facts = HashMap::new();
    let mut disks_info = Vec::new();

    // Iterate over all disks
    for disk in disks.list() {
        let disk_name = disk.name().to_string_lossy().to_string();
        let mount_point = disk.mount_point().to_string_lossy().to_string();

        // Filter by devices if specified
        if !collector.devices.is_empty() && !collector.devices.contains(&disk_name) {
            continue;
        }

        // Filter by mount points if specified
        if !collector.mount_points.is_empty() && !collector.mount_points.contains(&mount_point) {
            continue;
        }

        let mut disk_info = HashMap::new();

        // Basic disk info
        disk_info.insert("device".to_string(), Value::String(disk_name.clone()));
        disk_info.insert(
            "mount_point".to_string(),
            Value::String(mount_point.clone()),
        );
        disk_info.insert("is_removable".to_string(), Value::Bool(disk.is_removable()));

        let total_space = disk.total_space();
        let available_space = disk.available_space();
        let used_space = total_space.saturating_sub(available_space);

        // Collect total space
        if collector.collect.total {
            disk_info.insert("total_bytes".to_string(), Value::Number(total_space.into()));
            disk_info.insert(
                "total_mb".to_string(),
                Value::Number((total_space / 1024 / 1024).into()),
            );
            disk_info.insert(
                "total_gb".to_string(),
                Value::Number((total_space / 1024 / 1024 / 1024).into()),
            );
        }

        // Collect used space
        if collector.collect.used {
            disk_info.insert("used_bytes".to_string(), Value::Number(used_space.into()));
            disk_info.insert(
                "used_mb".to_string(),
                Value::Number((used_space / 1024 / 1024).into()),
            );
            disk_info.insert(
                "used_gb".to_string(),
                Value::Number((used_space / 1024 / 1024 / 1024).into()),
            );
        }

        // Collect free space (total - used)
        if collector.collect.free {
            let free_space = total_space.saturating_sub(used_space);
            disk_info.insert("free_bytes".to_string(), Value::Number(free_space.into()));
            disk_info.insert(
                "free_mb".to_string(),
                Value::Number((free_space / 1024 / 1024).into()),
            );
            disk_info.insert(
                "free_gb".to_string(),
                Value::Number((free_space / 1024 / 1024 / 1024).into()),
            );
        }

        // Collect available space
        if collector.collect.available {
            disk_info.insert(
                "available_bytes".to_string(),
                Value::Number(available_space.into()),
            );
            disk_info.insert(
                "available_mb".to_string(),
                Value::Number((available_space / 1024 / 1024).into()),
            );
            disk_info.insert(
                "available_gb".to_string(),
                Value::Number((available_space / 1024 / 1024 / 1024).into()),
            );
        }

        // Collect usage percentage and pressure monitoring
        if collector.collect.percentage {
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };
            disk_info.insert(
                "usage_percent".to_string(),
                Value::Number((usage_percent as i64).into()),
            );

            let available_percent = if total_space > 0 {
                (available_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };
            disk_info.insert(
                "available_percent".to_string(),
                Value::Number((available_percent as i64).into()),
            );

            // Disk pressure monitoring based on available space
            let disk_pressure = if available_percent < 5.0 {
                "critical"
            } else if available_percent < 10.0 {
                "high"
            } else if available_percent < 20.0 {
                "medium"
            } else {
                "low"
            };
            disk_info.insert(
                "disk_pressure".to_string(),
                Value::String(disk_pressure.to_string()),
            );

            // Check thresholds
            if let Some(warning) = collector.thresholds.usage_warning {
                disk_info.insert(
                    "usage_warning".to_string(),
                    Value::Bool(usage_percent >= warning),
                );
            }
            if let Some(critical) = collector.thresholds.usage_critical {
                disk_info.insert(
                    "usage_critical".to_string(),
                    Value::Bool(usage_percent >= critical),
                );
            }
        }

        // Collect I/O statistics if available
        if collector.collect.io {
            // Note: sysinfo 0.30 may not have I/O stats, this is a placeholder
            // In future versions, this might include read/write bytes, operations, etc.
            disk_info.insert("io_supported".to_string(), Value::Bool(false));
        }

        disks_info.push(Value::Mapping(
            disk_info
                .into_iter()
                .map(|(k, v)| (Value::String(k), v))
                .collect(),
        ));
    }

    facts.insert("disks".to_string(), Value::Sequence(disks_info));

    // Add base labels if any
    if !collector.base.labels.is_empty() {
        let mut labels = HashMap::new();
        for (key, value) in &collector.base.labels {
            labels.insert(key.clone(), Value::String(value.clone()));
        }
        facts.insert(
            "labels".to_string(),
            Value::Mapping(
                labels
                    .into_iter()
                    .map(|(k, v)| (Value::String(k), v))
                    .collect(),
            ),
        );
    }

    Ok(Value::Mapping(
        facts
            .into_iter()
            .map(|(k, v)| (Value::String(k), v))
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{BaseCollector, DiskCollectOptions, DiskCollector, DiskThresholds};
    use std::collections::HashMap;

    #[test]
    fn test_collect_disk_facts_basic() {
        let collector = DiskCollector {
            base: BaseCollector {
                name: "disk".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            devices: vec!["/dev/sda".to_string()],
            mount_points: vec!["/".to_string(), "/home".to_string()],
            collect: DiskCollectOptions {
                total: true,
                used: true,
                free: true,
                available: true,
                percentage: true,
                io: true,
            },
            thresholds: DiskThresholds {
                usage_warning: Some(80.0),
                usage_critical: Some(90.0),
            },
        };

        let result = collect_disk_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map
                .keys()
                .filter_map(|k| {
                    if let Value::String(s) = k {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            // Should have disks array
            assert!(keys.contains("disks"));

            // Check disks array
            if let Some(Value::Sequence(disks)) = map.get(&Value::String("disks".to_string())) {
                // Should have at least one disk (depending on system)
                // Note: In test environment, this might be empty, so we don't assert length

                // If there are disks, validate structure
                for disk in disks {
                    if let Value::Mapping(disk_map) = disk {
                        let disk_keys: std::collections::HashSet<_> = disk_map
                            .keys()
                            .filter_map(|k| {
                                if let Value::String(s) = k {
                                    Some(s.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect();

                        // Should have basic fields
                        assert!(disk_keys.contains("device"));
                        assert!(disk_keys.contains("mount_point"));
                        assert!(disk_keys.contains("is_removable"));

                        // Should have space metrics
                        assert!(disk_keys.contains("total_bytes"));
                        assert!(disk_keys.contains("used_bytes"));
                        assert!(disk_keys.contains("free_bytes"));
                        assert!(disk_keys.contains("available_bytes"));

                        // Should have percentage metrics
                        assert!(disk_keys.contains("usage_percent"));
                        assert!(disk_keys.contains("available_percent"));
                        assert!(disk_keys.contains("disk_pressure"));

                        // Should have threshold checks
                        assert!(disk_keys.contains("usage_warning"));
                        assert!(disk_keys.contains("usage_critical"));

                        // Should have I/O info
                        assert!(disk_keys.contains("io_supported"));
                    } else {
                        panic!("Disk entry should be a mapping");
                    }
                }
            } else {
                panic!("disks should be a sequence");
            }
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_disk_facts_with_devices_and_mounts() {
        let collector = DiskCollector {
            base: BaseCollector {
                name: "disk".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            devices: vec!["/dev/sda".to_string(), "/dev/sdb".to_string()],
            mount_points: vec!["/".to_string(), "/var".to_string(), "/tmp".to_string()],
            collect: DiskCollectOptions::default(),
            thresholds: DiskThresholds::default(),
        };

        let result = collect_disk_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map
                .keys()
                .filter_map(|k| {
                    if let Value::String(s) = k {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(keys.contains("disks"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_disk_facts_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("storage_type".to_string(), "ssd".to_string());

        let collector = DiskCollector {
            base: BaseCollector {
                name: "disk".to_string(),
                enabled: true,
                poll_interval: 60,
                labels,
            },
            devices: vec![],
            mount_points: vec![],
            collect: DiskCollectOptions::default(),
            thresholds: DiskThresholds::default(),
        };

        let result = collect_disk_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map
                .keys()
                .filter_map(|k| {
                    if let Value::String(s) = k {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(keys.contains("labels"));
            assert!(keys.contains("disks"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_disk_facts_empty_filters() {
        let collector = DiskCollector {
            base: BaseCollector {
                name: "disk".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            devices: vec![],      // No device filter
            mount_points: vec![], // No mount point filter
            collect: DiskCollectOptions::default(),
            thresholds: DiskThresholds::default(),
        };

        let result = collect_disk_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map
                .keys()
                .filter_map(|k| {
                    if let Value::String(s) = k {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(keys.contains("disks"));
        } else {
            panic!("Expected mapping value");
        }
    }
}
