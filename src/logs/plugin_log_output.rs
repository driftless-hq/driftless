//! Plugin log output implementation
//!
//! This module provides functionality for sending logs to plugin-provided outputs.

use crate::logs::{LogEntry, PluginOutput, Result};
use crate::plugins::PluginManager;

/// Plugin-based log output
pub struct PluginLogOutput {
    config: PluginOutput,
    plugin_manager: std::sync::Arc<std::sync::RwLock<PluginManager>>,
}

#[async_trait::async_trait]
impl super::LogOutputWriter for PluginLogOutput {
    /// Write a log entry to the plugin output
    async fn write_entry(&mut self, entry: &super::ShipperLogEntry) -> Result<()> {
        // Convert shipper entry to log entry
        let log_entry = LogEntry {
            raw: entry.message.clone(),
            timestamp: entry.timestamp,
            fields: entry.fields.clone(),
            level: None,
            message: Some(entry.message.clone()),
            source: entry.source.clone(),
            labels: entry.labels.clone(),
        };

        // Execute the plugin output
        let config_value = serde_json::to_value(&self.config.config)?;
        self.plugin_manager
            .read()
            .unwrap()
            .execute_log_output(
                &self.config.plugin_name,
                &self.config.output_name,
                &config_value,
                &log_entry,
            )
            .map_err(|e| anyhow::anyhow!("Plugin output error: {}", e))?;

        Ok(())
    }

    /// Flush any buffered data (no-op for plugin outputs)
    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    /// Close the output (no-op for plugin outputs)
    async fn close(self) -> Result<()> {
        Ok(())
    }
}

/// Create a plugin log output
pub async fn create_plugin_output(
    config: PluginOutput,
    plugin_manager: std::sync::Arc<std::sync::RwLock<PluginManager>>,
) -> Result<Box<dyn super::LogOutputWriter>> {
    Ok(Box::new(PluginLogOutput {
        config,
        plugin_manager,
    }))
}
