use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Module declarations for individual collectors
mod collector;
mod command_facts;
mod cpu_facts;
mod disk_facts;
mod memory_facts;
mod network_facts;
mod orchestrator;
mod process_facts;
mod system_facts;

// Type alias for facts collector functions
type FactsCollectorFn = Arc<dyn Fn(&Collector) -> Result<serde_yaml::Value> + Send + Sync>;

// Facts registry entry containing collector function and metadata
#[derive(Clone)]
pub(crate) struct FactsRegistryEntry {
    collector: FactsCollectorFn,
    category: String,
    description: String,
    filename: String,
}

// Global facts registry for extensible facts collection
static FACTS_REGISTRY: Lazy<RwLock<HashMap<String, FactsRegistryEntry>>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // Initialize with built-in collectors
    FactsRegistry::initialize_builtin_collectors(&mut registry);

    RwLock::new(registry)
});

/// Facts collector registry for runtime extensibility
pub struct FactsRegistry;

impl FactsRegistry {
    /// Register a facts collector function with the global registry
    #[allow(unused)]
    pub fn register_collector(
        collector_type: &str,
        category: &str,
        description: &str,
        filename: &str,
        collector: FactsCollectorFn,
    ) {
        let entry = FactsRegistryEntry {
            collector,
            category: category.to_string(),
            description: description.to_string(),
            filename: filename.to_string(),
        };
        let mut registry = FACTS_REGISTRY.write().unwrap();
        registry.insert(collector_type.to_string(), entry);
    }

    /// Register a facts collector function
    pub(crate) fn register(
        registry: &mut HashMap<String, FactsRegistryEntry>,
        collector_type: &str,
        category: &str,
        description: &str,
        filename: &str,
        collector: FactsCollectorFn,
    ) {
        let entry = FactsRegistryEntry {
            collector,
            category: category.to_string(),
            description: description.to_string(),
            filename: filename.to_string(),
        };
        registry.insert(collector_type.to_string(), entry);
    }

