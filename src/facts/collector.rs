//! Facts collector
//!
//! This module handles the collection of system metrics and facts as defined
//! in the facts schema.

use crate::facts::{Collector, FactsConfig};
use anyhow::Result;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Prometheus exposition format version
const PROMETHEUS_EXPOSITION_VERSION: &str = "text/plain; version=0.0.4";

/// Metrics collector for system facts
#[allow(dead_code)]
pub struct MetricsCollector {
    config: FactsConfig,
    collected_metrics: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

#[allow(unused)]
impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: FactsConfig) -> Self {
        Self {
            config,
            collected_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start collecting metrics
    pub async fn start(&self) -> Result<()> {
        println!(
            "Starting metrics collection with {} collectors",
            self.config.collectors.len()
        );

        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            self.config.global.poll_interval,
        ));

        loop {
            interval.tick().await;
            if let Err(e) = self.collect_once().await {
                eprintln!("Error collecting metrics: {}", e);
            }
        }
    }

    /// Collect a single round of metrics
    pub async fn collect_once(&self) -> Result<()> {
        let metrics = self.collect_metrics().await?;
        let mut collected = self.collected_metrics.write().await;
        *collected = metrics;
        Ok(())
    }

    /// Collect metrics from all enabled collectors
    pub async fn collect_metrics(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut metrics = HashMap::new();

        for collector in &self.config.collectors {
            if self.is_collector_enabled(collector) {
                let collector_name = self.get_collector_name(collector);
                match crate::facts::FactsRegistry::collect_facts(collector) {
                    Ok(facts) => {
                        // Convert yaml Value to json Value
                        let json_str = serde_yaml::to_string(&facts)?;
                        let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
                        metrics.insert(collector_name.to_string(), json_value);
                    }
                    Err(e) => {
                        eprintln!("Failed to collect facts from {}: {}", collector_name, e);
                    }
                }
            }
        }

        Ok(metrics)
    }

    /// Get the collected metrics
    pub async fn get_collected_metrics(&self) -> Result<HashMap<String, serde_json::Value>> {
        let metrics = self.collected_metrics.read().await;
        Ok(metrics.clone())
    }

    /// Check if a collector is enabled based on global and collector-specific settings
    fn is_collector_enabled(&self, collector: &Collector) -> bool {
        use crate::facts::Collector::*;

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
    fn get_collector_name<'a>(&self, collector: &'a Collector) -> &'a str {
        use crate::facts::Collector::*;

        match collector {
            System(c) => &c.base.name,
            Cpu(c) => &c.base.name,
            Memory(c) => &c.base.name,
            Disk(c) => &c.base.name,
            Network(c) => &c.base.name,
            Process(c) => &c.base.name,
            Command(c) => &c.base.name,
            Plugin(c) => &c.name,
        }
    }

    /// Get the poll interval for a collector
    fn get_collector_interval(&self, collector: &Collector) -> u64 {
        use crate::facts::Collector::*;

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
}

/// Prometheus exporter for metrics
#[allow(dead_code)]
pub struct PrometheusExporter {
    config: crate::facts::PrometheusExport,
    collector: Arc<MetricsCollector>,
}

#[allow(unused)]
impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(config: crate::facts::PrometheusExport, collector: Arc<MetricsCollector>) -> Self {
        Self { config, collector }
    }

    /// Start the Prometheus HTTP server
    pub async fn start(&self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        println!(
            "Starting Prometheus exporter on {}:{}",
            self.config.host, self.config.port
        );

        let collector = Arc::clone(&self.collector);
        let path = self.config.path.clone();

        let app = axum::Router::new().route(
            &path,
            axum::routing::get(move || {
                let collector = Arc::clone(&collector);
                async move {
                    match collector.get_collected_metrics().await {
                        Ok(metrics) => {
                            let body = Self::generate_metrics(&metrics);
                            Response::builder()
                                .status(StatusCode::OK)
                                .header(header::CONTENT_TYPE, PROMETHEUS_EXPOSITION_VERSION)
                                .body(body)
                                .unwrap()
                                .into_response()
                        }
                        Err(e) => {
                            eprintln!("Error getting metrics: {}", e);
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .header(header::CONTENT_TYPE, PROMETHEUS_EXPOSITION_VERSION)
                                .body("# Error getting metrics\n".to_string())
                                .unwrap()
                                .into_response()
                        }
                    }
                }
            }),
        );

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Generate Prometheus format metrics
    pub fn generate_metrics(metrics: &HashMap<String, serde_json::Value>) -> String {
        let mut output = String::new();

        for (collector_name, facts) in metrics {
            if let serde_json::Value::Object(fact_map) = facts {
                for (key, value) in fact_map {
                    if let serde_json::Value::Number(num) = value {
                        if let Some(num_val) = num.as_f64() {
                            output.push_str(&format!(
                                "# HELP driftless_{}_{} {}\n",
                                collector_name, key, key
                            ));
                            output.push_str(&format!(
                                "# TYPE driftless_{}_{} gauge\n",
                                collector_name, key
                            ));
                            output.push_str(&format!(
                                "driftless_{}_{} {}\n",
                                collector_name, key, num_val
                            ));
                        }
                    }
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facts::{BaseCollector, Collector, FactsConfig, GlobalSettings, SystemCollector};

    #[tokio::test]
    async fn test_metrics_collection() {
        let config = FactsConfig {
            global: GlobalSettings {
                enabled: true,
                poll_interval: 30,
                ..Default::default()
            },
            collectors: vec![Collector::System(SystemCollector {
                base: BaseCollector {
                    name: "system".to_string(),
                    enabled: true,
                    poll_interval: 60,
                    labels: HashMap::new(),
                },
                collect: Default::default(),
            })],
            export: Default::default(),
        };

        let collector = MetricsCollector::new(config);
        let metrics = collector.collect_metrics().await.unwrap();

        // Check that we have some metrics collected
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_collector_enabled_check() {
        let config = FactsConfig {
            global: GlobalSettings {
                enabled: true,
                ..Default::default()
            },
            collectors: vec![Collector::System(SystemCollector {
                base: BaseCollector {
                    name: "system".to_string(),
                    enabled: true,
                    poll_interval: 60,
                    labels: HashMap::new(),
                },
                collect: Default::default(),
            })],
            export: Default::default(),
        };

        let collector = MetricsCollector::new(config);
        let test_collector = &collector.config.collectors[0];

        assert!(collector.is_collector_enabled(test_collector));
        assert_eq!(collector.get_collector_name(test_collector), "system");
        assert_eq!(collector.get_collector_interval(test_collector), 60);
    }
}
