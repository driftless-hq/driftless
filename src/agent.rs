//! Agent mode implementation
//!
//! This module provides the agent mode functionality that runs an event loop
//! for continuous configuration enforcement, metrics collection, and log forwarding.

use crate::apply::{executor::TaskExecutor, ApplyConfig};
use crate::facts::{FactsConfig, FactsOrchestrator};
use crate::logs::{LogOrchestrator, LogsConfig};
// use crate::secrets::SecretsManager;
use anyhow::Result;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, instrument, warn};

/// Circuit breaker for error recovery and graceful degradation
#[derive(Debug)]
struct CircuitBreaker {
    failure_count: u32,
    last_failure_time: Option<Instant>,
    state: CircuitBreakerState,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, requests rejected
    HalfOpen, // Testing recovery
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            last_failure_time: None,
            state: CircuitBreakerState::Closed,
            failure_threshold,
            recovery_timeout,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    fn can_attempt(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }
}

/// Apply task execution metrics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApplyMetrics {
    pub execution_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_execution: Option<Instant>,
    pub last_duration: Option<Duration>,
}

/// Apply configuration status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ApplyStatus {
    pub configured: bool,
    pub dry_run: bool,
    pub task_count: usize,
}

/// Facts collection metrics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FactsMetrics {
    pub collection_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_collection: Option<Instant>,
    pub last_duration: Option<Duration>,
}

/// Facts configuration status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FactsStatus {
    pub configured: bool,
    pub collector_count: usize,
    pub exporter_count: usize,
}

/// Logs processing metrics
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogsMetrics {
    pub start_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_start: Option<Instant>,
    pub uptime: Option<Duration>,
}

/// Logs configuration status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogsStatus {
    pub configured: bool,
    pub source_count: usize,
    pub output_count: usize,
    pub running: bool,
}

/// Agent configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    /// Configuration directory to monitor
    pub config_dir: PathBuf,
    /// Plugin directory to scan for plugin files
    pub plugin_dir: PathBuf,
    /// Interval for running apply tasks (seconds)
    pub apply_interval: u64,
    /// Interval for collecting facts (seconds)
    pub facts_interval: u64,
    /// Whether to run apply tasks in dry-run mode
    pub apply_dry_run: bool,
    /// Metrics endpoint port
    pub metrics_port: u16,
    /// Whether agent is enabled
    pub enabled: bool,
    /// Secrets loaded from environment and files
    pub secrets: HashMap<String, String>,
    /// Resource monitoring configuration
    pub resource_monitoring: ResourceMonitoringConfig,
}

/// Resource monitoring configuration for performance optimization
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceMonitoringConfig {
    /// Whether resource monitoring is enabled
    pub enabled: bool,
    /// Cache duration for resource metrics (seconds) - reduces monitoring frequency
    pub cache_duration: u64,
    /// Memory usage threshold for warnings (bytes)
    pub memory_warning_threshold: u64,
    /// CPU usage threshold for warnings (percentage)
    pub cpu_warning_threshold: f64,
    /// Whether to use async/background monitoring
    pub async_monitoring: bool,
    /// Selective monitoring - only monitor when apply tasks are running
    pub selective_monitoring: bool,
    /// Use lightweight monitoring (less data refresh for better performance)
    pub lightweight_monitoring: bool,
}

impl Default for ResourceMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_duration: 30,                           // Cache for 30 seconds
            memory_warning_threshold: 1024 * 1024 * 1024, // 1GB
            cpu_warning_threshold: 80.0,                  // 80%
            async_monitoring: true,                       // Use async monitoring by default
            selective_monitoring: false,                  // Monitor continuously by default
            lightweight_monitoring: true,                 // Use lightweight monitoring by default
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            config_dir: PathBuf::from("~/.config/driftless/config"),
            plugin_dir: PathBuf::from("~/.config/driftless/plugins"),
            apply_interval: 300, // 5 minutes
            facts_interval: 60,  // 1 minute
            apply_dry_run: false,
            metrics_port: 8000,
            enabled: true,
            secrets: HashMap::new(),
            resource_monitoring: ResourceMonitoringConfig::default(),
        }
    }
}

/// Agent that orchestrates apply, facts, and logs operations
pub struct Agent {
    config: AgentConfig,
    apply_config: Option<ApplyConfig>,
    apply_executor: Option<Arc<Mutex<TaskExecutor>>>,
    facts_orchestrator: Option<Arc<Mutex<FactsOrchestrator>>>,
    logs_orchestrator: Option<Arc<Mutex<LogOrchestrator>>>,
    logs_config: Option<LogsConfig>,
    plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
    config_watcher: Option<RecommendedWatcher>,
    metrics_registry: prometheus::Registry,
    metrics_server_handle: Option<tokio::task::JoinHandle<()>>,
    running: bool,
    // Apply task execution metrics
    apply_execution_count: u64,
    apply_success_count: u64,
    apply_failure_count: u64,
    apply_last_execution: Option<Instant>,
    apply_last_duration: Option<Duration>,
    // Facts collection metrics
    facts_collection_count: u64,
    facts_success_count: u64,
    facts_failure_count: u64,
    facts_last_collection: Option<Instant>,
    facts_last_duration: Option<Duration>,
    // Facts configuration status cache
    facts_collector_count: usize,
    facts_exporter_count: usize,
    // Logs processing metrics
    logs_start_count: u64,
    logs_success_count: u64,
    logs_failure_count: u64,
    logs_last_start: Option<Instant>,
    logs_uptime_start: Option<Instant>,
    // Logs configuration status cache
    logs_source_count: usize,
    logs_output_count: usize,
    logs_running: bool,
    // Configuration change tracking
    config_changed: Arc<AtomicBool>,
    // Configuration hash tracking for change detection
    apply_config_hash: Option<String>,
    facts_config_hash: Option<String>,
    logs_config_hash: Option<String>,
    // Error recovery and circuit breaker
    apply_circuit_breaker: CircuitBreaker,
    facts_circuit_breaker: CircuitBreaker,
    logs_circuit_breaker: CircuitBreaker,
    // Resource monitoring
    memory_usage: u64,
    cpu_usage: f64,
    last_resource_check: Option<Instant>,
    // Resource monitoring cache and async task
    resource_cache: Arc<Mutex<ResourceCache>>,
    #[allow(dead_code)]
    resource_monitor_task: Option<tokio::task::JoinHandle<()>>,
}