    /// Initialize the registry with built-in facts collectors
    pub(crate) fn initialize_builtin_collectors(
        registry: &mut HashMap<String, FactsRegistryEntry>,
    ) {
        // System facts collector
        FactsRegistry::register(
            registry,
            "system",
            "System Information",
            "Collect system information including hostname, OS, kernel, uptime, and architecture",
            "system_facts",
            Arc::new(|collector| {
                if let Collector::System(system_collector) = collector {
                    system_facts::collect_system_facts(system_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for system facts"))
                }
            }),
        );

        // CPU facts collector
        FactsRegistry::register(
            registry,
            "cpu",
            "CPU Metrics",
            "Collect CPU usage, frequency, temperature, and load average metrics",
            "cpu_facts",
            Arc::new(|collector| {
                if let Collector::Cpu(cpu_collector) = collector {
                    cpu_facts::collect_cpu_facts(cpu_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for CPU facts"))
                }
            }),
        );

        // Memory facts collector
        FactsRegistry::register(
            registry,
            "memory",
            "Memory Metrics",
            "Collect memory usage statistics including total, used, free, and swap",
            "memory_facts",
            Arc::new(|collector| {
                if let Collector::Memory(memory_collector) = collector {
                    memory_facts::collect_memory_facts(memory_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for memory facts"))
                }
            }),
        );

        // Disk facts collector
        FactsRegistry::register(
            registry,
            "disk",
            "Disk Metrics",
            "Collect disk space and I/O statistics for mounted filesystems",
            "disk_facts",
            Arc::new(|collector| {
                if let Collector::Disk(disk_collector) = collector {
                    disk_facts::collect_disk_facts(disk_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for disk facts"))
                }
            }),
        );

        // Network facts collector
        FactsRegistry::register(
            registry,
            "network",
            "Network Metrics",
            "Collect network interface statistics and status information",
            "network_facts",
            Arc::new(|collector| {
                if let Collector::Network(network_collector) = collector {
                    network_facts::collect_network_facts(network_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for network facts"))
                }
            }),
        );

        // Process facts collector
        FactsRegistry::register(
            registry,
            "process",
            "Process Metrics",
            "Collect process information and resource usage statistics",
            "process_facts",
            Arc::new(|collector| {
                if let Collector::Process(process_collector) = collector {
                    process_facts::collect_process_facts(process_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for process facts"))
                }
            }),
        );

        // Command facts collector
        FactsRegistry::register(
            registry,
            "command",
            "Command Output",
            "Execute custom commands and collect their output as facts",
            "command_facts",
            Arc::new(|collector| {
                if let Collector::Command(command_collector) = collector {
                    command_facts::collect_command_facts(command_collector)
                } else {
                    Err(anyhow::anyhow!("Invalid collector type for command facts"))
                }
            }),
        );
    }

    /// Get all registered collector types
    pub fn get_registered_collector_types() -> Vec<String> {
        let registry = FACTS_REGISTRY.read().unwrap();
        registry.keys().cloned().collect()
    }

    /// Get the category for a collector type
    #[allow(unused)]
    pub fn get_collector_category(collector_type: &str) -> String {
        let registry = FACTS_REGISTRY.read().unwrap();
        registry
            .get(collector_type)
            .map(|e| e.category.clone())
            .unwrap_or_else(|| "Other".to_string())
    }

    /// Get the description for a collector type
    pub fn get_collector_description(collector_type: &str) -> String {
        let registry = FACTS_REGISTRY.read().unwrap();
        registry
            .get(collector_type)
            .map(|e| e.description.clone())
            .unwrap_or_else(|| "Unknown collector type".to_string())
    }

    /// Get the filename for a collector type
    #[allow(unused)]
    pub fn get_collector_filename(collector_type: &str) -> String {
        let registry = FACTS_REGISTRY.read().unwrap();
        registry
            .get(collector_type)
            .map(|e| e.filename.clone())
            .unwrap_or_else(|| "mod".to_string())
    }

    /// Execute a collector and return facts
    #[allow(unused)]
    pub fn collect_facts(collector: &Collector) -> Result<serde_yaml::Value> {
        let collector_type = match collector {
            Collector::System(_) => "system".to_string(),
            Collector::Cpu(_) => "cpu".to_string(),
            Collector::Memory(_) => "memory".to_string(),
            Collector::Disk(_) => "disk".to_string(),
            Collector::Network(_) => "network".to_string(),
            Collector::Process(_) => "process".to_string(),
            Collector::Command(_) => "command".to_string(),
            Collector::Plugin(plugin_collector) => plugin_collector.name.clone(),
        };

        let entry = {
            let registry = FACTS_REGISTRY.read().unwrap();
            registry.get(&collector_type).cloned()
        };

        if let Some(entry) = entry {
            (entry.collector)(collector)
        } else {
            Err(anyhow::anyhow!(
                "No collector registered for type: {}",
                collector_type
            ))
        }
    }

    /// Register a plugin-provided facts collector at runtime
    #[allow(dead_code)]
    pub fn register_plugin_collector(
        collector_name: &str,
        collector: FactsCollectorFn,
    ) -> Result<()> {
        let mut registry = FACTS_REGISTRY.write().unwrap();
        if registry.contains_key(collector_name) {
            return Err(anyhow::anyhow!(
                "Collector '{}' is already registered",
                collector_name
            ));
        }
        let entry = FactsRegistryEntry {
            collector,
            category: "Plugin Collectors".to_string(),
            description: format!("Plugin-provided collector: {}", collector_name),
            filename: "plugin".to_string(),
        };
        registry.insert(collector_name.to_string(), entry);
        Ok(())
    }
}

// Public exports
#[allow(unused)]
pub use orchestrator::{
    FactsExporter, FactsOrchestrator, FileExporter, PrometheusExporter, S3Exporter,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactsConfig {
    /// Global settings for facts collection
    #[serde(default)]
    pub global: GlobalSettings,
    /// List of collectors to run
    pub collectors: Vec<Collector>,
    /// Export configuration
    #[serde(default)]
    pub export: ExportConfig,
}

/// Global settings for facts collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Default poll interval (seconds)
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
    /// Enable/disable all collectors
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Labels to add to all metrics
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            poll_interval: default_poll_interval(),
            enabled: default_true(),
            labels: HashMap::new(),
        }
    }
}

/// Plugin-provided facts collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// Plugin collector name
    pub name: String,
    /// Collector-specific configuration
    #[serde(flatten)]
    pub config: serde_yaml::Value,
}

/// Types of collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Collector {
    /// System information collector
    System(SystemCollector),
    /// CPU metrics collector
    Cpu(CpuCollector),
    /// Memory metrics collector
    Memory(MemoryCollector),
    /// Disk metrics collector
    Disk(DiskCollector),
    /// Network metrics collector
    Network(NetworkCollector),
    /// Process metrics collector
    Process(ProcessCollector),
    /// Custom command output collector
    Command(CommandCollector),
    /// Plugin-provided facts collector
    Plugin(PluginCollector),
}

/// Base collector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCollector {
    /// Collector name (used for metric names)
    pub name: String,
    /// Whether this collector is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Poll interval in seconds (how often to collect this metric, default: 60)
    pub poll_interval: u64,
    /// Additional labels for this collector
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// System information collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// What system information to collect
    #[serde(default)]
    pub collect: SystemCollectOptions,
}

/// System information collection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemCollectOptions {
    /// Collect hostname
    #[serde(default = "default_true")]
    pub hostname: bool,
    /// Collect OS information
    #[serde(default = "default_true")]
    pub os: bool,
    /// Collect kernel version
    #[serde(default = "default_true")]
    pub kernel: bool,
    /// Collect system uptime
    #[serde(default = "default_true")]
    pub uptime: bool,
    /// Collect boot time
    #[serde(default = "default_true")]
    pub boot_time: bool,
    /// Collect CPU architecture
    #[serde(default = "default_true")]
    pub arch: bool,
}

