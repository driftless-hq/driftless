//! Log shipper
//!
//! This module handles the collection and shipping of logs as defined
//! in the logs schema.

use crate::logs::{LogsConfig, LogSource, LogOutput};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Log shipper for collecting and forwarding logs
pub struct LogShipper {
    config: LogsConfig,
}

impl LogShipper {
    /// Create a new log shipper
    pub fn new(config: LogsConfig) -> Self {
        Self { config }
    }

    /// Start the log shipping process
    pub async fn start(&self) -> Result<()> {
        println!("Starting log shipping with {} sources and {} outputs",
                 self.config.sources.len(), self.config.outputs.len());

        // Create channels for log processing pipeline
        let (_log_tx, mut _log_rx) = mpsc::channel::<LogEntry>(1000);

        // TODO: Start file tailing tasks for each source
        // TODO: Start output forwarding tasks

        // For now, just show what would be started
        for source in &self.config.sources {
            if source.enabled {
                println!("Would tail logs from: {}", source.name);
                for path in &source.paths {
                    println!("  - {}", path);
                }
            }
        }

        for output in &self.config.outputs {
            if self.is_output_enabled(output) {
                println!("Would forward to: {}", self.get_output_name(output));
            }
        }

        Ok(())
    }

    /// Process a single log entry
    pub async fn process_log_entry(&self, entry: LogEntry) -> Result<()> {
        // TODO: Apply filters and transformations
        // TODO: Forward to configured outputs

        println!("Processing log entry: {}", entry.message);
        Ok(())
    }

    /// Validate the log shipping configuration
    pub fn validate(&self) -> Result<()> {
        println!("Validating log shipping configuration...");

        // Check that we have at least one enabled source and output
        let enabled_sources = self.config.sources.iter().filter(|s| s.enabled).count();
        let enabled_outputs = self.config.outputs.iter().filter(|o| self.is_output_enabled(o)).count();

        if enabled_sources == 0 {
            return Err(anyhow::anyhow!("No enabled log sources configured"));
        }

        if enabled_outputs == 0 {
            return Err(anyhow::anyhow!("No enabled log outputs configured"));
        }

        // Validate source configurations
        for source in &self.config.sources {
            if source.enabled && source.paths.is_empty() {
                return Err(anyhow::anyhow!("Source '{}' has no paths configured", source.name));
            }
        }

        println!("Configuration validated: {} sources, {} outputs",
                 enabled_sources, enabled_outputs);
        Ok(())
    }

    /// Check if an output is enabled
    fn is_output_enabled(&self, output: &LogOutput) -> bool {
        use crate::logs::LogOutput::*;

        match output {
            File(o) => o.enabled,
            S3(o) => o.enabled,
            Http(o) => o.enabled,
            Syslog(o) => o.enabled,
            Console(o) => o.enabled,
        }
    }

    /// Get the name of an output
    fn get_output_name<'a>(&self, output: &'a LogOutput) -> &'a str {
        use crate::logs::LogOutput::*;

        match output {
            File(o) => &o.name,
            S3(o) => &o.name,
            Http(o) => &o.name,
            Syslog(o) => &o.name,
            Console(o) => &o.name,
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
    pub fn with_field(mut self, key: String, value: serde_json::Value) -> Self {
        self.fields.insert(key, value);
        self
    }

    /// Add a label
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
}

/// File tailer for monitoring log files
pub struct FileTailer {
    source: LogSource,
}

impl FileTailer {
    /// Create a new file tailer
    pub fn new(source: LogSource) -> Self {
        Self { source }
    }

    /// Start tailing the configured files
    pub async fn start_tailing(&self) -> Result<()> {
        println!("Starting to tail {} files for source: {}",
                 self.source.paths.len(), self.source.name);

        // TODO: Implement actual file watching and tailing
        // This would use notify crate for file changes and tokio for async processing

        for path in &self.source.paths {
            println!("Would tail file: {}", path);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::{LogsConfig, LogSource, LogOutput, FileOutput};

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