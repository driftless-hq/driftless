//! Memory facts collector
//!
//! Collects memory usage statistics including total, used, free, available,
//! and swap memory information.
//!
//! # Examples
//!
//! ## Basic memory metrics collection
//!
//! **YAML Format:**
//! ```yaml
//! type: memory
//! name: memory
//! collect:
//!   total: true
//!   used: true
//!   free: true
//!   available: true
//!   swap: true
//!   percentage: true
//! thresholds:
//!   usage_warning: 85.0
//!   usage_critical: 95.0
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "memory",
//!   "name": "memory",
//!   "collect": {
//!     "total": true,
//!     "used": true,
//!     "free": true,
//!     "available": true,
//!     "swap": true,
//!     "percentage": true
//!   },
//!   "thresholds": {
//!     "usage_warning": 85.0,
//!     "usage_critical": 95.0
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[collectors]]
//! type = "memory"
//! name = "memory"
//!
//! [collectors.collect]
//! total = true
//! used = true
//! free = true
//! available = true
//! swap = true
//! percentage = true
//!
//! [collectors.thresholds]
//! usage_warning = 85.0
//! usage_critical = 95.0
//! ```
//!
//! **Output:**
//! ```yaml
//! total_bytes: 8589934592
//! total_mb: 8192
//! total_gb: 8
//! used_bytes: 4294967296
//! used_mb: 4096
//! used_gb: 4
//! free_bytes: 2147483648
//! free_mb: 2048
//! free_gb: 2
//! available_bytes: 3221225472
//! available_mb: 3072
//! available_gb: 3
//! usage_percent: 50
//! available_percent: 37
//! memory_pressure: "low"
//! swap_total_bytes: 2147483648
//! swap_used_bytes: 536870912
//! swap_free_bytes: 1610612736
//! swap_total_mb: 2048
//! swap_used_mb: 512
//! swap_free_mb: 1536
//! swap_usage_percent: 25
//! swap_pressure: "low"
//! usage_warning: false
//! usage_critical: false
//! ```

use crate::facts::MemoryCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::System;

