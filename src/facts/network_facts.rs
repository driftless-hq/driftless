//! Network facts collector
//!
//! Collects network interface statistics and status information.
//!
//! # Examples
//!
//! ## Basic network metrics collection
//!
//! **YAML Format:**
//! ```yaml
//! type: network
//! name: network
//! interfaces: ["eth0", "wlan0"]
//! collect:
//!   bytes: true
//!   packets: true
//!   errors: true
//!   status: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "network",
//!   "name": "network",
//!   "interfaces": ["eth0", "wlan0"],
//!   "collect": {
//!     "bytes": true,
//!     "packets": true,
//!     "errors": true,
//!     "status": true
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[collectors]]
//! type = "network"
//! name = "network"
//! interfaces = ["eth0", "wlan0"]
//!
//! [collectors.collect]
//! bytes = true
//! packets = true
//! errors = true
//! status = true
//! ```

use crate::facts::NetworkCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::System;

/// Execute network facts collection
pub fn collect_network_facts(collector: &NetworkCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut facts = HashMap::new();
    let mut interfaces_info = Vec::new();

    // Placeholder network interface information
    // In a real implementation, this would iterate over system.networks()
    let placeholder_interface = HashMap::new();
    interfaces_info.push(Value::Mapping(
        placeholder_interface.into_iter().map(|(k, v)| (Value::String(k), v)).collect()
    ));

    facts.insert("interfaces".to_string(), Value::Sequence(interfaces_info));

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
    use crate::facts::{NetworkCollector, BaseCollector, NetworkCollectOptions};
    use std::collections::HashMap;

    #[test]
    fn test_collect_network_facts_basic() {
        let collector = NetworkCollector {
            base: BaseCollector {
                name: "network".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            interfaces: vec!["eth0".to_string(), "wlan0".to_string()],
            collect: NetworkCollectOptions {
                bytes: true,
                packets: true,
                errors: true,
                status: true,
            },
        };

        let result = collect_network_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("interfaces"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_network_facts_with_interfaces() {
        let collector = NetworkCollector {
            base: BaseCollector {
                name: "network".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            interfaces: vec!["lo".to_string(), "eth0".to_string(), "docker0".to_string()],
            collect: NetworkCollectOptions::default(),
        };

        let result = collect_network_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("interfaces"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_network_facts_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("network_type".to_string(), "corporate".to_string());

        let collector = NetworkCollector {
            base: BaseCollector {
                name: "network".to_string(),
                enabled: true,
                poll_interval: 60,
                labels,
            },
            interfaces: vec![],
            collect: NetworkCollectOptions::default(),
        };

        let result = collect_network_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("labels"));
            assert!(keys.contains("interfaces"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_network_facts_empty_interfaces() {
        let collector = NetworkCollector {
            base: BaseCollector {
                name: "network".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            interfaces: vec![], // No interface filter
            collect: NetworkCollectOptions::default(),
        };

        let result = collect_network_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("interfaces"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_network_facts_selective_collection() {
        let collector = NetworkCollector {
            base: BaseCollector {
                name: "network".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            interfaces: vec!["eth0".to_string()],
            collect: NetworkCollectOptions {
                bytes: true,
                packets: false,
                errors: true,
                status: false,
            },
        };

        let result = collect_network_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("interfaces"));
        } else {
            panic!("Expected mapping value");
        }
    }
}