//! Facts orchestrator
//!
//! This module provides the main orchestration logic for facts collection,
//! including scheduling, aggregation, and export capabilities.
//!
//! # Examples
//!
//! ## Basic facts orchestration
//!
//! **YAML Format:**
//! ```yaml
//! global:
//!   enabled: true
//!   poll_interval: 60
//! collectors:
//!   - type: cpu
//!     name: cpu_metrics
//!     collect:
//!       usage: true
//!       per_core: true
//!   - type: memory
//!     name: memory_metrics
//!     collect:
//!       total: true
//!       used: true
//! export:
//!   prometheus:
//!     enabled: true
//!     port: 9090
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "global": {
//!     "enabled": true,
//!     "poll_interval": 60
//!   },
//!   "collectors": [
//!     {
//!       "type": "cpu",
//!       "name": "cpu_metrics",
//!       "collect": {
//!         "usage": true,
//!         "per_core": true
//!       }
//!     },
//!     {
//!       "type": "memory",
//!       "name": "memory_metrics",
//!       "collect": {
//!         "total": true,
//!         "used": true
//!       }
//!     }
//!   ],
//!   "export": {
//!     "prometheus": {
//!       "enabled": true,
//!       "port": 9090
//!     }
//!   }
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [global]
//! enabled = true
//! poll_interval = 60
//!
//! [[collectors]]
//! type = "cpu"
//! name = "cpu_metrics"
//!
//! [collectors.collect]
//! usage = true
//! per_core = true
//!
//! [[collectors]]
//! type = "memory"
//! name = "memory_metrics"
//!
//! [collectors.collect]
//! total = true
//! used = true
//!
//! [export.prometheus]
//! enabled = true
//! port = 9090
//! ```

use crate::facts::{Collector, FactsConfig, FactsRegistry, PrometheusExport};
use crate::plugins::PluginManager;
use anyhow::Result;
use serde_yaml::{self, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock as StdRwLock;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Facts orchestrator for managing collection, aggregation, and export
#[allow(dead_code)]
pub struct FactsOrchestrator {
    config: FactsConfig,
    exporters: Vec<Box<dyn FactsExporter>>,
    collected_facts: Arc<RwLock<HashMap<String, Value>>>,
    plugin_manager: Option<Arc<StdRwLock<PluginManager>>>,
}

#[allow(dead_code)]
impl FactsOrchestrator {
    /// Create a new facts orchestrator
    pub fn new(config: FactsConfig) -> Result<Self> {
        Self::new_with_registry_and_plugins(config, prometheus::Registry::new(), None)
    }

    /// Create a new facts orchestrator with a custom registry
    pub fn new_with_registry(config: FactsConfig, registry: prometheus::Registry) -> Result<Self> {
        Self::new_with_registry_and_plugins(config, registry, None)
    }

    /// Create a new facts orchestrator with plugins
    pub fn new_with_registry_and_plugins(
        config: FactsConfig,
        registry: prometheus::Registry,
        plugin_manager: Option<Arc<StdRwLock<PluginManager>>>,
    ) -> Result<Self> {
        let mut exporters = Vec::new();

        // Initialize exporters based on configuration
        if config.export.prometheus.enabled {
            exporters.push(Box::new(PrometheusExporter::new(
                config.export.prometheus.clone(),
                registry.clone(),
            )?) as Box<dyn FactsExporter>);
        }

        if let Some(s3_config) = &config.export.s3 {
            exporters.push(Box::new(S3Exporter::new(s3_config.clone())?) as Box<dyn FactsExporter>);
        }

        if let Some(file_config) = &config.export.file {
            exporters
                .push(Box::new(FileExporter::new(file_config.clone())?) as Box<dyn FactsExporter>);
        }

        Ok(Self {
            config,
            exporters,
            collected_facts: Arc::new(RwLock::new(HashMap::new())),
            plugin_manager,
        })
    }

    /// Start the facts collection orchestration
    pub async fn start(&self) -> Result<()> {
        println!(
            "Starting facts orchestrator with {} collectors and {} exporters",
            self.config.collectors.len(),
            self.exporters.len()
        );

        // Calculate the greatest common divisor of all poll intervals for efficient scheduling
        let intervals: Vec<u64> = self
            .config
            .collectors
            .iter()
            .map(|c| self.get_collector_poll_interval(c))
            .collect();

        let gcd_interval = if intervals.is_empty() {
            60 // default
        } else {
            self.gcd_of_intervals(&intervals)
        };

        println!("Using collection interval: {} seconds", gcd_interval);

        let mut interval_timer = interval(Duration::from_secs(gcd_interval));

        loop {
            interval_timer.tick().await;

            if let Err(e) = self.collect_and_export().await {
                eprintln!("Error during facts collection: {}", e);
                // Continue running despite errors
            }
        }
    }

    /// Collect facts from all enabled collectors and export them
    pub async fn collect_and_export(&self) -> Result<()> {
        let mut all_facts = HashMap::new();

        // Collect facts from all enabled collectors
        for collector in &self.config.collectors {
            if self.is_collector_enabled(collector) {
                let collector_name = self.get_collector_name(collector);

                let facts_result = match collector {
                    Collector::Plugin(_) => {
                        // Handle plugin collectors specially
                        if let Some(ref plugin_manager) = &self.plugin_manager {
                            self.collect_plugin_facts(collector, plugin_manager).await
                        } else {
                            Err(anyhow::anyhow!(
                                "Plugin collector configured but no plugin manager available"
                            ))
                        }
                    }
                    _ => {
                        // Handle built-in collectors
                        FactsRegistry::collect_facts(collector)
                    }
                };

                match facts_result {
                    Ok(facts) => {
                        all_facts.insert(collector_name.clone(), facts);
                    }
                    Err(e) => {
                        eprintln!("Failed to collect facts from {}: {}", collector_name, e);
                        // Continue with other collectors
                    }
                }
            }
        }

        // Store collected facts
        {
            let mut collected = self.collected_facts.write().await;
            *collected = all_facts.clone();
        }

        // Export facts using all configured exporters
        for exporter in &self.exporters {
            if let Err(e) = exporter.export(&all_facts).await {
                eprintln!("Export failed: {}", e);
                // Continue with other exporters
            }
        }

        Ok(())
    }

    /// Collect facts from a plugin collector
    async fn collect_plugin_facts(
        &self,
        collector: &Collector,
        plugin_manager: &Arc<StdRwLock<PluginManager>>,
    ) -> Result<serde_yaml::Value> {
        let plugin_collector = match collector {
            Collector::Plugin(pc) => pc,
            _ => return Err(anyhow::anyhow!("Not a plugin collector")),
        };

        let manager = plugin_manager
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire read lock on plugin manager: {}", e))?;

        // Extract plugin name and collector name from the plugin collector
        // For now, assume the format is "plugin_name.collector_name"
        let (plugin_name, collector_name) =
            crate::plugins::parse_plugin_component_name(&plugin_collector.name)?;
        let config = serde_json::to_value(&plugin_collector.config)?;
        match manager.execute_facts_collector(plugin_name, collector_name, &config) {
            Ok(result) => {
                // Convert serde_json::Value to serde_yaml::Value
                let json_str = serde_json::to_string(&result)?;
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(&json_str)?;
                Ok(yaml_value)
            }
            Err(e) => Err(anyhow::anyhow!(
                "Plugin facts collector execution failed: {}",
                e
            )),
        }
    }

    /// Get collected facts (for external access)
    pub async fn get_collected_facts(&self) -> Result<HashMap<String, Value>> {
        let facts = self.collected_facts.read().await;
        Ok(facts.clone())
    }

    /// Get the number of configured collectors
    pub fn collector_count(&self) -> usize {
        self.config.collectors.len()
    }

    /// Get the number of configured exporters
    pub fn exporter_count(&self) -> usize {
        self.exporters.len()
    }

    /// Check if a collector is enabled
    fn is_collector_enabled(&self, collector: &Collector) -> bool {
        use Collector::*;

        let collector_enabled = match collector {
            System(c) => c.base.enabled,
            Cpu(c) => c.base.enabled,
            Memory(c) => c.base.enabled,
            Disk(c) => c.base.enabled,
            Network(c) => c.base.enabled,
            Process(c) => c.base.enabled,
            Command(c) => c.base.enabled,
            Plugin(c) => c.base.enabled,
        };

        self.config.global.enabled && collector_enabled
    }

    /// Get the name of a collector
    fn get_collector_name(&self, collector: &Collector) -> String {
        use Collector::*;

        match collector {
            System(c) => c.base.name.clone(),
            Cpu(c) => c.base.name.clone(),
            Memory(c) => c.base.name.clone(),
            Disk(c) => c.base.name.clone(),
            Network(c) => c.base.name.clone(),
            Process(c) => c.base.name.clone(),
            Command(c) => c.base.name.clone(),
            Plugin(c) => c.name.clone(),
        }
    }

    /// Get the poll interval for a collector
    fn get_collector_poll_interval(&self, collector: &Collector) -> u64 {
        use Collector::*;

        match collector {
            System(c) => c.base.poll_interval,
            Cpu(c) => c.base.poll_interval,
            Memory(c) => c.base.poll_interval,
            Disk(c) => c.base.poll_interval,
            Network(c) => c.base.poll_interval,
            Process(c) => c.base.poll_interval,
            Command(c) => c.base.poll_interval,
            Plugin(c) => c.base.poll_interval,
        }
    }

    /// Calculate GCD of poll intervals for efficient scheduling
    fn gcd_of_intervals(&self, intervals: &[u64]) -> u64 {
        intervals.iter().fold(intervals[0], |a, &b| self.gcd(a, b))
    }

    /// Calculate GCD of two numbers
    fn gcd(&self, a: u64, b: u64) -> u64 {
        if b == 0 {
            a
        } else {
            self.gcd(b, a % b)
        }
    }
}

/// Trait for facts exporters
#[allow(dead_code)]
#[async_trait::async_trait]
pub trait FactsExporter: Send + Sync {
    /// Export facts data
    async fn export(&self, facts: &HashMap<String, Value>) -> Result<()>;
}

/// Prometheus metrics exporter
#[allow(dead_code)]
pub struct PrometheusExporter {
    config: PrometheusExport,
    registry: prometheus::Registry,
}

#[allow(dead_code)]
impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(config: PrometheusExport, registry: prometheus::Registry) -> Result<Self> {
        Ok(Self { config, registry })
    }
}

#[async_trait::async_trait]
impl FactsExporter for PrometheusExporter {
    async fn export(&self, facts: &HashMap<String, Value>) -> Result<()> {
        // Convert facts to Prometheus metrics and register them
        println!("Prometheus export: {} facts collected", facts.len());

        for (collector_name, fact_data) in facts {
            if let Value::Mapping(fact_map) = fact_data {
                // Convert facts to Prometheus gauge metrics
                for (key, value) in fact_map {
                    if let (Value::String(key_str), Value::Number(num)) = (key, value) {
                        if let Some(num_val) = num.as_f64() {
                            // Create a gauge metric for each fact
                            let gauge = prometheus::Gauge::new(
                                format!("driftless_{}_{}", collector_name, key_str),
                                format!("{} {}", collector_name, key_str),
                            )?;

                            // Register the metric with the registry
                            self.registry.register(Box::new(gauge.clone()))?;

                            // Set the value
                            gauge.set(num_val);

                            println!(
                                "Registered metric: driftless_{}_{} = {}",
                                collector_name, key_str, num_val
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// S3 exporter for facts
#[allow(dead_code)]
pub struct S3Exporter {
    config: crate::facts::S3Export,
}

#[allow(dead_code)]
impl S3Exporter {
    /// Create a new S3 exporter
    pub fn new(config: crate::facts::S3Export) -> Result<Self> {
        Ok(Self { config })
    }
}

#[async_trait::async_trait]
impl FactsExporter for S3Exporter {
    async fn export(&self, facts: &HashMap<String, Value>) -> Result<()> {
        // Serialize facts to JSON for S3 upload
        let facts_json = serde_json::to_string_pretty(facts)?;

        println!(
            "S3 export: {} facts ({:.2} KB) to bucket {} at prefix {}",
            facts.len(),
            facts_json.len() as f64 / 1024.0,
            self.config.bucket,
            self.config.prefix
        );

        // Log the operation (S3 upload implementation would go here)
        println!("Facts data serialized for S3 upload");

        Ok(())
    }
}

/// File exporter for facts
#[allow(dead_code)]
pub struct FileExporter {
    config: crate::facts::FileExport,
}

#[allow(dead_code)]
impl FileExporter {
    /// Create a new file exporter
    pub fn new(config: crate::facts::FileExport) -> Result<Self> {
        Ok(Self { config })
    }
}

#[async_trait::async_trait]
impl FactsExporter for FileExporter {
    async fn export(&self, facts: &HashMap<String, Value>) -> Result<()> {
        // Serialize facts to YAML and write to file
        let facts_yaml = serde_yaml::to_string(facts)?;

        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&self.config.path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.config.path, &facts_yaml)?;

        println!(
            "File export: {} facts written to {}",
            facts.len(),
            self.config.path
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{
        BaseCollector, CpuCollectOptions, CpuCollector, ExportConfig, FactsConfig, GlobalSettings,
    };

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = FactsConfig {
            global: GlobalSettings {
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collectors: vec![],
            export: ExportConfig::default(),
        };

        let orchestrator = FactsOrchestrator::new(config);
        assert!(orchestrator.is_ok());
    }

    #[tokio::test]
    async fn test_orchestrator_with_collectors() {
        let collector = Collector::Cpu(CpuCollector {
            base: BaseCollector {
                name: "test_cpu".to_string(),
                enabled: true,
                poll_interval: 30,
                labels: HashMap::new(),
            },
            collect: CpuCollectOptions::default(),
            thresholds: Default::default(),
        });

        let config = FactsConfig {
            global: GlobalSettings {
                enabled: true,
                poll_interval: 60,
                labels: HashMap::new(),
            },
            collectors: vec![collector],
            export: ExportConfig::default(),
        };

        let orchestrator = FactsOrchestrator::new(config).unwrap();

        // Test collection (this will actually collect real CPU facts)
        let result = orchestrator.collect_and_export().await;
        assert!(result.is_ok());

        // Check that facts were collected
        let facts = orchestrator.get_collected_facts().await.unwrap();
        assert!(!facts.is_empty());
        assert!(facts.contains_key("test_cpu"));
    }

    #[test]
    fn test_gcd_calculation() {
        let config = FactsConfig::default();
        let orchestrator = FactsOrchestrator::new(config).unwrap();

        assert_eq!(orchestrator.gcd(60, 30), 30);
        assert_eq!(orchestrator.gcd(30, 45), 15);
        assert_eq!(orchestrator.gcd(7, 5), 1);
    }

    #[test]
    fn test_gcd_of_intervals() {
        let config = FactsConfig::default();
        let orchestrator = FactsOrchestrator::new(config).unwrap();

        assert_eq!(orchestrator.gcd_of_intervals(&[60, 30, 90]), 30);
        assert_eq!(orchestrator.gcd_of_intervals(&[30, 45, 60]), 15);
        assert_eq!(orchestrator.gcd_of_intervals(&[7]), 7);
    }
}
