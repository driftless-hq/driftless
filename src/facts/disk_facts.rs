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

use crate::facts::DiskCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::System;

/// Execute disk facts collection
pub fn collect_disk_facts(collector: &DiskCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut facts = HashMap::new();
    let mut disks_info = Vec::new();

    // Placeholder disk information
    // In a real implementation, this would iterate over system.disks()
    let placeholder_disk = HashMap::new();
    disks_info.push(Value::Mapping(
        placeholder_disk.into_iter().map(|(k, v)| (Value::String(k), v)).collect()
    ));

    facts.insert("disks".to_string(), Value::Sequence(disks_info));

    // Add base labels if any
    if !collector.base.labels.is_empty() {
        let mut labels = HashMap::new();
        for (key, value) in &collector.base.labels {
            labels.insert(key.clone(), Value::String(value.clone()));
        }
        facts.insert("labels".to_string(), Value::Mapping(
            labels.into_iter().map(|(k, v)| (Value::String(k), v)).collect()
        ));
    }

    Ok(Value::Mapping(
        facts.into_iter().map(|(k, v)| (Value::String(k), v)).collect()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{DiskCollector, BaseCollector, DiskCollectOptions, DiskThresholds};
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
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            // Should have disks array
            assert!(keys.contains("disks"));
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
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
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
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
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
            devices: vec![], // No device filter
            mount_points: vec![], // No mount point filter
            collect: DiskCollectOptions::default(),
            thresholds: DiskThresholds::default(),
        };

        let result = collect_disk_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("disks"));
        } else {
            panic!("Expected mapping value");
        }
    }
}