/// Execute memory facts collection
pub fn collect_memory_facts(collector: &MemoryCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut facts = HashMap::new();

    // Get memory information (simplified placeholders)
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let available_memory = system.available_memory();

    // Calculate percentages
    let memory_usage_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    // Collect total memory
    if collector.collect.total {
        facts.insert(
            "total_bytes".to_string(),
            Value::Number(total_memory.into()),
        );
        facts.insert(
            "total_mb".to_string(),
            Value::Number((total_memory / 1024 / 1024).into()),
        );
        facts.insert(
            "total_gb".to_string(),
            Value::Number((total_memory / 1024 / 1024 / 1024).into()),
        );
    }

    // Collect used memory
    if collector.collect.used {
        facts.insert("used_bytes".to_string(), Value::Number(used_memory.into()));
        facts.insert(
            "used_mb".to_string(),
            Value::Number((used_memory / 1024 / 1024).into()),
        );
        facts.insert(
            "used_gb".to_string(),
            Value::Number((used_memory / 1024 / 1024 / 1024).into()),
        );
    }

    // Collect free memory
    if collector.collect.free {
        let free_memory = total_memory - used_memory;
        facts.insert("free_bytes".to_string(), Value::Number(free_memory.into()));
        facts.insert(
            "free_mb".to_string(),
            Value::Number((free_memory / 1024 / 1024).into()),
        );
        facts.insert(
            "free_gb".to_string(),
            Value::Number((free_memory / 1024 / 1024 / 1024).into()),
        );
    }

    // Collect available memory
    if collector.collect.available {
        facts.insert(
            "available_bytes".to_string(),
            Value::Number(available_memory.into()),
        );
        facts.insert(
            "available_mb".to_string(),
            Value::Number((available_memory / 1024 / 1024).into()),
        );
        facts.insert(
            "available_gb".to_string(),
            Value::Number((available_memory / 1024 / 1024 / 1024).into()),
        );
    }

    // Collect swap information
    if collector.collect.swap {
        let total_swap = system.total_swap();
        let used_swap = system.used_swap();
        let free_swap = total_swap.saturating_sub(used_swap);

        facts.insert(
            "swap_total_bytes".to_string(),
            Value::Number(total_swap.into()),
        );
        facts.insert(
            "swap_used_bytes".to_string(),
            Value::Number(used_swap.into()),
        );
        facts.insert(
            "swap_free_bytes".to_string(),
            Value::Number(free_swap.into()),
        );

        facts.insert(
            "swap_total_mb".to_string(),
            Value::Number((total_swap / 1024 / 1024).into()),
        );
        facts.insert(
            "swap_used_mb".to_string(),
            Value::Number((used_swap / 1024 / 1024).into()),
        );
        facts.insert(
            "swap_free_mb".to_string(),
            Value::Number((free_swap / 1024 / 1024).into()),
        );

        // Calculate swap usage percentage
        let swap_usage_percent = if total_swap > 0 {
            (used_swap as f64 / total_swap as f64) * 100.0
        } else {
            0.0
        };
        facts.insert(
            "swap_usage_percent".to_string(),
            Value::Number((swap_usage_percent as i64).into()),
        );

        // Swap pressure monitoring
        let swap_pressure = if swap_usage_percent > 90.0 {
            "critical"
        } else if swap_usage_percent > 75.0 {
            "high"
        } else if swap_usage_percent > 50.0 {
            "medium"
        } else {
            "low"
        };
        facts.insert(
            "swap_pressure".to_string(),
            Value::String(swap_pressure.to_string()),
        );
    }

    // Collect usage percentage
    if collector.collect.percentage {
        facts.insert(
            "usage_percent".to_string(),
            Value::Number((memory_usage_percent as i64).into()),
        );

        // Calculate available memory percentage
        let available_percent = if total_memory > 0 {
            (available_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };
        facts.insert(
            "available_percent".to_string(),
            Value::Number((available_percent as i64).into()),
        );

        // Memory pressure monitoring
        let memory_pressure = if available_percent < 10.0 {
            "critical"
        } else if available_percent < 20.0 {
            "high"
        } else if available_percent < 30.0 {
            "medium"
        } else {
            "low"
        };
        facts.insert(
            "memory_pressure".to_string(),
            Value::String(memory_pressure.to_string()),
        );

        // Check thresholds
        if let Some(warning) = collector.thresholds.usage_warning {
            facts.insert(
                "usage_warning".to_string(),
                Value::Bool(memory_usage_percent >= warning),
            );
        }
        if let Some(critical) = collector.thresholds.usage_critical {
            facts.insert(
                "usage_critical".to_string(),
                Value::Bool(memory_usage_percent >= critical),
            );
        }
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
    use crate::facts::{BaseCollector, MemoryCollectOptions, MemoryCollector, MemoryThresholds};
    use std::collections::HashMap;

    #[test]
    fn test_collect_memory_facts_basic() {
        let collector = MemoryCollector {
            base: BaseCollector {
                name: "memory".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collect: MemoryCollectOptions {
                total: true,
                used: true,
                free: true,
                available: true,
                swap: true,
                percentage: true,
            },
            thresholds: MemoryThresholds {
                usage_warning: Some(85.0),
                usage_critical: Some(95.0),
            },
        };

        let result = collect_memory_facts(&collector);
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

            // Check for expected memory fields
            assert!(keys.contains("total_bytes"));
            assert!(keys.contains("used_bytes"));
            assert!(keys.contains("free_bytes"));
            assert!(keys.contains("available_bytes"));
            assert!(keys.contains("usage_percent"));
            assert!(keys.contains("available_percent"));
            assert!(keys.contains("memory_pressure"));

            // Check for swap fields
            assert!(keys.contains("swap_total_bytes"));
            assert!(keys.contains("swap_used_bytes"));
            assert!(keys.contains("swap_free_bytes"));
            assert!(keys.contains("swap_usage_percent"));
            assert!(keys.contains("swap_pressure"));

            // Check for threshold fields
            assert!(keys.contains("usage_warning"));
            assert!(keys.contains("usage_critical"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_memory_facts_selective_collection() {
        let collector = MemoryCollector {
            base: BaseCollector {
                name: "memory".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collect: MemoryCollectOptions {
                total: true,
                used: false,
                free: true,
                available: false,
                swap: false,
                percentage: true,
            },
            thresholds: MemoryThresholds::default(),
        };

        let result = collect_memory_facts(&collector);
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

            // Should have total and free but not used or available
            assert!(keys.contains("total_bytes"));
            assert!(keys.contains("free_bytes"));
            assert!(!keys.contains("used_bytes"));
            assert!(!keys.contains("available_bytes"));
            assert!(keys.contains("usage_percent"));
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_memory_facts_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("memory_type".to_string(), "system".to_string());

        let collector = MemoryCollector {
            base: BaseCollector {
                name: "memory".to_string(),
                enabled: true,
                poll_interval: 60,
                labels,
            },
            collect: MemoryCollectOptions::default(),
            thresholds: MemoryThresholds::default(),
        };

        let result = collect_memory_facts(&collector);
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
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_memory_facts_thresholds() {
        let collector = MemoryCollector {
            base: BaseCollector {
                name: "memory".to_string(),
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collect: MemoryCollectOptions {
                total: false,
                used: false,
                free: false,
                available: false,
                swap: false,
                percentage: true,
            },
            thresholds: MemoryThresholds {
                usage_warning: Some(50.0), // Low threshold to ensure it triggers
                usage_critical: Some(95.0),
            },
        };

        let result = collect_memory_facts(&collector);
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

            // Should have threshold fields
            assert!(keys.contains("usage_warning"));
            assert!(keys.contains("usage_critical"));
            assert!(keys.contains("usage_percent"));
        } else {
            panic!("Expected mapping value");
        }
    }
}