/// CPU metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// CPU metrics to collect
    #[serde(default)]
    pub collect: CpuCollectOptions,
    /// Thresholds for alerts
    #[serde(default)]
    pub thresholds: CpuThresholds,
}

/// CPU metrics collection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuCollectOptions {
    /// Collect CPU usage percentage
    #[serde(default = "default_true")]
    pub usage: bool,
    /// Collect per-core usage
    #[serde(default = "default_true")]
    pub per_core: bool,
    /// Collect CPU frequency
    #[serde(default = "default_true")]
    pub frequency: bool,
    /// Collect CPU temperature (if available)
    #[serde(default = "default_true")]
    pub temperature: bool,
    /// Collect CPU load average
    #[serde(default = "default_true")]
    pub load_average: bool,
}

/// CPU thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuThresholds {
    /// CPU usage warning threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_warning: Option<f64>,
    /// CPU usage critical threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_critical: Option<f64>,
    /// Temperature warning threshold (celsius)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_warning: Option<f64>,
    /// Temperature critical threshold (celsius)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temp_critical: Option<f64>,
}

/// Memory metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// Memory metrics to collect
    #[serde(default)]
    pub collect: MemoryCollectOptions,
    /// Thresholds for alerts
    #[serde(default)]
    pub thresholds: MemoryThresholds,
}

/// Memory metrics collection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryCollectOptions {
    /// Collect total memory
    #[serde(default = "default_true")]
    pub total: bool,
    /// Collect used memory
    #[serde(default = "default_true")]
    pub used: bool,
    /// Collect free memory
    #[serde(default = "default_true")]
    pub free: bool,
    /// Collect available memory
    #[serde(default = "default_true")]
    pub available: bool,
    /// Collect swap usage
    #[serde(default = "default_true")]
    pub swap: bool,
    /// Collect memory usage percentage
    #[serde(default = "default_true")]
    pub percentage: bool,
}

/// Memory thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryThresholds {
    /// Memory usage warning threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_warning: Option<f64>,
    /// Memory usage critical threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_critical: Option<f64>,
}

/// Disk metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// Disk devices to monitor (empty = all)
    #[serde(default)]
    pub devices: Vec<String>,
    /// Mount points to monitor (empty = all)
    #[serde(default)]
    pub mount_points: Vec<String>,
    /// Disk metrics to collect
    #[serde(default)]
    pub collect: DiskCollectOptions,
    /// Thresholds for alerts
    #[serde(default)]
    pub thresholds: DiskThresholds,
}

/// Disk metrics collection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskCollectOptions {
    /// Collect total disk space
    #[serde(default = "default_true")]
    pub total: bool,
    /// Collect used disk space
    #[serde(default = "default_true")]
    pub used: bool,
    /// Collect free disk space
    #[serde(default = "default_true")]
    pub free: bool,
    /// Collect available disk space
    #[serde(default = "default_true")]
    pub available: bool,
    /// Collect disk usage percentage
    #[serde(default = "default_true")]
    pub percentage: bool,
    /// Collect disk I/O statistics
    #[serde(default = "default_true")]
    pub io: bool,
}

/// Disk thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskThresholds {
    /// Disk usage warning threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_warning: Option<f64>,
    /// Disk usage critical threshold (percentage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_critical: Option<f64>,
}

/// Network metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// Network interfaces to monitor (empty = all)
    #[serde(default)]
    pub interfaces: Vec<String>,
    /// Network metrics to collect
    #[serde(default)]
    pub collect: NetworkCollectOptions,
}

/// Network metrics collection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkCollectOptions {
    /// Collect bytes transmitted/received
    #[serde(default = "default_true")]
    pub bytes: bool,
    /// Collect packets transmitted/received
    #[serde(default = "default_true")]
    pub packets: bool,
    /// Collect errors and drops
    #[serde(default = "default_true")]
    pub errors: bool,
    /// Collect network interface status
    #[serde(default = "default_true")]
    pub status: bool,
}

/// Process metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// Process name patterns to monitor (empty = all processes)
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Process metrics to collect
    #[serde(default)]
    pub collect: ProcessCollectOptions,
}

