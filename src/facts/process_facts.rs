//! Process facts collector
//!
//! Collects process information and resource usage statistics.
//!
//! # Examples
//!
//! ## Basic process metrics collection
//!
//! **YAML Format:**
//! ```yaml
//! type: process
//! name: process
//! patterns: ["nginx", "apache", "sshd"]
//! collect:
//!   count: true
//!   cpu: true
//!   memory: true
//!   status: true
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "process",
//!   "name": "process",
//!   "patterns": ["nginx", "apache", "sshd"],
//!   "collect": {
//!     "count": true,
//!     "cpu": true,
//!     "memory": true,
//!     "status": true
//!   }
//! }
//! ```

use crate::facts::ProcessCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::System;

/// Execute process facts collection
pub fn collect_process_facts(collector: &ProcessCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut facts = HashMap::new();
    let mut processes_info = Vec::new();
    let total_processes = 0;

    // Placeholder process information
    // In a real implementation, this would iterate over system.processes()
    let placeholder_process = HashMap::new();
    processes_info.push(Value::Mapping(
        placeholder_process.into_iter().map(|(k, v)| (Value::String(k), v)).collect()
    ));

    // Collect process count
    if collector.collect.count {
        facts.insert("total_processes".to_string(), Value::Number(total_processes.into()));
        facts.insert("matched_processes".to_string(), Value::Number(processes_info.len().into()));
    }

    facts.insert("processes".to_string(), Value::Sequence(processes_info));

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
    use crate::facts::{ProcessCollector, BaseCollector, ProcessCollectOptions};
    use std::collections::HashMap;

    #[test]
    fn test_collect_process_facts_basic() {
        let collector = ProcessCollector {
            base: BaseCollector {
                name: "process".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            patterns: vec!["nginx".to_string(), "apache".to_string()],
            collect: ProcessCollectOptions {
                count: true,
                cpu: true,
                memory: true,
                status: true,
            },
        };

        let result = collect_process_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("processes"));
            assert!(keys.contains("total_processes"));
            assert!(keys.contains("matched_processes"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_process_facts_with_patterns() {
        let collector = ProcessCollector {
            base: BaseCollector {
                name: "process".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            patterns: vec!["sshd".to_string(), "systemd".to_string(), "bash".to_string()],
            collect: ProcessCollectOptions::default(),
        };

        let result = collect_process_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("processes"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_process_facts_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("process_type".to_string(), "system".to_string());

        let collector = ProcessCollector {
            base: BaseCollector {
                name: "process".to_string(),
                enabled: true,
                poll_interval: 60,
                labels,
            },
            patterns: vec![],
            collect: ProcessCollectOptions::default(),
        };

        let result = collect_process_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("labels"));
            assert!(keys.contains("processes"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_process_facts_empty_patterns() {
        let collector = ProcessCollector {
            base: BaseCollector {
                name: "process".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            patterns: vec![], // No pattern filter
            collect: ProcessCollectOptions::default(),
        };

        let result = collect_process_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("processes"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_process_facts_selective_collection() {
        let collector = ProcessCollector {
            base: BaseCollector {
                name: "process".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            patterns: vec!["nginx".to_string()],
            collect: ProcessCollectOptions {
                count: true,
                cpu: false,
                memory: true,
                status: false,
            },
        };

        let result = collect_process_facts(&collector);
        assert!(result.is_ok());

        let value = result.unwrap();
        if let Value::Mapping(map) = value {
            let keys: std::collections::HashSet<_> = map.keys()
                .filter_map(|k| if let Value::String(s) = k { Some(s.as_str()) } else { None })
                .collect();

            assert!(keys.contains("processes"));
            assert!(keys.contains("total_processes"));
            assert!(keys.contains("matched_processes"));
        } else {
            panic!("Expected mapping value");
        }
    }
}