/// Cached resource monitoring data for performance optimization
#[derive(Debug, Clone)]
struct ResourceCache {
    memory_usage: u64,
    cpu_usage: f64,
    last_updated: Option<Instant>,
    cache_duration: Duration,
}

/// Compare two log configurations to see if they are equivalent
fn configs_are_equal(a: &crate::logs::LogsConfig, b: &crate::logs::LogsConfig) -> bool {
    // Compare global settings
    if a.global.enabled != b.global.enabled
        || a.global.buffer_size != b.global.buffer_size
        || a.global.flush_interval != b.global.flush_interval
        || a.global.labels != b.global.labels
    {
        return false;
    }

    // Compare sources (order matters for now, could be made order-independent)
    if a.sources.len() != b.sources.len() {
        return false;
    }
    for (source_a, source_b) in a.sources.iter().zip(b.sources.iter()) {
        if source_a.name != source_b.name
            || source_a.enabled != source_b.enabled
            || source_a.source_type != source_b.source_type
            || source_a.paths != source_b.paths
        {
            return false;
        }
    }

    // Compare outputs
    if a.outputs.len() != b.outputs.len() {
        return false;
    }
    for (output_a, output_b) in a.outputs.iter().zip(b.outputs.iter()) {
        // This is a simplified comparison - in practice, you'd need to compare
        // the actual output configurations which vary by type
        if !outputs_are_equal(output_a, output_b) {
            return false;
        }
    }

    // Compare processing config
    if a.processing.enabled != b.processing.enabled
        || a.processing.global_filters.len() != b.processing.global_filters.len()
        || a.processing.transformations.len() != b.processing.transformations.len()
    {
        return false;
    }

    true
}

/// Compare two log outputs for equality
fn outputs_are_equal(a: &crate::logs::LogOutput, b: &crate::logs::LogOutput) -> bool {
    use crate::logs::LogOutput::*;
    match (a, b) {
        (File(a), File(b)) => a.enabled == b.enabled && a.path == b.path,
        (S3(a), S3(b)) => a.enabled == b.enabled && a.bucket == b.bucket && a.prefix == b.prefix,
        (Http(a), Http(b)) => a.enabled == b.enabled && a.url == b.url,
        (Syslog(a), Syslog(b)) => a.enabled == b.enabled && a.facility == b.facility,
        (Console(a), Console(b)) => a.enabled == b.enabled,
        (Plugin(a), Plugin(b)) => a.enabled == b.enabled && a.config == b.config,
        _ => false, // Different output types
    }
}

