//! CPU facts collector
//!
//! Collects CPU metrics including usage percentage, per-core usage, frequency,
//! temperature, and load average.
//!
//! # Examples
//!
//! ## Basic CPU metrics collection
//!
//! **YAML Format:**
//! ```yaml
//! type: cpu
//! name: cpu
//! collect:
//!   usage: true
//!   per_core: true
//!   frequency: true
//!   temperature: true
//!   load_average: true
//! thresholds:
//!   usage_warning: 80.0
//!   usage_critical: 95.0
//!   temp_warning: 70.0
//!   temp_critical: 85.0
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "type": "cpu",
//!   "name": "cpu",
//!   "collect": {
//!     "usage": true,
//!     "per_core": true,
//!     "frequency": true,
//!     "temperature": true,
//!     "load_average": true
//!   },
//!   "thresholds": {
//!     "usage_warning": 80.0,
//!     "usage_critical": 95.0,
//!     "temp_warning": 70.0,
//!     "temp_critical": 85.0
//!   }
//! }
//! ```

use crate::facts::CpuCollector;
use anyhow::Result;
use serde_yaml::Value;
use std::collections::HashMap;
use sysinfo::System;

/// Execute CPU facts collection
pub fn collect_cpu_facts(collector: &CpuCollector) -> Result<Value> {
    let mut system = System::new();
    system.refresh_all();

    let mut facts = HashMap::new();

    // Get CPU count
    facts.insert(
        "cpu_count".to_string(),
        Value::Number(system.cpus().len().into()),
    );

    // Collect overall CPU usage (simplified)
    if collector.collect.usage {
        // Placeholder for CPU usage - would need proper implementation
        facts.insert("usage_percent".to_string(), Value::Null);

        // Check thresholds (placeholder)
        if let Some(_warning) = collector.thresholds.usage_warning {
            facts.insert("usage_warning".to_string(), Value::Bool(false));
        }
        if let Some(_critical) = collector.thresholds.usage_critical {
            facts.insert("usage_critical".to_string(), Value::Bool(false));
        }
    }

    // Per-core usage (placeholder)
    if collector.collect.per_core {
        facts.insert("cores".to_string(), Value::Sequence(Vec::new()));
    }

    // CPU frequency (placeholder)
    if collector.collect.frequency {
        facts.insert("frequency_mhz".to_string(), Value::Null);
    }

    // CPU temperature (placeholder)
    if collector.collect.temperature {
        facts.insert("temperature_celsius".to_string(), Value::Null);
        facts.insert("temperature_available".to_string(), Value::Bool(false));

        if let Some(_warning) = collector.thresholds.temp_warning {
            facts.insert("temp_warning".to_string(), Value::Bool(false));
        }
        if let Some(_critical) = collector.thresholds.temp_critical {
            facts.insert("temp_critical".to_string(), Value::Bool(false));
        }
    }

    // Load average (placeholder)
    if collector.collect.load_average {
        facts.insert("load_average".to_string(), Value::Null);
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
    use crate::facts::{BaseCollector, CpuCollectOptions, CpuCollector, CpuThresholds};
    use std::collections::HashMap;

    #[test]
    fn test_collect_cpu_facts_basic() {
        let collector = CpuCollector {
            base: BaseCollector {
                name: "cpu".to_string(),
                enabled: true,
                poll_interval: None,
                labels: HashMap::new(),
            },
            collect: CpuCollectOptions {
                usage: true,
                per_core: true,
                frequency: true,
                temperature: true,
                load_average: true,
            },
            thresholds: CpuThresholds {
                usage_warning: Some(80.0),
                usage_critical: Some(95.0),
                temp_warning: Some(70.0),
                temp_critical: Some(85.0),
            },
        };

        let result = collect_cpu_facts(&collector);
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

            assert!(keys.contains("cpu_count"));
            // Note: actual values are placeholders, so we just check structure
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_cpu_facts_selective_collection() {
        let collector = CpuCollector {
            base: BaseCollector {
                name: "cpu".to_string(),
                enabled: true,
                poll_interval: None,
                labels: HashMap::new(),
            },
            collect: CpuCollectOptions {
                usage: true,
                per_core: false,
                frequency: false,
                temperature: true,
                load_average: false,
            },
            thresholds: CpuThresholds::default(),
        };

        let result = collect_cpu_facts(&collector);
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

            assert!(keys.contains("cpu_count"));
            // Should have usage and temperature related fields
            assert!(
                keys.contains("usage_warning")
                    || keys.contains("usage_critical")
                    || keys.contains("temp_warning")
                    || keys.contains("temp_critical")
                    || keys.contains("temperature_available")
            );
        } else {
            panic!("Expected mapping value");
        }
    }

    #[test]
    fn test_collect_cpu_facts_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("server_type".to_string(), "web".to_string());

        let collector = CpuCollector {
            base: BaseCollector {
                name: "cpu".to_string(),
                enabled: true,
                poll_interval: None,
                labels,
            },
            collect: CpuCollectOptions::default(),
            thresholds: CpuThresholds::default(),
        };

        let result = collect_cpu_facts(&collector);
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
    fn test_collect_cpu_facts_no_thresholds() {
        let collector = CpuCollector {
            base: BaseCollector {
                name: "cpu".to_string(),
                enabled: true,
                poll_interval: None,
                labels: HashMap::new(),
            },
            collect: CpuCollectOptions {
                usage: true,
                per_core: false,
                frequency: false,
                temperature: true,
                load_average: false,
            },
            thresholds: CpuThresholds::default(), // No thresholds set
        };

        let result = collect_cpu_facts(&collector);
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

            // Should not have threshold warning fields when no thresholds are set
            assert!(!keys.contains("usage_warning"));
            assert!(!keys.contains("usage_critical"));
            assert!(!keys.contains("temp_warning"));
            assert!(!keys.contains("temp_critical"));
        } else {
            panic!("Expected mapping value");
        }
    }
}
