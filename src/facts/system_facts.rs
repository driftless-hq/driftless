//! System facts collector
//!
//! Collects system information including hostname, OS details, kernel version,
//! uptime, boot time, and CPU architecture.
//!
//! # Examples
//!
//! ## Basic system information collection
//!
//! **YAML Format:**
//! ```yaml
//! type: system
//! name: system
//! collect:
//!   hostname: true
//!   os: true
//!   kernel: true
//!   uptime: true
//!   boot_time: true
//!   arch: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "system",
//!   "name": "system",
//!   "collect": {
//!     "hostname": true,
//!     "os": true,
//!     "kernel": true,
//!     "uptime": true,
//!     "boot_time": true,
//!     "arch": true
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[collectors]]
//! type = "system"
//! name = "system"
//!
//! [collectors.collect]
//! hostname = true
//! os = true
//! kernel = true
//! uptime = true
//! boot_time = true
//! arch = true
//! ```
//!
//! **Output:**
//! ```yaml
//! hostname: "myhost.example.com"
//! os: "linux"
//! os_family: "unix"
//! kernel_version: "5.15.0-91-generic"
//! uptime_seconds: 1234567
//! boot_time: 1706012345
//! cpu_arch: "x86_64"
//! ```

use crate::facts::SystemCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::System;

/// Execute system facts collection
pub fn collect_system_facts(collector: &SystemCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut facts = HashMap::new();

    // Collect hostname
    if collector.collect.hostname {
        if let Ok(hostname) = hostname::get() {
            if let Some(hostname_str) = hostname.to_str() {
                facts.insert(
                    "hostname".to_string(),
                    Value::String(hostname_str.to_string()),
                );
            }
        }
    }

    // Collect OS information
    if collector.collect.os {
        facts.insert(
            "os".to_string(),
            Value::String(std::env::consts::OS.to_string()),
        );
        facts.insert(
            "os_family".to_string(),
            Value::String(std::env::consts::FAMILY.to_string()),
        );
    }

    // Collect kernel version
    if collector.collect.kernel {
        if let Some(kernel_version) = System::kernel_version() {
            facts.insert("kernel_version".to_string(), Value::String(kernel_version));
        } else {
            facts.insert("kernel_version".to_string(), Value::Null);
        }
    }

    // Collect uptime
    if collector.collect.uptime {
        let uptime_seconds = System::uptime();
        facts.insert(
            "uptime_seconds".to_string(),
            Value::Number(serde_yaml::Number::from(uptime_seconds)),
        );
    }

    // Collect boot time
    if collector.collect.boot_time {
        let boot_time = System::boot_time();
        facts.insert(
            "boot_time".to_string(),
            Value::Number(serde_yaml::Number::from(boot_time)),
        );
    }

    // Collect CPU architecture
    if collector.collect.arch {
        facts.insert(
            "cpu_arch".to_string(),
            Value::String(std::env::consts::ARCH.to_string()),
        );
    }

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
    use crate::facts::{BaseCollector, SystemCollectOptions, SystemCollector};
    use std::collections::HashMap;

    #[test]
    fn test_collect_system_facts_basic() {
        let collector = SystemCollector {
            base: BaseCollector {
                name: "system".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collect: SystemCollectOptions {
                hostname: true,
                os: true,
                kernel: true,
                uptime: true,
                boot_time: true,
                arch: true,
            },
        };

        let result = collect_system_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            // Check that expected keys are present
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

            assert!(keys.contains("hostname") || keys.contains("os"));
            assert!(keys.contains("os"));
            assert!(keys.contains("os_family"));
            assert!(keys.contains("cpu_arch"));

            // Check that kernel_version is collected and not null
            if keys.contains("kernel_version") {
                let kernel_value = map
                    .get(Value::String("kernel_version".to_string()))
                    .unwrap();
                assert!(!matches!(kernel_value, Value::Null));
            }

            // Check that uptime_seconds is collected and not null
            if keys.contains("uptime_seconds") {
                let uptime_value = map
                    .get(Value::String("uptime_seconds".to_string()))
                    .unwrap();
                assert!(!matches!(uptime_value, Value::Null));
                assert!(matches!(uptime_value, Value::Number(_)));
            }

            // Check that boot_time is collected and not null
            if keys.contains("boot_time") {
                let boot_value = map.get(Value::String("boot_time".to_string())).unwrap();
                assert!(!matches!(boot_value, Value::Null));
                assert!(matches!(boot_value, Value::Number(_)));
            }
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_system_facts_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "test".to_string());
        labels.insert("datacenter".to_string(), "us-west".to_string());

        let collector = SystemCollector {
            base: BaseCollector {
                name: "system".to_string(),
                enabled: true,
                poll_interval: 60,
                labels,
            },
            collect: SystemCollectOptions {
                hostname: true,
                os: false,
                kernel: false,
                uptime: false,
                boot_time: false,
                arch: false,
            },
        };

        let result = collect_system_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            // Check that labels are included
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
            assert!(keys.contains("hostname"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_system_facts_selective_collection() {
        let collector = SystemCollector {
            base: BaseCollector {
                name: "system".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collect: SystemCollectOptions {
                hostname: false,
                os: true,
                kernel: false,
                uptime: false,
                boot_time: false,
                arch: true,
            },
        };

        let result = collect_system_facts(&collector);
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

            // Should have OS and arch but not hostname
            assert!(keys.contains("os"));
            assert!(keys.contains("os_family"));
            assert!(keys.contains("cpu_arch"));
            assert!(!keys.contains("hostname"));
        } else {
            panic!("Expected mapping value");
        }
    }
}
