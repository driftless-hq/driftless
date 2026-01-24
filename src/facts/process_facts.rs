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
//!
//! **TOML Format:**
//! ```toml
//! [[collectors]]
//! type = "process"
//! name = "process"
//! patterns = ["nginx", "apache", "sshd"]
//!
//! [collectors.collect]
//! count = true
//! cpu = true
//! memory = true
//! status = true
//! ```
//!
//! **Output:**
//! ```yaml
//! total_processes: 150
//! matched_processes: 3
//! processes:
//!   - pid: 1234
//!     name: "nginx"
//!     cpu_percent: 5
//!     memory_bytes: 104857600
//!     memory_mb: 100
//!     memory_gb: 0
//!     status: "running"
//!     command: "/usr/sbin/nginx"
//!     parent_pid: 1
//!   - pid: 1235
//!     name: "nginx"
//!     cpu_percent: 3
//!     memory_bytes: 52428800
//!     memory_mb: 50
//!     memory_gb: 0
//!     status: "running"
//!     command: "/usr/sbin/nginx"
//!     parent_pid: 1234
//!   - pid: 5678
//!     name: "apache2"
//!     cpu_percent: 2
//!     memory_bytes: 209715200
//!     memory_mb: 200
//!     memory_gb: 0
//!     status: "sleeping"
//!     command: "/usr/sbin/apache2"
//!     parent_pid: 1
//! labels:
//!   process_type: web_servers
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
    let total_processes = system.processes().len();

    // Compile regex patterns for filtering
    let patterns: Vec<regex::Regex> = collector
        .patterns
        .iter()
        .filter_map(|pattern| regex::Regex::new(pattern).ok())
        .collect();

    let mut matched_processes = 0;

    // Iterate over all processes
    for (pid, process) in system.processes() {
        let process_name = process.name().to_string();

        // Filter by patterns if specified
        let matches_pattern = if !patterns.is_empty() {
            patterns
                .iter()
                .any(|pattern| pattern.is_match(&process_name))
        } else {
            true // No patterns means include all processes
        };

        if !matches_pattern {
            continue;
        }

        matched_processes += 1;

        let mut process_info = HashMap::new();

        // Basic process info
        process_info.insert("pid".to_string(), Value::Number(pid.as_u32().into()));
        process_info.insert("name".to_string(), Value::String(process_name.clone()));

        // Collect CPU usage
        if collector.collect.cpu {
            let cpu_usage = process.cpu_usage();
            process_info.insert(
                "cpu_percent".to_string(),
                Value::Number((cpu_usage as i64).into()),
            );
        }

        // Collect memory usage
        if collector.collect.memory {
            let memory_usage = process.memory();
            process_info.insert(
                "memory_bytes".to_string(),
                Value::Number(memory_usage.into()),
            );
            process_info.insert(
                "memory_mb".to_string(),
                Value::Number((memory_usage / 1024 / 1024).into()),
            );
            process_info.insert(
                "memory_gb".to_string(),
                Value::Number((memory_usage / 1024 / 1024 / 1024).into()),
            );
        }

        // Collect process status
        if collector.collect.status {
            let status = match process.status() {
                sysinfo::ProcessStatus::Run => "running",
                sysinfo::ProcessStatus::Sleep => "sleeping",
                sysinfo::ProcessStatus::Stop => "stopped",
                sysinfo::ProcessStatus::Zombie => "zombie",
                sysinfo::ProcessStatus::Tracing => "tracing",
                sysinfo::ProcessStatus::Dead => "dead",
                sysinfo::ProcessStatus::Wakekill => "wakekill",
                sysinfo::ProcessStatus::Waking => "waking",
                sysinfo::ProcessStatus::Parked => "parked",
                sysinfo::ProcessStatus::Idle => "idle",
                _ => "unknown",
            };
            process_info.insert("status".to_string(), Value::String(status.to_string()));

            // Add additional process info
            if let Some(cmd) = process.cmd().first() {
                process_info.insert("command".to_string(), Value::String(cmd.clone()));
            }
            process_info.insert(
                "parent_pid".to_string(),
                Value::Number(process.parent().map(|p| p.as_u32()).unwrap_or(0).into()),
            );
        }

        processes_info.push(Value::Mapping(
            process_info
                .into_iter()
                .map(|(k, v)| (Value::String(k), v))
                .collect(),
        ));
    }

    // Collect process count
    if collector.collect.count {
        facts.insert(
            "total_processes".to_string(),
            Value::Number(total_processes.into()),
        );
        facts.insert(
            "matched_processes".to_string(),
            Value::Number(matched_processes.into()),
        );
    }

    facts.insert("processes".to_string(), Value::Sequence(processes_info));

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
    use crate::facts::{BaseCollector, ProcessCollectOptions, ProcessCollector};
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

            // Should have processes array
            assert!(keys.contains("processes"));
            assert!(keys.contains("total_processes"));
            assert!(keys.contains("matched_processes"));

            // Check processes array
            if let Some(Value::Sequence(processes)) =
                map.get(Value::String("processes".to_string()))
            {
                // If there are processes, validate structure
                for process in processes {
                    if let Value::Mapping(process_map) = process {
                        let process_keys: std::collections::HashSet<_> = process_map
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
                        assert!(process_keys.contains("pid"));
                        assert!(process_keys.contains("name"));

                        // Should have CPU metrics
                        assert!(process_keys.contains("cpu_percent"));

                        // Should have memory metrics
                        assert!(process_keys.contains("memory_bytes"));
                        assert!(process_keys.contains("memory_mb"));
                        assert!(process_keys.contains("memory_gb"));

                        // Should have status
                        assert!(process_keys.contains("status"));
                        assert!(process_keys.contains("command"));
                        assert!(process_keys.contains("parent_pid"));
                    } else {
                        panic!("Process entry should be a mapping");
                    }
                }
            } else {
                panic!("processes should be a sequence");
            }
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
            patterns: vec![
                "sshd".to_string(),
                "systemd".to_string(),
                "bash".to_string(),
            ],
            collect: ProcessCollectOptions::default(),
        };

        let result = collect_process_facts(&collector);
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

            assert!(keys.contains("processes"));
            assert!(keys.contains("total_processes"));
            assert!(keys.contains("matched_processes"));
        } else {
            panic!("Expected mapping value");
        }
    }
}
