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
//!
//! **Output:**
//! ```yaml
//! interfaces:
//!   - name: "eth0"
//!     bytes_received: 1234567890
//!     bytes_transmitted: 987654321
//!     total_bytes: 2222222211
//!     packets_received: 1234567
//!     packets_transmitted: 987654
//!     total_packets: 2222221
//!     errors_on_received: 0
//!     errors_on_transmitted: 0
//!     total_errors: 0
//!     status: "up"
//!   - name: "lo"
//!     bytes_received: 123456789
//!     bytes_transmitted: 123456789
//!     total_bytes: 246913578
//!     packets_received: 123456
//!     packets_transmitted: 123456
//!     total_packets: 246912
//!     errors_on_received: 0
//!     errors_on_transmitted: 0
//!     total_errors: 0
//!     status: "up"
//! labels:
//!   network_type: corporate
//! ```

use crate::facts::NetworkCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::{Networks, System};

/// Execute network facts collection
pub fn collect_network_facts(collector: &NetworkCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut networks = Networks::new();
    networks.refresh();

    let mut facts = HashMap::new();
    let mut interfaces_info = Vec::new();

    // Iterate over all network interfaces
    for (interface_name, network_data) in networks.list() {
        // Filter by interfaces if specified
        if !collector.interfaces.is_empty() && !collector.interfaces.contains(interface_name) {
            continue;
        }

        let mut interface_info = HashMap::new();

        // Basic interface info
        interface_info.insert("name".to_string(), Value::String(interface_name.clone()));

        // Collect bytes transmitted/received
        if collector.collect.bytes {
            interface_info.insert(
                "bytes_received".to_string(),
                Value::Number(network_data.received().into()),
            );
            interface_info.insert(
                "bytes_transmitted".to_string(),
                Value::Number(network_data.transmitted().into()),
            );
            interface_info.insert(
                "total_bytes".to_string(),
                Value::Number((network_data.received() + network_data.transmitted()).into()),
            );
        }

        // Collect packets transmitted/received
        if collector.collect.packets {
            interface_info.insert(
                "packets_received".to_string(),
                Value::Number(network_data.packets_received().into()),
            );
            interface_info.insert(
                "packets_transmitted".to_string(),
                Value::Number(network_data.packets_transmitted().into()),
            );
            interface_info.insert(
                "total_packets".to_string(),
                Value::Number(
                    (network_data.packets_received() + network_data.packets_transmitted()).into(),
                ),
            );
        }

        // Collect errors and drops
        if collector.collect.errors {
            interface_info.insert(
                "errors_on_received".to_string(),
                Value::Number(network_data.errors_on_received().into()),
            );
            interface_info.insert(
                "errors_on_transmitted".to_string(),
                Value::Number(network_data.errors_on_transmitted().into()),
            );
            interface_info.insert(
                "total_errors".to_string(),
                Value::Number(
                    (network_data.errors_on_received() + network_data.errors_on_transmitted())
                        .into(),
                ),
            );
        }

        // Collect network interface status
        if collector.collect.status {
            // Note: sysinfo doesn't provide direct interface status (up/down)
            // This is a placeholder - in a real implementation, you might need
            // to use platform-specific APIs or external tools
            let is_up =
                network_data.packets_received() > 0 || network_data.packets_transmitted() > 0;
            interface_info.insert(
                "status".to_string(),
                Value::String(if is_up { "up" } else { "unknown" }.to_string()),
            );
        }

        interfaces_info.push(Value::Mapping(
            interface_info
                .into_iter()
                .map(|(k, v)| (Value::String(k), v))
                .collect(),
        ));
    }

    facts.insert("interfaces".to_string(), Value::Sequence(interfaces_info));

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
    use crate::facts::{BaseCollector, NetworkCollectOptions, NetworkCollector};
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

            // Should have interfaces array
            assert!(keys.contains("interfaces"));

            // Check interfaces array
            if let Some(Value::Sequence(interfaces)) =
                map.get(&Value::String("interfaces".to_string()))
            {
                // Should have at least one interface (depending on system)
                // Note: In test environment, this might be empty, so we don't assert length

                // If there are interfaces, validate structure
                for interface in interfaces {
                    if let Value::Mapping(interface_map) = interface {
                        let interface_keys: std::collections::HashSet<_> = interface_map
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
                        assert!(interface_keys.contains("name"));

                        // Should have bytes metrics
                        assert!(interface_keys.contains("bytes_received"));
                        assert!(interface_keys.contains("bytes_transmitted"));
                        assert!(interface_keys.contains("total_bytes"));

                        // Should have packets metrics
                        assert!(interface_keys.contains("packets_received"));
                        assert!(interface_keys.contains("packets_transmitted"));
                        assert!(interface_keys.contains("total_packets"));

                        // Should have error metrics
                        assert!(interface_keys.contains("errors_on_received"));
                        assert!(interface_keys.contains("errors_on_transmitted"));
                        assert!(interface_keys.contains("total_errors"));

                        // Should have status
                        assert!(interface_keys.contains("status"));
                    } else {
                        panic!("Interface entry should be a mapping");
                    }
                }
            } else {
                panic!("interfaces should be a sequence");
            }
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

            assert!(keys.contains("interfaces"));
        } else {
            panic!("Expected mapping value");
        }
    }
}