/// Process metrics collection options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessCollectOptions {
    /// Collect process count
    #[serde(default = "default_true")]
    pub count: bool,
    /// Collect CPU usage per process
    #[serde(default = "default_true")]
    pub cpu: bool,
    /// Collect memory usage per process
    #[serde(default = "default_true")]
    pub memory: bool,
    /// Collect process status
    #[serde(default = "default_true")]
    pub status: bool,
}

/// Command output collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandCollector {
    #[serde(flatten)]
    pub base: BaseCollector,
    /// Command to execute
    pub command: String,
    /// Working directory for command
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Expected output format
    #[serde(default)]
    pub format: CommandOutputFormat,
    /// Labels to extract from command output
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Command output format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CommandOutputFormat {
    /// Plain text output
    #[default]
    Text,
    /// JSON output
    Json,
    /// Key-value pairs (key=value format)
    KeyValue,
}

/// Export configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExportConfig {
    /// Prometheus export settings
    #[serde(default)]
    pub prometheus: PrometheusExport,
    /// S3 export settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3: Option<S3Export>,
    /// Local file export settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileExport>,
}

/// Prometheus export configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrometheusExport {
    /// Whether to enable Prometheus endpoint
    #[serde(default)]
    pub enabled: bool,
    /// Port for Prometheus metrics endpoint
    #[serde(default = "default_prometheus_port")]
    pub port: u16,
    /// Host for Prometheus metrics endpoint
    #[serde(default = "default_prometheus_host")]
    pub host: String,
    /// Path for metrics endpoint
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

/// S3 export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Export {
    /// S3 bucket name
    pub bucket: String,
    /// S3 region
    pub region: String,
    /// S3 key prefix
    #[serde(default = "default_s3_prefix")]
    pub prefix: String,
    /// Export interval (seconds)
    #[serde(default = "default_export_interval")]
    pub interval: u64,
    /// S3 credentials (if not using environment/default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
}

/// File export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExport {
    /// Output file path
    pub path: String,
    /// Export format
    #[serde(default)]
    pub format: FileFormat,
    /// Export interval (seconds)
    #[serde(default = "default_export_interval")]
    pub interval: u64,
}

/// File export format
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    /// Prometheus format
    #[default]
    Prometheus,
    /// JSON format
    Json,
    /// InfluxDB line protocol
    Influx,
}

// Default value functions
fn default_poll_interval() -> u64 {
    30
}
fn default_export_interval() -> u64 {
    60
}
pub fn default_true() -> bool {
    true
}
fn default_prometheus_port() -> u16 {
    8000
}
fn default_prometheus_host() -> String {
    "0.0.0.0".to_string()
}
fn default_metrics_path() -> String {
    "/metrics".to_string()
}
fn default_s3_prefix() -> String {
    "metrics/".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_cpu_collector() {
        let yaml = r#"
type: cpu
name: cpu
enabled: true
poll_interval: 60
collect:
  usage: true
  per_core: true
  temperature: true
thresholds:
  usage_warning: 80.0
  usage_critical: 95.0
  temp_warning: 70.0
"#;

        let collector: Collector = serde_yaml::from_str(yaml).unwrap();
        match collector {
            Collector::Cpu(cpu) => {
                assert_eq!(cpu.base.name, "cpu");
                assert!(cpu.base.enabled);
                assert!(cpu.collect.usage);
                assert!(cpu.collect.per_core);
                assert!(cpu.collect.temperature);
                assert_eq!(cpu.thresholds.usage_warning, Some(80.0));
                assert_eq!(cpu.thresholds.usage_critical, Some(95.0));
            }
            _ => panic!("Expected CPU collector"),
        }
    }

    #[test]
    fn test_deserialize_memory_collector() {
        let yaml = r#"
type: memory
name: memory
poll_interval: 60
collect:
  total: true
  used: true
  percentage: true
thresholds:
  usage_warning: 85.0
  usage_critical: 95.0
"#;

        let collector: Collector = serde_yaml::from_str(yaml).unwrap();
        match collector {
            Collector::Memory(mem) => {
                assert_eq!(mem.base.name, "memory");
                assert!(mem.collect.total);
                assert!(mem.collect.used);
                assert!(mem.collect.percentage);
                assert_eq!(mem.thresholds.usage_warning, Some(85.0));
            }
            _ => panic!("Expected Memory collector"),
        }
    }

    #[test]
    fn test_deserialize_prometheus_export() {
        let yaml = r#"
global:
  enabled: true
collectors: []
export:
  prometheus:
    enabled: true
    port: 9090
    host: "127.0.0.1"
    path: "/metrics"
"#;

        let config: FactsConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.export.prometheus.enabled);
        assert_eq!(config.export.prometheus.port, 9090);
        assert_eq!(config.export.prometheus.host, "127.0.0.1");
        assert_eq!(config.export.prometheus.path, "/metrics");
    }
}
