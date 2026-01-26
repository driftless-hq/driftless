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
//! poll_interval: 30
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
//!   "poll_interval": 30,
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
//!
//! **TOML Format:**
//! ```toml
//! [[collectors]]
//! type = "cpu"
//! name = "cpu"
//! poll_interval = 30
//!
//! [collectors.collect]
//! usage = true
//! per_core = true
//! frequency = true
//! temperature = true
//! load_average = true
//!
//! [collectors.thresholds]
//! usage_warning = 80.0
//! usage_critical = 95.0
//! temp_warning = 70.0
//! temp_critical = 85.0
//! ```
//!
//! **Output:**
//! ```text
//! cpu_count: 4
//! usage_percent: 45.2
//! usage_warning: false
//! usage_critical: false
//! cores:
//!   - core_id: 0
//!     usage_percent: 42.1
//!     frequency_mhz: 2400
//!   - core_id: 1
//!     usage_percent: 48.3
//!     frequency_mhz: 2400
//! frequency_mhz: 2400.0
//! temperature_celsius: null
//! temperature_available: false
//! temp_warning: false
//! temp_critical: false
//! load_average:
//!   "1m": 1.25
//!   "5m": 1.15
//!   "15m": 1.08
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

    // Collect overall CPU usage
    if collector.collect.usage {
        system.refresh_cpu();
        let usage = system.global_cpu_info().cpu_usage();
        facts.insert(
            "usage_percent".to_string(),
            Value::Number(serde_yaml::Number::from(usage as f64)),
        );

        // Check thresholds
        if let Some(warning) = collector.thresholds.usage_warning {
            let is_warning = usage as f64 >= warning;
            facts.insert("usage_warning".to_string(), Value::Bool(is_warning));
        }
        if let Some(critical) = collector.thresholds.usage_critical {
            let is_critical = usage as f64 >= critical;
            facts.insert("usage_critical".to_string(), Value::Bool(is_critical));
        }
    }

    // Per-core usage
    if collector.collect.per_core {
        system.refresh_cpu();
        let mut cores = Vec::new();
        for (i, cpu) in system.cpus().iter().enumerate() {
            let mut core_info = HashMap::new();
            core_info.insert("core_id".to_string(), Value::Number(i.into()));
            core_info.insert(
                "usage_percent".to_string(),
                Value::Number(serde_yaml::Number::from(cpu.cpu_usage() as f64)),
            );
            core_info.insert(
                "frequency_mhz".to_string(),
                Value::Number(serde_yaml::Number::from(cpu.frequency())),
            );
            cores.push(Value::Mapping(
                core_info
                    .into_iter()
                    .map(|(k, v)| (Value::String(k), v))
                    .collect(),
            ));
        }
        facts.insert("cores".to_string(), Value::Sequence(cores));
    }

    // CPU frequency (average across all cores)
    if collector.collect.frequency {
        system.refresh_cpu();
        let total_freq: u64 = system.cpus().iter().map(|cpu| cpu.frequency()).sum();
        let avg_freq = total_freq as f64 / system.cpus().len() as f64;
        facts.insert(
            "frequency_mhz".to_string(),
            Value::Number(serde_yaml::Number::from(avg_freq)),
        );
    }

    // CPU temperature (not available in sysinfo crate)
    if collector.collect.temperature {
        facts.insert("temperature_celsius".to_string(), Value::Null);
        facts.insert("temperature_available".to_string(), Value::Bool(false));

        // Threshold checks would be false since no data available
        if let Some(_warning) = collector.thresholds.temp_warning {
            facts.insert("temp_warning".to_string(), Value::Bool(false));
        }
        if let Some(_critical) = collector.thresholds.temp_critical {
            facts.insert("temp_critical".to_string(), Value::Bool(false));
        }
    }

    // Load average
    if collector.collect.load_average {
        let load_avg = System::load_average();
        let mut load_info = HashMap::new();
        load_info.insert(
            "1m".to_string(),
            Value::Number(serde_yaml::Number::from(load_avg.one)),
        );
        load_info.insert(
            "5m".to_string(),
            Value::Number(serde_yaml::Number::from(load_avg.five)),
        );
        load_info.insert(
            "15m".to_string(),
            Value::Number(serde_yaml::Number::from(load_avg.fifteen)),
        );
        facts.insert(
            "load_average".to_string(),
            Value::Mapping(
                load_info
                    .into_iter()
                    .map(|(k, v)| (Value::String(k), v))
                    .collect(),
            ),
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
    use crate::facts::{BaseCollector, CpuCollectOptions, CpuCollector, CpuThresholds};
    use std::collections::HashMap;

    #[test]
    fn test_collect_cpu_facts_basic() {
        let collector = CpuCollector {
            base: BaseCollector {
                name: "cpu".to_string(),
                enabled: true,
                poll_interval: 60,
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

            // Check that we have actual values, not null
            if keys.contains("usage_percent") {
                let usage_value = map.get(Value::String("usage_percent".to_string())).unwrap();
                assert!(!matches!(usage_value, Value::Null));
                assert!(matches!(usage_value, Value::Number(_)));
            }

            if keys.contains("cores") {
                let cores_value = map.get(Value::String("cores".to_string())).unwrap();
                if let Value::Sequence(cores) = cores_value {
                    assert!(!cores.is_empty());
                    // Check first core has proper structure
                    if let Some(Value::Mapping(core_map)) = cores.first() {
                        let core_keys: std::collections::HashSet<_> = core_map
                            .keys()
                            .filter_map(|k| {
                                if let Value::String(s) = k {
                                    Some(s.as_str())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        assert!(core_keys.contains("core_id"));
                        assert!(core_keys.contains("usage_percent"));
                        assert!(core_keys.contains("frequency_mhz"));
                    }
                }
            }

            if keys.contains("frequency_mhz") {
                let freq_value = map.get(Value::String("frequency_mhz".to_string())).unwrap();
                assert!(!matches!(freq_value, Value::Null));
                assert!(matches!(freq_value, Value::Number(_)));
            }

            if keys.contains("load_average") {
                let load_value = map.get(Value::String("load_average".to_string())).unwrap();
                assert!(!matches!(load_value, Value::Null));
                assert!(matches!(load_value, Value::Mapping(_)));
            }
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
                poll_interval: 60,
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
                poll_interval: 60,
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
                poll_interval: 60,
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
