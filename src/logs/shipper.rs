//! Log shipper
//!
//! This module handles the collection and shipping of logs as defined
//! in the logs schema.

use crate::logs::{LogOutput, LogsConfig};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Log shipper for collecting and forwarding logs
#[allow(dead_code)]
pub struct LogShipper {
    config: LogsConfig,
    plugin_manager: Option<std::sync::Arc<std::sync::RwLock<crate::plugins::PluginManager>>>,
}

impl LogShipper {
    /// Create a new log shipper
    #[allow(dead_code)]
    pub fn new(config: LogsConfig) -> Self {
        Self::new_with_plugins(config, None)
    }

    /// Create a new log shipper with plugin support
    #[allow(dead_code)]
    pub fn new_with_plugins(
        config: LogsConfig,
        plugin_manager: Option<std::sync::Arc<std::sync::RwLock<crate::plugins::PluginManager>>>,
    ) -> Self {
        Self {
            config,
            plugin_manager,
        }
    }

    /// Start the log shipping process
    #[allow(dead_code)]
    pub async fn start(&self) -> Result<()> {
        println!(
            "Starting log shipping with {} sources and {} outputs",
            self.config.sources.len(),
            self.config.outputs.len()
        );

        // Create channels for log processing pipeline
        let (log_tx, _keep_alive_rx) = tokio::sync::broadcast::channel::<LogEntry>(1000);

        // Keep the initial receiver alive for the lifetime of the shipper
        // to prevent send errors when no outputs are subscribed yet

        // Start file tailing tasks for each source
        let mut source_tasks = Vec::new();
        for source in &self.config.sources {
            if source.enabled {
                println!("Starting log source: {}", source.name);

                // Create file log source for actual file tailing
                let source_name = source.name.clone();
                match crate::logs::FileLogSource::new(source.clone()) {
                    Ok(file_source) => {
                        let tx = log_tx.clone();
                        let source_name_clone = source_name.clone();
                        let task = tokio::spawn(async move {
                            println!("Log source '{}' started", source_name);

                            // Create a channel for raw lines from file tailing
                            let (line_tx, mut line_rx) = mpsc::channel::<String>(100);

                            // Start file tailing in a separate task
                            let tail_handle = tokio::spawn(async move {
                                if let Err(e) = file_source.start_tailing(line_tx).await {
                                    eprintln!(
                                        "Error tailing files for source {}: {}",
                                        source_name_clone, e
                                    );
                                }
                            });

                            // Process lines from file tailing
                            while let Some(line) = line_rx.recv().await {
                                // Parse the line into a LogEntry
                                let entry = LogEntry::new(line, source_name.clone());
                                if tx.send(entry).is_err() {
                                    // Receiver closed, stop processing
                                    break;
                                }
                            }

                            // Wait for tailing to finish
                            let _ = tail_handle.await;
                        });
                        source_tasks.push(task);
                    }
                    Err(e) => {
                        eprintln!("Failed to create file source for {}: {}", source_name, e);
                    }
                }
            }
        }

        // Start output forwarding tasks - one task per output for better parallelism
        let mut output_tasks = Vec::new();
        let plugin_manager = self.plugin_manager.clone();

        for output in &self.config.outputs {
            if Self::is_output_enabled_static(output) {
                let output_name = Self::get_output_name_static(output).to_string();
                let config_clone = self.config.clone();
                let output_clone = output.clone();
                let plugin_manager_clone = plugin_manager.clone();
                let log_tx_clone = log_tx.clone();

                let task = tokio::spawn(async move {
                    println!("Log output '{}' started", output_name);
                    let mut output_writer = match Self::create_output_writer_static(
                        &output_clone,
                        &config_clone,
                        plugin_manager_clone,
                    )
                    .await
                    {
                        Ok(writer) => writer,
                        Err(e) => {
                            eprintln!("Failed to create output writer for {}: {}", output_name, e);
                            return;
                        }
                    };

                    // Each output gets its own receiver from the broadcast
                    let mut output_rx = log_tx_clone.subscribe();

                    // Process entries for this specific output
                    while let Ok(entry) = output_rx.recv().await {
                        // Apply filtering for each source
                        let should_forward =
                            Self::should_forward_entry(&config_clone, &entry).await;

                        if should_forward {
                            if let Err(e) = output_writer.write_entry(&entry).await {
                                eprintln!("Error forwarding to output {}: {}", output_name, e);
                            }
                        }
                    }

                    // Flush and close the output
                    if let Err(e) = output_writer.flush().await {
                        eprintln!("Error flushing output {}: {}", output_name, e);
                    }
                    // Note: close() consumes self, so we can't call it on a borrowed value
                    // The writer will be dropped automatically
                });
                output_tasks.push(task);
            }
        }

        // Wait for all tasks to complete while keeping the receiver alive
        // This prevents send errors in sources when no outputs are subscribed
        for task in source_tasks {
            if let Err(e) = task.await {
                eprintln!("Source task error: {:?}", e);
            }
        }

        for task in output_tasks {
            if let Err(e) = task.await {
                eprintln!("Output task error: {:?}", e);
            }
        }

        // Keep _keep_alive_rx alive until tasks complete
        drop(_keep_alive_rx);

        Ok(())
    }

