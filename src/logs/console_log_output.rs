//! Console log output implementation
//!
//! This module provides console-based log output for stdout/stderr with
//! structured formatting and configurable output targets.

use crate::logs::{ConsoleOutput, ConsoleTarget, ShipperLogEntry};
use anyhow::{Context, Result};
use chrono::Utc;
use std::io::{self, Write};
use tokio::task;

/// Console-based log output with structured formatting
pub struct ConsoleLogOutput {
    #[allow(dead_code)]
    config: ConsoleOutput,
    writer: Box<dyn Write + Send>,
}

impl ConsoleLogOutput {
    /// Create a new console log output
    pub fn new(config: ConsoleOutput) -> Result<Self> {
        let writer: Box<dyn Write + Send> = match config.target {
            ConsoleTarget::Stdout => Box::new(io::stdout()),
            ConsoleTarget::Stderr => Box::new(io::stderr()),
        };

        Ok(Self { config, writer })
    }

    /// Format a log entry for console output
    fn format_entry(&self, entry: &ShipperLogEntry) -> String {
        let timestamp = entry
            .timestamp
            .unwrap_or(Utc::now())
            .format("%Y-%m-%d %H:%M:%S%.3f");

        let source = &entry.source;

        // Format: [timestamp] [source] message
        format!("[{}] [{}] {}", timestamp, source, entry.message)
    }

    /// Format a log entry as JSON for console output
    #[allow(dead_code)]
    fn format_entry_json(&self, entry: &ShipperLogEntry) -> Result<String> {
        let mut json_entry = serde_json::Map::new();

        json_entry.insert("timestamp".to_string(), {
            let ts = entry.timestamp.unwrap_or(Utc::now());
            serde_json::Value::String(ts.to_rfc3339())
        });

        json_entry.insert(
            "source".to_string(),
            serde_json::Value::String(entry.source.clone()),
        );
        json_entry.insert(
            "message".to_string(),
            serde_json::Value::String(entry.message.clone()),
        );

        // Add parsed fields if available
        if !entry.fields.is_empty() {
            json_entry.insert("fields".to_string(), serde_json::json!(entry.fields));
        }

        // Add labels if available
        if !entry.labels.is_empty() {
            json_entry.insert("labels".to_string(), serde_json::json!(entry.labels));
        }

        serde_json::to_string(&json_entry).context("Failed to serialize log entry to JSON")
    }
}

#[async_trait::async_trait]
impl super::LogOutputWriter for ConsoleLogOutput {
    /// Write a log entry to console
    async fn write_entry(&mut self, entry: &ShipperLogEntry) -> Result<()> {
        let formatted = self.format_entry(entry);

        // Use blocking task for synchronous I/O
        task::block_in_place(|| {
            self.writer
                .write_all(formatted.as_bytes())
                .context("Failed to write to console")?;

            self.writer
                .write_all(b"\n")
                .context("Failed to write newline to console")?;

            self.writer
                .flush()
                .context("Failed to flush console output")?;

            Ok(())
        })
    }

    /// Flush any buffered data
    async fn flush(&mut self) -> Result<()> {
        task::block_in_place(|| {
            self.writer
                .flush()
                .context("Failed to flush console output")
        })
    }

    /// Close the console output (no-op for console)
    async fn close(self) -> Result<()> {
        // Console streams don't need explicit closing
        Ok(())
    }
}

/// Create a console output instance
pub fn create_console_output(config: ConsoleOutput) -> Result<Box<dyn super::LogOutputWriter>> {
    Ok(Box::new(ConsoleLogOutput::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::ShipperLogEntry;

    #[test]
    fn test_format_entry() {
        let config = ConsoleOutput {
            name: "test".to_string(),
            enabled: true,
            target: ConsoleTarget::Stdout,
        };

        let output = ConsoleLogOutput::new(config).unwrap();
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());
        let formatted = output.format_entry(&entry);

        assert!(formatted.contains("test message"));
        assert!(formatted.contains("test_source"));
        assert!(formatted.starts_with('['));
        assert!(formatted.contains("] ["));
    }

    #[test]
    fn test_format_entry_json() {
        let config = ConsoleOutput {
            name: "test".to_string(),
            enabled: true,
            target: ConsoleTarget::Stdout,
        };

        let output = ConsoleLogOutput::new(config).unwrap();
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());
        let formatted = output.format_entry_json(&entry).unwrap();

        // Should be valid JSON
        let json: serde_json::Value = serde_json::from_str(&formatted).unwrap();

        assert_eq!(json["message"], "test message");
        assert_eq!(json["source"], "test_source");
        assert!(json["timestamp"].is_string());
    }

    #[test]
    fn test_console_output_creation() {
        let config_stdout = ConsoleOutput {
            name: "stdout".to_string(),
            enabled: true,
            target: ConsoleTarget::Stdout,
        };

        let config_stderr = ConsoleOutput {
            name: "stderr".to_string(),
            enabled: true,
            target: ConsoleTarget::Stderr,
        };

        let output_stdout = ConsoleLogOutput::new(config_stdout);
        assert!(output_stdout.is_ok());

        let output_stderr = ConsoleLogOutput::new(config_stderr);
        assert!(output_stderr.is_ok());
    }

    #[test]
    fn test_create_console_output() {
        let config = ConsoleOutput {
            name: "test".to_string(),
            enabled: true,
            target: ConsoleTarget::Stdout,
        };

        let result = create_console_output(config);
        assert!(result.is_ok());
    }
}