/// Calculate SHA256 hash of a serializable configuration
fn calculate_config_hash<T: serde::Serialize>(config: &T) -> Result<String> {
    let json = serde_json::to_string(config)?;
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

impl Agent {
    /// Create a new agent with the given configuration
    pub fn new(config: AgentConfig) -> Self {
        let cache_duration = Duration::from_secs(config.resource_monitoring.cache_duration);

        Self {
            config,
            apply_config: None,
            apply_executor: None,
            facts_orchestrator: None,
            logs_orchestrator: None,
            logs_config: None,
            plugin_manager: None,
            config_watcher: None,
            metrics_registry: prometheus::Registry::new(),
            metrics_server_handle: None,
            running: false,
            apply_execution_count: 0,
            apply_success_count: 0,
            apply_failure_count: 0,
            apply_last_execution: None,
            apply_last_duration: None,
            facts_collection_count: 0,
            facts_success_count: 0,
            facts_failure_count: 0,
            facts_last_collection: None,
            facts_last_duration: None,
            facts_collector_count: 0,
            facts_exporter_count: 0,
            logs_start_count: 0,
            logs_success_count: 0,
            logs_failure_count: 0,
            logs_last_start: None,
            logs_uptime_start: None,
            logs_source_count: 0,
            logs_output_count: 0,
            logs_running: false,
            config_changed: Arc::new(AtomicBool::new(false)),
            // Initialize configuration hash tracking
            apply_config_hash: None,
            facts_config_hash: None,
            logs_config_hash: None,
            // Initialize circuit breakers with reasonable defaults
            apply_circuit_breaker: CircuitBreaker::new(5, Duration::from_secs(60)),
            facts_circuit_breaker: CircuitBreaker::new(3, Duration::from_secs(30)),
            logs_circuit_breaker: CircuitBreaker::new(3, Duration::from_secs(30)),
            // Initialize resource monitoring
            memory_usage: 0,
            cpu_usage: 0.0,
            last_resource_check: None,
            // Initialize resource cache
            resource_cache: Arc::new(Mutex::new(ResourceCache {
                memory_usage: 0,
                cpu_usage: 0.0,
                last_updated: None,
                cache_duration,
            })),
            resource_monitor_task: None,
        }
    }

    /// Start the agent and initialize all orchestrators
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(anyhow::anyhow!("Agent is already running"));
        }

        println!("Starting driftless agent...");

        // Load secrets
        self.load_secrets()?;

        // Initialize orchestrators
        self.initialize_orchestrators().await?;

        // Start configuration watcher
        self.start_config_watcher()?;

        // Start metrics HTTP server
        self.start_metrics_server()?;

        self.running = true;
        println!("Agent started successfully");

        Ok(())
    }

    /// Stop the agent and shutdown all orchestrators
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        println!("Stopping driftless agent...");

        // Stop metrics server
        if let Some(handle) = self.metrics_server_handle.take() {
            handle.abort();
            println!("Metrics server stopped");
        }

        // Stop logs orchestrator
        if let Some(orchestrator) = &self.logs_orchestrator {
            let mut orchestrator = orchestrator.lock().await;
            if let Err(e) = orchestrator.stop().await {
                eprintln!("Error stopping logs orchestrator: {}", e);
            }
        }

        // Stop orchestrators in reverse order
        if let Some(_logs_orchestrator) = &self.logs_orchestrator {
            // Note: LogOrchestrator doesn't have a public stop method yet
            // This will be implemented in Phase 5
        }

        if let Some(_facts_orchestrator) = &self.facts_orchestrator {
            // Note: FactsOrchestrator doesn't have a public stop method yet
            // This will be implemented in Phase 4
        }

        if let Some(_apply_executor) = &self.apply_executor {
            // Note: TaskExecutor doesn't have a stop method
            // Apply tasks are typically run on-demand
        }

        self.running = false;
        println!("Agent stopped successfully");

        Ok(())
    }

    /// Get apply task execution metrics
    #[allow(dead_code)]
    pub fn apply_metrics(&self) -> ApplyMetrics {
        ApplyMetrics {
            execution_count: self.apply_execution_count,
            success_count: self.apply_success_count,
            failure_count: self.apply_failure_count,
            last_execution: self.apply_last_execution,
            last_duration: self.apply_last_duration,
        }
    }

    /// Get current apply configuration status
    #[allow(dead_code)]
    pub fn apply_status(&self) -> ApplyStatus {
        ApplyStatus {
            configured: self.apply_config.is_some(),
            dry_run: self.config.apply_dry_run,
            task_count: self
                .apply_config
                .as_ref()
                .map(|c| c.tasks.len())
                .unwrap_or(0),
        }
    }

    /// Get facts collection metrics
    #[allow(dead_code)]
    pub fn facts_metrics(&self) -> FactsMetrics {
        FactsMetrics {
            collection_count: self.facts_collection_count,
            success_count: self.facts_success_count,
            failure_count: self.facts_failure_count,
            last_collection: self.facts_last_collection,
            last_duration: self.facts_last_duration,
        }
    }

    /// Get current facts configuration status
    #[allow(dead_code)]
    pub fn facts_status(&self) -> FactsStatus {
        FactsStatus {
            configured: self.facts_orchestrator.is_some(),
            collector_count: self.facts_collector_count,
            exporter_count: self.facts_exporter_count,
        }
    }

    /// Get logs processing metrics
    #[allow(dead_code)]
    pub fn logs_metrics(&self) -> LogsMetrics {
        LogsMetrics {
            start_count: self.logs_start_count,
            success_count: self.logs_success_count,
            failure_count: self.logs_failure_count,
            last_start: self.logs_last_start,
            uptime: self.logs_uptime_start.map(|start| start.elapsed()),
        }
    }

    /// Get current logs configuration status
    #[allow(dead_code)]
    pub fn logs_status(&self) -> LogsStatus {
        LogsStatus {
            configured: self.logs_orchestrator.is_some(),
            source_count: self.logs_source_count,
            output_count: self.logs_output_count,
            running: self.logs_running,
        }
    }

    /// Check if the agent is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get apply execution count
    pub fn apply_execution_count(&self) -> u64 {
        self.apply_execution_count
    }

    /// Get apply success count
    pub fn apply_success_count(&self) -> u64 {
        self.apply_success_count
    }

    /// Get apply failure count
    pub fn apply_failure_count(&self) -> u64 {
        self.apply_failure_count
    }

    /// Get apply last execution time
    pub fn apply_last_execution(&self) -> Option<Instant> {
        self.apply_last_execution
    }

    /// Get apply last duration
    pub fn apply_last_duration(&self) -> Option<Duration> {
        self.apply_last_duration
    }

    /// Get facts collection count
    pub fn facts_collection_count(&self) -> u64 {
        self.facts_collection_count
    }

    /// Get facts success count
    pub fn facts_success_count(&self) -> u64 {
        self.facts_success_count
    }

    /// Get facts failure count
    pub fn facts_failure_count(&self) -> u64 {
        self.facts_failure_count
    }

    /// Get facts last collection time
    pub fn facts_last_collection(&self) -> Option<Instant> {
        self.facts_last_collection
    }

    /// Get facts last duration
    pub fn facts_last_duration(&self) -> Option<Duration> {
        self.facts_last_duration
    }

    /// Update resource usage metrics with caching and async support
    pub async fn update_resource_usage(&mut self) -> Result<()> {
        if !self.config.resource_monitoring.enabled {
            return Ok(());
        }

        // Check cache first
        {
            let cache = self.resource_cache.lock().await;
            if let Some(last_updated) = cache.last_updated {
                if last_updated.elapsed() < cache.cache_duration {
                    // Use cached values
                    self.memory_usage = cache.memory_usage;
                    self.cpu_usage = cache.cpu_usage;
                    self.last_resource_check = Some(Instant::now());
                    return Ok(());
                }
            }
        }

        // Selective monitoring: only monitor when apply tasks are running
        if self.config.resource_monitoring.selective_monitoring && self.apply_executor.is_none() {
            return Ok(());
        }

        if self.config.resource_monitoring.async_monitoring {
            // Async monitoring - spawn background task
            self.update_resource_usage_async().await
        } else {
            // Synchronous monitoring (fallback)
            self.update_resource_usage_sync();
            Ok(())
        }
    }

    /// Update resource usage metrics (synchronous version for benchmarks)
    #[allow(dead_code)]
    pub fn update_resource_usage_sync(&mut self) {
        use sysinfo::System;

        let mut sys = System::new();

        if self.config.resource_monitoring.lightweight_monitoring {
            // Lightweight monitoring - only refresh memory and CPU
            sys.refresh_memory();
            sys.refresh_cpu();
        } else {
            // Full monitoring - refresh all system info
            sys.refresh_all();
        }

        // Get memory usage in bytes
        let memory_usage = sys.used_memory();
        // Get CPU usage as percentage
        let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

        // Update cache
        {
            let mut cache = self.resource_cache.blocking_lock();
            cache.memory_usage = memory_usage;
            cache.cpu_usage = cpu_usage;
            cache.last_updated = Some(Instant::now());
        }

        // Update agent fields
        self.memory_usage = memory_usage;
        self.cpu_usage = cpu_usage;
        self.last_resource_check = Some(Instant::now());
    }

    /// Asynchronous resource monitoring for better performance
    async fn update_resource_usage_async(&mut self) -> Result<()> {
        // Spawn background task for resource monitoring
        let cache = Arc::clone(&self.resource_cache);
        let lightweight = self.config.resource_monitoring.lightweight_monitoring;

        let task = tokio::spawn(async move {
            use sysinfo::System;

            let mut sys = System::new();

            if lightweight {
                // Lightweight monitoring - only refresh memory and CPU
                sys.refresh_memory();
                sys.refresh_cpu();
            } else {
                // Full monitoring - refresh all system info
                sys.refresh_all();
            }

            let memory_usage = sys.used_memory();
            let cpu_usage = sys.global_cpu_info().cpu_usage() as f64;

            let mut cache = cache.lock().await;
            cache.memory_usage = memory_usage;
            cache.cpu_usage = cpu_usage;
            cache.last_updated = Some(Instant::now());
        });

        // For now, wait for the task to complete. In a more advanced implementation,
        // we could make this truly asynchronous and not block the main loop.
        task.await?;
        Ok(())
    }
    pub fn check_resource_limits(&self) -> bool {
        if !self.config.resource_monitoring.enabled {
            return true; // If monitoring is disabled, assume resources are fine
        }

        let memory_limit = self.config.resource_monitoring.memory_warning_threshold;
        let cpu_limit = self.config.resource_monitoring.cpu_warning_threshold;

        self.memory_usage < memory_limit && self.cpu_usage < cpu_limit
    }

    /// Get current memory usage in bytes
    #[allow(dead_code)]
    pub fn memory_usage(&self) -> u64 {
        self.memory_usage
    }

    /// Get current CPU usage as percentage
    #[allow(dead_code)]
    pub fn cpu_usage(&self) -> f64 {
        self.cpu_usage
    }

    /// Check if apply circuit breaker allows attempts
    #[allow(dead_code)]
    pub fn can_attempt_apply(&mut self) -> bool {
        self.apply_circuit_breaker.can_attempt()
    }

    /// Record success in apply circuit breaker
    #[allow(dead_code)]
    pub fn record_apply_success(&mut self) {
        self.apply_circuit_breaker.record_success();
    }

    /// Record failure in apply circuit breaker
    #[allow(dead_code)]
    pub fn record_apply_failure(&mut self) {
        self.apply_circuit_breaker.record_failure();
    }

    /// Get circuit breaker status for monitoring
    #[allow(dead_code)]
    pub fn circuit_breaker_status(&self) -> serde_json::Value {
        serde_json::json!({
            "apply": {
                "state": format!("{:?}", self.apply_circuit_breaker.state),
                "failure_count": self.apply_circuit_breaker.failure_count
            },
            "facts": {
                "state": format!("{:?}", self.facts_circuit_breaker.state),
                "failure_count": self.facts_circuit_breaker.failure_count
            },
            "logs": {
                "state": format!("{:?}", self.logs_circuit_breaker.state),
                "failure_count": self.logs_circuit_breaker.failure_count
            }
        })
    }

    /// Get plugin information for configuration and monitoring
    #[allow(dead_code)]
    pub fn plugin_info(&self) -> serde_json::Value {
        if let Some(pm) = &self.plugin_manager {
            match pm.read() {
                Ok(pm_guard) => match pm_guard.get_available_capabilities() {
                    Ok(capabilities) => capabilities,
                    Err(e) => serde_json::json!({
                        "error": format!("Failed to get plugin capabilities: {}", e)
                    }),
                },
                Err(e) => serde_json::json!({
                    "error": format!("Plugin manager lock poisoned: {}", e)
                }),
            }
        } else {
            serde_json::json!({
                "error": "Plugin manager not initialized"
            })
        }
    }

    /// Run the main event loop
    #[instrument(skip(self))]
    pub async fn run_event_loop(&mut self) -> Result<()> {
        if !self.running {
            return Err(anyhow::anyhow!(
                "Agent must be started before running event loop"
            ));
        }

        info!("Starting agent event loop");
        debug!(
            "Apply interval: {}s, Facts interval: {}s",
            self.config.apply_interval, self.config.facts_interval
        );

        // Create intervals for periodic tasks
        let mut apply_interval = interval(Duration::from_secs(self.config.apply_interval));
        let mut facts_interval = interval(Duration::from_secs(self.config.facts_interval));
        let mut config_check_interval = interval(Duration::from_secs(5)); // Check for config changes every 5 seconds

        // Initial execution with circuit breaker checks
        if self.apply_circuit_breaker.can_attempt() {
            if let Err(e) = self.run_apply_tasks().await {
                error!("Error running apply tasks: {}", e);
                self.apply_circuit_breaker.record_failure();
            } else {
                info!("Apply tasks completed successfully");
                self.apply_circuit_breaker.record_success();
            }
        } else {
            warn!("Apply tasks circuit breaker is open, skipping execution");
        }

        if self.facts_circuit_breaker.can_attempt() {
            if let Err(e) = self.run_facts_collection().await {
                error!("Error collecting facts: {}", e);
                self.facts_circuit_breaker.record_failure();
            } else {
                info!("Facts collection completed successfully");
                self.facts_circuit_breaker.record_success();
            }
        } else {
            warn!("Facts collection circuit breaker is open, skipping execution");
        }

        if self.logs_circuit_breaker.can_attempt() {
            if let Err(e) = self.run_logs_processing().await {
                error!("Error in logs processing: {}", e);
                self.logs_circuit_breaker.record_failure();
            } else {
                info!("Logs processing started successfully");
                self.logs_circuit_breaker.record_success();
            }
        } else {
            warn!("Logs processing circuit breaker is open, skipping execution");
        }

        loop {
            tokio::select! {
                // Apply tasks interval
                _ = apply_interval.tick() => {
                    // Update resource usage (async for better performance)
                    if let Err(e) = self.update_resource_usage().await {
                        warn!("Failed to update resource usage: {}", e);
                    }

                    if !self.check_resource_limits() {
                        warn!("Resource limits exceeded, reducing activity. Memory: {} bytes, CPU: {:.2}%",
                              self.memory_usage, self.cpu_usage);
                        // Could implement backoff or reduced frequency here
                    }

                    if self.apply_circuit_breaker.can_attempt() {
                        let start = Instant::now();
                        if let Err(e) = self.run_apply_tasks().await {
                            error!("Error running apply tasks: {}", e);
                            self.apply_circuit_breaker.record_failure();
                        } else {
                            let duration = start.elapsed();
                            info!("Apply tasks completed successfully in {:?}", duration);
                            self.apply_circuit_breaker.record_success();
                        }
                    } else {
                        warn!("Apply tasks circuit breaker is open, skipping execution");
                    }
                }

                // Facts collection interval
                _ = facts_interval.tick() => {
                    if self.facts_circuit_breaker.can_attempt() {
                        let start = Instant::now();
                        if let Err(e) = self.run_facts_collection().await {
                            error!("Error collecting facts: {}", e);
                            self.facts_circuit_breaker.record_failure();
                        } else {
                            let duration = start.elapsed();
                            info!("Facts collection completed successfully in {:?}", duration);
                            self.facts_circuit_breaker.record_success();
                        }
                    } else {
                        warn!("Facts collection circuit breaker is open, skipping execution");
                    }
                }

                // Check for configuration changes
                _ = config_check_interval.tick() => {
                    if self.config_changed.load(Ordering::Relaxed) {
                        self.config_changed.store(false, Ordering::Relaxed);
                        info!("Configuration change detected, will be applied on next task run");
                        // Note: Configuration reloading is handled in run_apply_tasks and similar methods
                    }

                    // Check logs processing status and restart if needed
                    if self.logs_circuit_breaker.can_attempt() {
                        if let Err(e) = self.run_logs_processing().await {
                            error!("Error in logs processing: {}", e);
                            self.logs_circuit_breaker.record_failure();
                        } else {
                            debug!("Logs processing check completed");
                            self.logs_circuit_breaker.record_success();
                        }
                    } else {
                        warn!("Logs processing circuit breaker is open, skipping execution");
                    }
                }

                // Handle shutdown signal (Ctrl+C)
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        // Graceful shutdown
        self.stop().await?;

        Ok(())
    }

    /// Start the metrics HTTP server
    fn start_metrics_server(&mut self) -> Result<()> {
        use axum::{routing::get, Router};
        use std::net::SocketAddr;
        use tower_http::cors::CorsLayer;

        let registry = self.metrics_registry.clone();
        let app = Router::new()
            .route(
                "/metrics",
                get(move || async move {
                    use prometheus::Encoder;
                    let encoder = prometheus::TextEncoder::new();
                    let metric_families = registry.gather();
                    let mut buffer = Vec::new();
                    encoder.encode(&metric_families, &mut buffer).unwrap();
                    String::from_utf8(buffer).unwrap()
                }),
            )
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.metrics_port));
        println!("Starting metrics server on http://{}", addr);

        let server_handle = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

        self.metrics_server_handle = Some(server_handle);
        Ok(())
    }

    /// Load secrets from environment variables and secret files
    fn load_secrets(&mut self) -> Result<()> {
        // Load from environment variables (DRIFTLESS_SECRET_*)
        for (key, value) in std::env::vars() {
            if key.starts_with("DRIFTLESS_SECRET_") {
                let secret_key = key
                    .strip_prefix("DRIFTLESS_SECRET_")
                    .unwrap()
                    .to_lowercase();
                self.config.secrets.insert(secret_key, value);
            }
        }

        // Load from secret files in config directory
        let secrets_dir = self.config.config_dir.join("secrets");
        if secrets_dir.exists() {
            for entry in std::fs::read_dir(&secrets_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        let contents = std::fs::read_to_string(&path)?;
                        self.config
                            .secrets
                            .insert(filename.to_string(), contents.trim().to_string());
                    }
                }
            }
        }

        println!("Loaded {} secrets", self.config.secrets.len());
        Ok(())
    }

    /// Validate agent configuration
    fn validate_config(&self) -> Result<()> {
        if self.config.apply_interval == 0 {
            return Err(anyhow::anyhow!("apply_interval must be greater than 0"));
        }
        if self.config.facts_interval == 0 {
            return Err(anyhow::anyhow!("facts_interval must be greater than 0"));
        }
        if self.config.metrics_port == 0 {
            return Err(anyhow::anyhow!("metrics_port must be greater than 0"));
        }
        Ok(())
    }

    /// Validate apply configuration
    fn validate_apply_config(&self, config: &ApplyConfig) -> Result<()> {
        if config.tasks.is_empty() {
            return Err(anyhow::anyhow!(
                "apply configuration must contain at least one task"
            ));
        }
        // Additional validation can be added here
        Ok(())
    }

    /// Validate facts configuration
    fn validate_facts_config(&self, config: &FactsConfig) -> Result<()> {
        if config.collectors.is_empty() {
            return Err(anyhow::anyhow!(
                "facts configuration must contain at least one collector"
            ));
        }
        // Additional validation can be added here
        Ok(())
    }

    /// Validate logs configuration
    fn validate_logs_config(&self, config: &LogsConfig) -> Result<()> {
        if config.sources.is_empty() {
            return Err(anyhow::anyhow!(
                "logs configuration must contain at least one source"
            ));
        }
        if config.outputs.is_empty() {
            return Err(anyhow::anyhow!(
                "logs configuration must contain at least one output"
            ));
        }
        // Additional validation can be added here
        Ok(())
    }

    /// Initialize all orchestrators from configuration
    async fn initialize_orchestrators(&mut self) -> Result<()> {
        // Validate agent configuration
        self.validate_config()?;

        // Initialize plugin manager
        self.initialize_plugin_manager()?;

        // Load configurations from the config directory
        let apply_config = self.load_apply_config()?;
        let facts_config = self.load_facts_config()?;
        let logs_config = self.load_logs_config()?;

        // Validate configurations
        if let Some(ref config) = apply_config {
            self.validate_apply_config(config)?;
        }
        if let Some(ref config) = facts_config {
            self.validate_facts_config(config)?;
        }
        if let Some(ref config) = logs_config {
            self.validate_logs_config(config)?;
        }

        // Initialize apply executor
        if let Some(config) = apply_config {
            self.apply_config = Some(config.clone());
            let executor = TaskExecutor::with_vars_from_context(
                self.config.apply_dry_run,
                config.vars.clone(),
                crate::apply::variables::VariableContext::new(),
                self.config.config_dir.clone(),
                self.plugin_manager.clone(),
            );
            self.apply_executor = Some(Arc::new(Mutex::new(executor)));
        }

        // Initialize facts orchestrator
        if let Some(config) = facts_config {
            let orchestrator = FactsOrchestrator::new_with_registry_and_plugins(
                config,
                self.metrics_registry.clone(),
                self.plugin_manager.clone(),
            )?;
            self.facts_collector_count = orchestrator.collector_count();
            self.facts_exporter_count = orchestrator.exporter_count();
            self.facts_orchestrator = Some(Arc::new(Mutex::new(orchestrator)));
        }

        // Initialize logs orchestrator
        if let Some(config) = logs_config {
            let orchestrator =
                LogOrchestrator::new_with_plugins(config, self.plugin_manager.clone());
            self.logs_source_count = orchestrator.source_count();
            self.logs_output_count = orchestrator.output_count();
            self.logs_orchestrator = Some(Arc::new(Mutex::new(orchestrator)));
        }

        Ok(())
    }

    /// Initialize the plugin manager and load plugins
    fn initialize_plugin_manager(&mut self) -> Result<()> {
        // Load plugin registry configuration (including security settings) from the config directory
        let registry_config = crate::config::load_plugin_registry_config(&self.config.config_dir)
            .unwrap_or_else(|_| {
                eprintln!("Warning: Failed to load plugin registry config, using defaults");
                crate::config::PluginRegistryConfig::default()
            });

        let mut plugin_manager = crate::plugins::PluginManager::new_with_security_config(
            self.config.plugin_dir.clone(),
            registry_config.security,
        )
        .map_err(|e| anyhow::anyhow!("Failed to create plugin manager: {}", e))?;

        // Scan for and load plugins
        if let Err(e) = plugin_manager.scan_plugins() {
            eprintln!("Warning: Failed to scan plugins: {}", e);
        }

        if let Err(e) = plugin_manager.load_all_plugins() {
            eprintln!("Warning: Failed to load plugins: {}", e);
        }
        // Register plugin components
        let plugin_manager_arc = Arc::new(RwLock::new(plugin_manager));
        {
            let mut pm = plugin_manager_arc.write().unwrap();
            if let Err(e) = pm.register_plugin_tasks() {
                eprintln!("Warning: Failed to register plugin tasks: {}", e);
            }
            if let Err(e) = pm.register_plugin_facts_collectors() {
                eprintln!("Warning: Failed to register plugin facts collectors: {}", e);
            }
            if let Err(e) = pm.register_plugin_logs_components() {
                eprintln!("Warning: Failed to register plugin logs components: {}", e);
            }
            if let Err(e) = pm.register_plugin_template_extensions(plugin_manager_arc.clone()) {
                eprintln!(
                    "Warning: Failed to register plugin template extensions: {}",
                    e
                );
            }
        }

        self.plugin_manager = Some(plugin_manager_arc);
        Ok(())
    }

    /// Start watching configuration files for changes
    fn start_config_watcher(&mut self) -> Result<()> {
        let config_dir = self.config.config_dir.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Ok(event) = res {
                        if event.kind.is_modify()
                            || event.kind.is_create()
                            || event.kind.is_remove()
                        {
                            let _ = tx.send(()).await;
                        }
                    }
                });
            },
            Config::default(),
        )?;

        watcher.watch(&config_dir, RecursiveMode::Recursive)?;
        self.config_watcher = Some(watcher);

        // Spawn a task to handle configuration changes
        let config_changed = self.config_changed.clone();

        tokio::spawn(async move {
            while rx.recv().await.is_some() {
                println!("Configuration change detected");
                config_changed.store(true, Ordering::Relaxed);
            }
        });

        Ok(())
    }

    /// Reload configuration and restart orchestrators
    /// Load a configuration file of the specified type
    #[allow(dead_code)]
    fn load_config_file<T: serde::de::DeserializeOwned>(
        agent_config: &AgentConfig,
        name: &str,
    ) -> Result<Option<T>> {
        let yaml_path = agent_config.config_dir.join(format!("{}.yml", name));
        let json_path = agent_config.config_dir.join(format!("{}.json", name));

        let contents = if yaml_path.exists() {
            std::fs::read_to_string(&yaml_path)?
        } else if json_path.exists() {
            std::fs::read_to_string(&json_path)?
        } else {
            return Ok(None);
        };

        let config: T = if yaml_path.exists() {
            serde_yaml::from_str(&contents)?
        } else {
            serde_json::from_str(&contents)?
        };

        Ok(Some(config))
    }

    /// Load apply configuration from config directory
    fn load_apply_config(&self) -> Result<Option<ApplyConfig>> {
        let config_path = self.config.config_dir.join("apply.yml");
        if !config_path.exists() {
            let config_path_json = self.config.config_dir.join("apply.json");
            if !config_path_json.exists() {
                return Ok(None);
            }
            // Load from JSON
            let contents = std::fs::read_to_string(&config_path_json)?;
            let config: ApplyConfig = serde_json::from_str(&contents)?;
            return Ok(Some(config));
        }

        // Load from YAML
        let contents = std::fs::read_to_string(&config_path)?;
        let config: ApplyConfig = serde_yaml::from_str(&contents)?;
        Ok(Some(config))
    }

    /// Load apply configuration with change detection
    fn load_apply_config_with_change_detection(&mut self) -> Result<Option<(ApplyConfig, bool)>> {
        let config = match self.load_apply_config()? {
            Some(config) => config,
            None => return Ok(None),
        };

        // Calculate hash of new configuration
        let new_hash = calculate_config_hash(&config)?;

        // Check if configuration has changed
        let changed = match &self.apply_config_hash {
            Some(current_hash) => current_hash != &new_hash,
            None => true, // First load
        };

        if changed {
            self.apply_config_hash = Some(new_hash);
        }

        Ok(Some((config, changed)))
    }

    /// Load facts configuration from config directory
    fn load_facts_config(&self) -> Result<Option<FactsConfig>> {
        let config_path = self.config.config_dir.join("facts.yml");
        if !config_path.exists() {
            let config_path_json = self.config.config_dir.join("facts.json");
            if !config_path_json.exists() {
                return Ok(None);
            }
            // Load from JSON
            let contents = std::fs::read_to_string(&config_path_json)?;
            let config: FactsConfig = serde_json::from_str(&contents)?;
            return Ok(Some(config));
        }

        // Load from YAML
        let contents = std::fs::read_to_string(&config_path)?;
        let config: FactsConfig = serde_yaml::from_str(&contents)?;
        Ok(Some(config))
    }

    /// Load facts configuration with change detection
    fn load_facts_config_with_change_detection(&mut self) -> Result<Option<(FactsConfig, bool)>> {
        let config = match self.load_facts_config()? {
            Some(config) => config,
            None => return Ok(None),
        };

        // Calculate hash of new configuration
        let new_hash = calculate_config_hash(&config)?;

        // Check if configuration has changed
        let changed = match &self.facts_config_hash {
            Some(current_hash) => current_hash != &new_hash,
            None => true, // First load
        };

        if changed {
            self.facts_config_hash = Some(new_hash);
        }

        Ok(Some((config, changed)))
    }

    /// Load logs configuration from config directory
    fn load_logs_config(&self) -> Result<Option<LogsConfig>> {
        let config_path = self.config.config_dir.join("logs.yml");
        if !config_path.exists() {
            let config_path_json = self.config.config_dir.join("logs.json");
            if !config_path_json.exists() {
                return Ok(None);
            }
            // Load from JSON
            let contents = std::fs::read_to_string(&config_path_json)?;
            let config: LogsConfig = serde_json::from_str(&contents)?;
            return Ok(Some(config));
        }

        // Load from YAML
        let contents = std::fs::read_to_string(&config_path)?;
        let config: LogsConfig = serde_yaml::from_str(&contents)?;
        Ok(Some(config))
    }

    /// Load logs configuration with change detection
    fn load_logs_config_with_change_detection(&mut self) -> Result<Option<(LogsConfig, bool)>> {
        let config = match self.load_logs_config()? {
            Some(config) => config,
            None => return Ok(None),
        };

        // Calculate hash of new configuration
        let new_hash = calculate_config_hash(&config)?;

        // Check if configuration has changed
        let changed = match &self.logs_config_hash {
            Some(current_hash) => current_hash != &new_hash,
            None => true, // First load
        };

        if changed {
            self.logs_config_hash = Some(new_hash);
        }

        Ok(Some((config, changed)))
    }

    /// Run apply tasks
    #[instrument(skip(self))]
    async fn run_apply_tasks(&mut self) -> Result<()> {
        if self.apply_executor.is_none() {
            // No apply configuration loaded, skip
            debug!("No apply configuration loaded, skipping apply tasks");
            return Ok(());
        }

        let start_time = Instant::now();
        self.apply_execution_count += 1;
        self.apply_last_execution = Some(start_time);

        info!(
            "Starting apply task execution (run #{})",
            self.apply_execution_count
        );

        // Reload apply configuration in case it changed
        match self.load_apply_config_with_change_detection() {
            Ok(Some((config, changed))) => {
                if changed {
                    // Validate the new config
                    if let Err(e) = self.validate_apply_config(&config) {
                        error!("Apply configuration validation failed: {}", e);
                        self.apply_failure_count += 1;
                        return Err(e);
                    }
                    self.apply_config = Some(config);
                    debug!("Apply configuration reloaded and validated (configuration changed)");
                } else {
                    debug!("Apply configuration unchanged, skipping reload");
                }
            }
            Ok(None) => {
                // Configuration was removed, clear it
                if self.apply_config.is_some() {
                    self.apply_config = None;
                    self.apply_executor = None;
                    self.apply_config_hash = None; // Clear hash
                    info!("Apply configuration removed, stopping apply task execution");
                }
                return Ok(());
            }
            Err(e) => {
                error!("Failed to reload apply configuration: {}", e);
                self.apply_failure_count += 1;
                return Err(e);
            }
        }

        if let (Some(config), Some(executor)) = (&self.apply_config, &self.apply_executor) {
            let mut executor = executor.lock().await;

            match executor.execute(config).await {
                Ok(()) => {
                    self.apply_success_count += 1;
                    let duration = start_time.elapsed();
                    self.apply_last_duration = Some(duration);
                    info!("Apply tasks completed successfully in {:?}", duration);
                    Ok(())
                }
                Err(e) => {
                    self.apply_failure_count += 1;
                    let duration = start_time.elapsed();
                    self.apply_last_duration = Some(duration);
                    error!("Apply tasks failed after {:?}: {}", duration, e);
                    Err(e)
                }
            }
        } else {
            // This shouldn't happen, but handle it gracefully
            error!("Apply configuration or executor missing");
            self.apply_failure_count += 1;
            Ok(())
        }
    }

    /// Run facts collection
    #[instrument(skip(self))]
    async fn run_facts_collection(&mut self) -> Result<()> {
        if self.facts_orchestrator.is_none() {
            // No facts configuration loaded, skip
            debug!("No facts configuration loaded, skipping facts collection");
            return Ok(());
        }

        let start_time = Instant::now();
        self.facts_collection_count += 1;
        self.facts_last_collection = Some(start_time);

        info!(
            "Starting facts collection (run #{})",
            self.facts_collection_count
        );

        // Reload facts configuration in case it changed
        match self.load_facts_config_with_change_detection() {
            Ok(Some((config, changed))) => {
                if changed {
                    // Validate the new config
                    if let Err(e) = self.validate_facts_config(&config) {
                        error!("Facts configuration validation failed: {}", e);
                        self.facts_failure_count += 1;
                        return Err(e);
                    }

                    // Reinitialize orchestrator with new config
                    match FactsOrchestrator::new_with_registry_and_plugins(
                        config,
                        self.metrics_registry.clone(),
                        self.plugin_manager.clone(),
                    ) {
                        Ok(orchestrator) => {
                            self.facts_collector_count = orchestrator.collector_count();
                            self.facts_exporter_count = orchestrator.exporter_count();
                            self.facts_orchestrator = Some(Arc::new(Mutex::new(orchestrator)));
                            debug!("Facts orchestrator reinitialized with new configuration (configuration changed)");
                        }
                        Err(e) => {
                            error!("Failed to create facts orchestrator: {}", e);
                            self.facts_failure_count += 1;
                            return Err(e);
                        }
                    }
                } else {
                    debug!("Facts configuration unchanged, skipping reload");
                }
            }
            Ok(None) => {
                // Configuration was removed, clear it
                if self.facts_orchestrator.is_some() {
                    self.facts_orchestrator = None;
                    self.facts_config_hash = None; // Clear hash
                    info!("Facts configuration removed, stopping facts collection");
                }
                return Ok(());
            }
            Err(e) => {
                error!("Failed to reload facts configuration: {}", e);
                self.facts_failure_count += 1;
                return Err(e);
            }
        }

        if let Some(orchestrator_arc) = &self.facts_orchestrator {
            let orchestrator = orchestrator_arc.lock().await;

            match orchestrator.collect_and_export().await {
                Ok(()) => {
                    self.facts_success_count += 1;
                    let duration = start_time.elapsed();
                    self.facts_last_duration = Some(duration);
                    info!("Facts collection completed successfully in {:?}", duration);
                    Ok(())
                }
                Err(e) => {
                    self.facts_failure_count += 1;
                    let duration = start_time.elapsed();
                    self.facts_last_duration = Some(duration);
                    error!("Facts collection failed after {:?}: {}", duration, e);
                    Err(e)
                }
            }
        } else {
            // This shouldn't happen, but handle it gracefully
            error!("Facts orchestrator missing after initialization");
            self.facts_failure_count += 1;
            Ok(())
        }
    }

    /// Run logs processing
    #[instrument(skip(self))]
    async fn run_logs_processing(&mut self) -> Result<()> {
        if self.logs_orchestrator.is_none() {
            // No logs configuration loaded, skip
            debug!("No logs configuration loaded, skipping logs processing");
            return Ok(());
        }

        // If logs processing is already running, check if config changed
        if self.logs_running {
            // Check if configuration has changed
            match self.load_logs_config_with_change_detection() {
                Ok(Some((config, changed))) => {
                    if changed {
                        // Validate the new config
                        if let Err(e) = self.validate_logs_config(&config) {
                            error!("Logs configuration validation failed: {}", e);
                            self.logs_failure_count += 1;
                            return Err(e);
                        }

                        let needs_restart = match &self.logs_config {
                            Some(current_config) => !configs_are_equal(current_config, &config),
                            None => true, // No current config, so restart needed
                        };

                        if needs_restart {
                            info!("Logs configuration changed, restarting logs processing...");

                            // Stop current logs processing
                            if let Some(orchestrator) = &self.logs_orchestrator {
                                let mut orchestrator = orchestrator.lock().await;
                                if let Err(e) = orchestrator.stop().await {
                                    error!("Error stopping logs orchestrator: {}", e);
                                }
                            }
                            self.logs_running = false;

                            // Reinitialize orchestrator with new config
                            let orchestrator = LogOrchestrator::new_with_plugins(
                                config.clone(),
                                self.plugin_manager.clone(),
                            );
                            self.logs_source_count = orchestrator.source_count();
                            self.logs_output_count = orchestrator.output_count();
                            self.logs_orchestrator = Some(Arc::new(Mutex::new(orchestrator)));
                            self.logs_config = Some(config);
                            debug!("Logs orchestrator reinitialized with new configuration");
                        } else {
                            debug!("Logs configuration unchanged, no restart needed");
                        }
                    } else {
                        debug!("Logs configuration hash unchanged, no reload needed");
                    }
                }
                Ok(None) => {
                    // Configuration was removed, stop logs processing
                    if let Some(orchestrator) = &self.logs_orchestrator {
                        let mut orchestrator = orchestrator.lock().await;
                        if let Err(e) = orchestrator.stop().await {
                            error!("Error stopping logs orchestrator: {}", e);
                        }
                    }
                    self.logs_running = false;
                    self.logs_orchestrator = None;
                    self.logs_config = None;
                    self.logs_config_hash = None; // Clear hash
                    self.logs_source_count = 0;
                    self.logs_output_count = 0;
                    info!("Logs configuration removed, stopping logs processing");
                    return Ok(());
                }
                Err(e) => {
                    error!("Failed to reload logs configuration: {}", e);
                    self.logs_failure_count += 1;
                    return Err(e);
                }
            }
        }

        // Start logs processing if not running
        if !self.logs_running {
            if let Some(orchestrator) = &self.logs_orchestrator {
                let start_time = Instant::now();
                self.logs_start_count += 1;
                self.logs_last_start = Some(start_time);
                self.logs_uptime_start = Some(start_time);

                info!(
                    "Starting logs processing (start #{})",
                    self.logs_start_count
                );

                let mut orchestrator = orchestrator.lock().await;
                match orchestrator.start().await {
                    Ok(()) => {
                        self.logs_success_count += 1;
                        self.logs_running = true;
                        info!("Logs processing started successfully");
                    }
                    Err(e) => {
                        self.logs_failure_count += 1;
                        self.logs_running = false;
                        let duration = start_time.elapsed();
                        self.logs_uptime_start = None;
                        error!(
                            "Logs processing failed to start after {:?}: {}",
                            duration, e
                        );
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.apply_interval, 300);
        assert_eq!(config.facts_interval, 60);
        assert_eq!(config.metrics_port, 8000);
        assert!(config.enabled);
        assert!(!config.apply_dry_run);
        assert!(config.secrets.is_empty());
    }

    #[test]
    fn test_agent_creation() {
        let config = AgentConfig::default();
        let agent = Agent::new(config);
        assert!(!agent.running);
        assert!(agent.apply_executor.is_none());
        assert!(agent.facts_orchestrator.is_none());
        assert!(agent.logs_orchestrator.is_none());
    }
}