    /// Process a single log entry
    #[allow(dead_code)]
    pub async fn process_log_entry(&self, entry: LogEntry) -> Result<()> {
        // Apply filters and transformations
        let processed_entry = entry;

        // Apply configured filters (same logic as in main processing loop)
        let should_forward = Self::should_forward_entry(&self.config, &processed_entry).await;

        if should_forward {
            // Forward to configured outputs
            for output in &self.config.outputs {
                if self.is_output_enabled(output) {
                    let output_name = self.get_output_name(output);
                    match Self::create_output_writer(self, output, &self.config).await {
                        Ok(mut writer) => {
                            if let Err(e) = writer.write_entry(&processed_entry).await {
                                eprintln!("Error forwarding to output {}: {}", output_name, e);
                            }
                            if let Err(e) = writer.flush().await {
                                eprintln!("Error flushing output {}: {}", output_name, e);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error creating output writer for {}: {}", output_name, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate the log shipping configuration
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<()> {
        println!("Validating log shipping configuration...");

        // Check that we have at least one enabled source and output
        let enabled_sources = self.config.sources.iter().filter(|s| s.enabled).count();
        let enabled_outputs = self
            .config
            .outputs
            .iter()
            .filter(|o| self.is_output_enabled(o))
            .count();

        if enabled_sources == 0 {
            return Err(anyhow::anyhow!("No enabled log sources configured"));
        }

        if enabled_outputs == 0 {
            return Err(anyhow::anyhow!("No enabled log outputs configured"));
        }

        // Validate source configurations
        for source in &self.config.sources {
            if source.enabled && source.paths.is_empty() {
                return Err(anyhow::anyhow!(
                    "Source '{}' has no paths configured",
                    source.name
                ));
            }
        }

        println!(
            "Configuration validated: {} sources, {} outputs",
            enabled_sources, enabled_outputs
        );
        Ok(())
    }

    /// Check if an output is enabled
    #[allow(dead_code)]
    fn is_output_enabled(&self, output: &LogOutput) -> bool {
        use crate::logs::LogOutput::*;

        match output {
            File(o) => o.enabled,
            S3(o) => o.enabled,
            Http(o) => o.enabled,
            Syslog(o) => o.enabled,
            Console(o) => o.enabled,
            Plugin(o) => o.enabled,
        }
    }

    /// Get the name of an output
    #[allow(dead_code)]
    fn get_output_name<'a>(&self, output: &'a LogOutput) -> &'a str {
        use crate::logs::LogOutput::*;

        match output {
            File(o) => &o.name,
            S3(o) => &o.name,
            Http(o) => &o.name,
            Syslog(o) => &o.name,
            Console(o) => &o.name,
            Plugin(o) => &o.output_name,
        }
    }

    /// Check if an entry should be forwarded based on source filters
    async fn should_forward_entry(config: &LogsConfig, entry: &LogEntry) -> bool {
        // Find the source configuration
        if let Some(source) = config.sources.iter().find(|s| s.name == entry.source) {
            // Apply filters
            for filter_config in &source.filters {
                match crate::logs::create_filter(filter_config, None) {
                    Ok(filter) => {
                        // Convert shipper LogEntry to log_parsers LogEntry for filtering
                        let parser_entry = crate::logs::log_parsers::LogEntry {
                            raw: entry.message.clone(),
                            timestamp: entry.timestamp,
                            fields: entry.fields.clone(),
                            level: None, // Not available in shipper entry
                            message: Some(entry.message.clone()),
                            source: entry.source.clone(),
                            labels: entry.labels.clone(),
                        };
                        if let Ok(should_keep) = filter.filter(&parser_entry) {
                            if !should_keep {
                                return false;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creating filter: {}", e);
                    }
                }
            }
        }
        true
    }

    /// Static version of is_output_enabled for use in async contexts
    fn is_output_enabled_static(output: &LogOutput) -> bool {
        use crate::logs::LogOutput::*;

        match output {
            File(o) => o.enabled,
            S3(o) => o.enabled,
            Http(o) => o.enabled,
            Syslog(o) => o.enabled,
            Console(o) => o.enabled,
            Plugin(o) => o.enabled,
        }
    }

    /// Static version of get_output_name for use in async contexts
    fn get_output_name_static(output: &LogOutput) -> &str {
        use crate::logs::LogOutput::*;

        match output {
            File(o) => &o.name,
            S3(o) => &o.name,
            Http(o) => &o.name,
            Syslog(o) => &o.name,
            Console(o) => &o.name,
            Plugin(o) => &o.name,
        }
    }

    /// Create an output writer for the given output configuration
    async fn create_output_writer(
        &self,
        output: &LogOutput,
        _config: &LogsConfig,
    ) -> Result<Box<dyn crate::logs::LogOutputWriter>> {
        use crate::logs::LogOutput::*;

        match output {
            File(config) => crate::logs::create_file_output(config.clone()),
            S3(config) => crate::logs::create_s3_output(config.clone()).await,
            Http(config) => crate::logs::create_http_output(config.clone()).await,
            Syslog(config) => crate::logs::create_syslog_output(config.clone()),
            Console(config) => crate::logs::create_console_output(config.clone()),
            Plugin(config) => {
                if let Some(pm) = &self.plugin_manager {
                    crate::logs::create_plugin_output(config.clone(), pm.clone()).await
                } else {
                    Err(anyhow::anyhow!(
                        "Plugin manager required for plugin outputs"
                    ))
                }
            }
        }
    }

    /// Static version of create_output_writer for use in async contexts
    async fn create_output_writer_static(
        output: &LogOutput,
        _config: &LogsConfig,
        plugin_manager: Option<std::sync::Arc<std::sync::RwLock<crate::plugins::PluginManager>>>,
    ) -> Result<Box<dyn crate::logs::LogOutputWriter>> {
        use crate::logs::LogOutput::*;

        match output {
            File(config) => crate::logs::create_file_output(config.clone()),
            S3(config) => crate::logs::create_s3_output(config.clone()).await,
            Http(config) => crate::logs::create_http_output(config.clone()).await,
            Syslog(config) => crate::logs::create_syslog_output(config.clone()),
            Console(config) => crate::logs::create_console_output(config.clone()),
            Plugin(config) => {
                if let Some(pm) = &plugin_manager {
                    crate::logs::create_plugin_output(config.clone(), pm.clone()).await
                } else {
                    Err(anyhow::anyhow!(
                        "Plugin manager required for plugin outputs"
                    ))
                }
            }
        }
    }
}

/// Processed log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Raw log message
    pub message: String,
    /// Parsed timestamp
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Parsed fields
    pub fields: HashMap<String, serde_json::Value>,
    /// Source that generated this entry
    pub source: String,
    /// Additional labels
    pub labels: HashMap<String, String>,
}

impl LogEntry {
    /// Create a new log entry
    #[allow(dead_code)]
    pub fn new(message: String, source: String) -> Self {
        Self {
            message,
            timestamp: Some(chrono::Utc::now()),
            fields: HashMap::new(),
            source,
            labels: HashMap::new(),
        }
    }

    /// Add a parsed field
    #[allow(dead_code)]
    pub fn with_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.fields.insert(key, value);
        self
    }

    /// Add a label
    #[allow(dead_code)]
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::{FileOutput, LogOutput, LogSource, LogsConfig};

    #[tokio::test]
    async fn test_log_entry_creation() {
        let entry = LogEntry::new("test message".to_string(), "test_source".to_string())
            .with_field("level".to_string(), "INFO".into())
            .with_label("service".to_string(), "myapp".to_string());

        assert_eq!(entry.message, "test message");
        assert_eq!(entry.source, "test_source");
        assert_eq!(entry.fields.get("level").unwrap(), "INFO");
        assert_eq!(entry.labels.get("service").unwrap(), "myapp");
    }

    #[test]
    fn test_config_validation() {
        // Valid config
        let valid_config = LogsConfig {
            global: Default::default(),
            sources: vec![LogSource {
                name: "test_source".to_string(),
                enabled: true,
                paths: vec!["/var/log/test.log".to_string()],
                ..Default::default()
            }],
            outputs: vec![LogOutput::File(FileOutput {
                name: "test_output".to_string(),
                enabled: true,
                path: "/tmp/logs".to_string(),
                ..Default::default()
            })],
            processing: Default::default(),
        };

        let shipper = LogShipper::new(valid_config);
        assert!(shipper.validate().is_ok());

        // Invalid config - no enabled sources
        let invalid_config = LogsConfig {
            global: Default::default(),
            sources: vec![LogSource {
                name: "test_source".to_string(),
                enabled: false,
                paths: vec!["/var/log/test.log".to_string()],
                ..Default::default()
            }],
            outputs: vec![LogOutput::File(FileOutput {
                name: "test_output".to_string(),
                enabled: true,
                path: "/tmp/logs".to_string(),
                ..Default::default()
            })],
            processing: Default::default(),
        };

        let shipper = LogShipper::new(invalid_config);
        assert!(shipper.validate().is_err());
    }
}
