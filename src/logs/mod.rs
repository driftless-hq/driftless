//! Logs processing module
//!
//! This module provides functionality for log collection, processing, and shipping.
//! It supports various sources (files, network) and outputs (files, S3, HTTP, syslog, console).
//!
//! # Examples
//!
//! ## File log output
//!
//! **YAML Format:**
//! ```yaml
//! logs:
//!   - type: file
//!     path: /var/log/app.log
//!     format: json
//!     rotation:
//!       size: 10MB
//!       count: 5
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "logs": [
//!     {
//!       "type": "file",
//!       "path": "/var/log/app.log",
//!       "format": "json",
//!       "rotation": {
//!         "size": "10MB",
//!         "count": 5
//!       }
//!     }
//!   ]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[logs]]
//! type = "file"
//! path = "/var/log/app.log"
//! format = "json"
//!
//! [logs.rotation]
//! size = "10MB"
//! count = 5
//! ```
//!
//! ## Console log output
//!
//! **YAML Format:**
//! ```yaml
//! logs:
//!   - type: console
//!     format: text
//!     level: info
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "logs": [
//!     {
//!       "type": "console",
//!       "format": "text",
//!       "level": "info"
//!     }
//!   ]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[logs]]
//! type = "console"
//! format = "text"
//! level = "info"
//! ```
//!
//! ## Syslog log output
//!
//! **YAML Format:**
//! ```yaml
//! logs:
//!   - type: syslog
//!     facility: local0
//!     severity: info
//!     tag: driftless
//!     server: 127.0.0.1:514
//!     protocol: udp
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "logs": [
//!     {
//!       "type": "syslog",
//!       "facility": "local0",
//!       "severity": "info",
//!       "tag": "driftless",
//!       "server": "127.0.0.1:514",
//!       "protocol": "udp"
//!     }
//!   ]
//! }
//! ```
//!
//! **TOML Format:**
//! ```toml
//! [[logs]]
//! type = "syslog"
//! facility = "local0"
//! severity = "info"
//! tag = "driftless"
//! server = "127.0.0.1:514"
//! protocol = "udp"
//! ```

mod console_log_output;
mod file_log_output;
mod file_log_source;
mod http_log_output;
mod log_filters;
mod log_parsers;
mod orchestrator;
mod plugin_log_output;
mod s3_log_output;
mod shipper;
mod syslog_log_output;

use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Type alias for log source functions
type LogSourceFn = Arc<dyn Fn(&LogSource) -> Result<Box<dyn std::io::Read + Send>> + Send + Sync>;

// Type alias for log output functions
type LogOutputFn =
    Arc<dyn Fn(&LogOutput, Box<dyn std::io::Read + Send>) -> Result<()> + Send + Sync>;

// Type alias for log processor functions (used internally by registry)
type LogProcessorFn =
    Arc<dyn Fn(&serde_yaml::Value, Box<dyn std::io::Read + Send>) -> Result<()> + Send + Sync>;

// Log registry entry containing source/output function and metadata
#[derive(Clone)]
pub(crate) struct LogRegistryEntry {
    function: LogProcessorFn,
    category: String,
    description: String,
    is_source: bool, // true for sources, false for outputs
    filename: String,
}

// Global logs registry for extensible log processing
static LOGS_REGISTRY: Lazy<RwLock<HashMap<String, LogRegistryEntry>>> = Lazy::new(|| {
    let mut registry = HashMap::new();

    // Initialize with built-in sources and outputs
    LogsRegistry::initialize_builtin_processors(&mut registry);

    RwLock::new(registry)
});

/// Logs processor registry for runtime extensibility
pub struct LogsRegistry;

impl LogsRegistry {
    /// Register a log source function with the global registry
    #[allow(unused)]
    pub fn register_log_source(
        source_type: &str,
        category: &str,
        description: &str,
        filename: &str,
        source_fn: LogSourceFn,
    ) {
        let function = Arc::new(
            move |config: &serde_yaml::Value, _reader: Box<dyn std::io::Read + Send>| {
                // Parse the config and call the actual source function
                // For sources in the registry, we ignore the input reader and create a new one
                let source_config: LogSource = serde_yaml::from_value(config.clone())?;
                let _new_reader = source_fn(&source_config)?;
                // In a full pipeline, this reader would be passed to the next processor
                // For now, we just ensure the source function works
                Ok(())
            },
        );

        let entry = LogRegistryEntry {
            function,
            category: category.to_string(),
            description: description.to_string(),
            is_source: true,
            filename: filename.to_string(),
        };
        let mut registry = LOGS_REGISTRY.write().unwrap();
        registry.insert(source_type.to_string(), entry);
    }

    /// Register a log output function with the global registry
    #[allow(unused)]
    pub fn register_log_output(
        output_type: &str,
        category: &str,
        description: &str,
        filename: &str,
        output_fn: LogOutputFn,
    ) {
        let function = Arc::new(
            move |config: &serde_yaml::Value, reader: Box<dyn std::io::Read + Send>| {
                // Parse the config and call the actual output function
                let output_config: LogOutput = serde_yaml::from_value(config.clone())?;
                output_fn(&output_config, reader)
            },
        );

        let entry = LogRegistryEntry {
            function,
            category: category.to_string(),
            description: description.to_string(),
            is_source: false,
            filename: filename.to_string(),
        };
        let mut registry = LOGS_REGISTRY.write().unwrap();
        registry.insert(output_type.to_string(), entry);
    }

    /// Register a log source function
    pub(crate) fn register_source(
        registry: &mut HashMap<String, LogRegistryEntry>,
        source_type: &str,
        category: &str,
        description: &str,
        filename: &str,
        _source_fn: LogSourceFn,
    ) {
        let function = Arc::new(
            move |config: &serde_yaml::Value, _reader: Box<dyn std::io::Read + Send>| {
                // For sources, we create a reader from the config
                // This is a simplified implementation - in practice, this would be more complex
                let _source_config: LogSource = serde_yaml::from_value(config.clone())?;
                Ok(())
            },
        );

        let entry = LogRegistryEntry {
            function,
            category: category.to_string(),
            description: description.to_string(),
            is_source: true,
            filename: filename.to_string(),
        };
        registry.insert(source_type.to_string(), entry);
    }

    /// Register a log output function
    pub(crate) fn register_output(
        registry: &mut HashMap<String, LogRegistryEntry>,
        output_type: &str,
        category: &str,
        description: &str,
        filename: &str,
        _output_fn: LogOutputFn,
    ) {
        let function = Arc::new(
            move |_config: &serde_yaml::Value, _reader: Box<dyn std::io::Read + Send>| {
                // For outputs, we process the reader
                // This is a simplified implementation - in practice, this would be more complex
                let _output_config: LogOutput = serde_yaml::from_value(_config.clone())?;
                Ok(())
            },
        );

        let entry = LogRegistryEntry {
            function,
            category: category.to_string(),
            description: description.to_string(),
            is_source: false,
            filename: filename.to_string(),
        };
        registry.insert(output_type.to_string(), entry);
    }

    /// Initialize the registry with built-in log processors
    pub(crate) fn initialize_builtin_processors(registry: &mut HashMap<String, LogRegistryEntry>) {
        // File log source
        LogsRegistry::register_source(
            registry,
            "file",
            "Log Sources",
            "Tail log files with rotation handling and encoding support",
            "file_log_source",
            Arc::new(|source| {
                // For registry processing, read the file content, parse it, and return structured data
                use std::io::Cursor;

                if source.paths.is_empty() {
                    return Ok(Box::new(Cursor::new("[]")) as Box<dyn std::io::Read + Send>);
                }

                // Read the first file
                let path = &source.paths[0];
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        // Parse the file content into log entries
                        let mut entries = Vec::new();
                        for line in content.lines() {
                            if !line.trim().is_empty() {
                                let entry = crate::logs::LogEntry {
                                    raw: line.to_string(),
                                    timestamp: Some(chrono::Utc::now()),
                                    fields: HashMap::new(),
                                    level: None,
                                    message: Some(line.to_string()),
                                    source: source.name.clone(),
                                    labels: source.labels.clone(),
                                };
                                entries.push(entry);
                            }
                        }

                        // Serialize to JSON for the reader
                        let json =
                            serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string());
                        Ok(Box::new(Cursor::new(json)) as Box<dyn std::io::Read + Send>)
                    }
                    Err(_) => Ok(Box::new(Cursor::new("[]")) as Box<dyn std::io::Read + Send>),
                }
            }),
        );

        // File log output
        LogsRegistry::register_output(
            registry,
            "file",
            "Log Outputs",
            "Write logs to files with rotation and compression",
            "mod",
            Arc::new(|output, mut reader| {
                if let LogOutput::File(file_config) = output {
                    use std::fs::OpenOptions;
                    use std::io::Write;

                    let mut file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&file_config.path)?;

                    // Read JSON data from reader and deserialize log entries
                    let mut buffer = String::new();
                    reader.read_to_string(&mut buffer)?;

                    if let Ok(entries) = serde_json::from_str::<Vec<crate::logs::LogEntry>>(&buffer)
                    {
                        // Format and write each entry
                        for entry in entries {
                            let formatted = format!(
                                "[{}] {}: {}",
                                entry
                                    .timestamp
                                    .map(|t| t.to_rfc3339())
                                    .unwrap_or_else(|| "unknown".to_string()),
                                entry.source,
                                entry.message.unwrap_or_else(|| entry.raw.clone())
                            );
                            writeln!(file, "{}", formatted)?;
                        }
                    } else {
                        // If not JSON, write as plain text
                        writeln!(file, "{}", buffer.trim())?;
                    }
                }
                Ok(())
            }),
        );

        // S3 log output
        LogsRegistry::register_output(
            registry,
            "s3",
            "Log Outputs",
            "Upload logs to S3 with batching and compression",
            "mod",
            Arc::new(|output, mut reader| {
                if let LogOutput::S3(s3_config) = output {
                    // Read and format log entries
                    let mut buffer = String::new();
                    reader.read_to_string(&mut buffer)?;

                    let formatted_logs = if let Ok(entries) =
                        serde_json::from_str::<Vec<crate::logs::LogEntry>>(&buffer)
                    {
                        entries
                            .into_iter()
                            .map(|entry| {
                                format!(
                                    "[{}] {}: {}",
                                    entry
                                        .timestamp
                                        .map(|t| t.to_rfc3339())
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    entry.source,
                                    entry.message.unwrap_or_else(|| entry.raw.clone())
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        buffer
                    };

                    println!(
                        "Registry: Would upload {} bytes to S3 bucket {} with key prefix '{}'",
                        formatted_logs.len(),
                        s3_config.bucket,
                        s3_config.prefix
                    );
                }
                Ok(())
            }),
        );

        // HTTP log output
        LogsRegistry::register_output(
            registry,
            "http",
            "Log Outputs",
            "Send logs to HTTP endpoints with authentication and retry",
            "mod",
            Arc::new(|output, mut reader| {
                if let LogOutput::Http(http_config) = output {
                    // Read and format log entries
                    let mut buffer = String::new();
                    reader.read_to_string(&mut buffer)?;

                    let formatted_logs = if let Ok(entries) =
                        serde_json::from_str::<Vec<crate::logs::LogEntry>>(&buffer)
                    {
                        entries
                            .into_iter()
                            .map(|entry| {
                                format!(
                                    "[{}] {}: {}",
                                    entry
                                        .timestamp
                                        .map(|t| t.to_rfc3339())
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    entry.source,
                                    entry.message.unwrap_or_else(|| entry.raw.clone())
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        buffer
                    };

                    println!(
                        "Registry: Would send {} bytes to HTTP endpoint {} with method {}",
                        formatted_logs.len(),
                        http_config.url,
                        http_config.method
                    );
                }
                Ok(())
            }),
        );

        // Syslog output
        LogsRegistry::register_output(
            registry,
            "syslog",
            "Log Outputs",
            "Send logs to syslog with RFC compliance",
            "mod",
            Arc::new(|output, mut reader| {
                if let LogOutput::Syslog(syslog_config) = output {
                    // Read and format log entries
                    let mut buffer = String::new();
                    reader.read_to_string(&mut buffer)?;

                    let formatted_logs = if let Ok(entries) =
                        serde_json::from_str::<Vec<crate::logs::LogEntry>>(&buffer)
                    {
                        entries
                            .into_iter()
                            .map(|entry| {
                                format!(
                                    "[{}] {}: {}",
                                    entry
                                        .timestamp
                                        .map(|t| t.to_rfc3339())
                                        .unwrap_or_else(|| "unknown".to_string()),
                                    entry.source,
                                    entry.message.unwrap_or_else(|| entry.raw.clone())
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    } else {
                        buffer
                    };

                    println!(
                        "Registry: Would send {} bytes to syslog facility {:?} with severity {:?}",
                        formatted_logs.len(),
                        syslog_config.facility,
                        syslog_config.severity
                    );
                }
                Ok(())
            }),
        );

        // Console output
        LogsRegistry::register_output(
            registry,
            "console",
            "Log Outputs",
            "Output logs to stdout/stderr for debugging",
            "mod",
            Arc::new(|output, mut reader| {
                if let LogOutput::Console(console_config) = output {
                    use std::io::{self, Write};

                    // Read and format log entries
                    let mut buffer = String::new();
                    reader.read_to_string(&mut buffer)?;

                    let mut output_stream: Box<dyn Write> = match console_config.target {
                        ConsoleTarget::Stdout => Box::new(io::stdout()),
                        ConsoleTarget::Stderr => Box::new(io::stderr()),
                    };

                    if let Ok(entries) = serde_json::from_str::<Vec<crate::logs::LogEntry>>(&buffer)
                    {
                        // Format and write each entry
                        for entry in entries {
                            let formatted = format!(
                                "[{}] {}: {}",
                                entry
                                    .timestamp
                                    .map(|t| t.to_rfc3339())
                                    .unwrap_or_else(|| "unknown".to_string()),
                                entry.source,
                                entry.message.unwrap_or_else(|| entry.raw.clone())
                            );
                            writeln!(output_stream, "{}", formatted)?;
                        }
                    } else {
                        // If not JSON, write as plain text
                        writeln!(output_stream, "{}", buffer.trim())?;
                    }
                }
                Ok(())
            }),
        );
    }

    /// Get all registered processor types
    pub fn get_registered_processor_types() -> Vec<String> {
        let registry = LOGS_REGISTRY.read().unwrap();
        registry.keys().cloned().collect()
    }

    /// Get the category for a processor type
    #[allow(unused)]
    pub fn get_processor_category(processor_type: &str) -> String {
        let registry = LOGS_REGISTRY.read().unwrap();
        registry
            .get(processor_type)
            .map(|e| e.category.clone())
            .unwrap_or_else(|| "Other".to_string())
    }

    /// Get the description for a processor type
    pub fn get_processor_description(processor_type: &str) -> String {
        let registry = LOGS_REGISTRY.read().unwrap();
        registry
            .get(processor_type)
            .map(|e| e.description.clone())
            .unwrap_or_else(|| "Unknown processor type".to_string())
    }

    /// Check if a processor type is a source
    #[allow(unused)]
    pub fn is_source_processor(processor_type: &str) -> bool {
        let registry = LOGS_REGISTRY.read().unwrap();
        registry
            .get(processor_type)
            .map(|e| e.is_source)
            .unwrap_or(false)
    }

    /// Get the filename for a processor type
    #[allow(unused)]
    pub fn get_processor_filename(processor_type: &str) -> String {
        let registry = LOGS_REGISTRY.read().unwrap();
        registry
            .get(processor_type)
            .map(|e| e.filename.clone())
            .unwrap_or_else(|| "mod".to_string())
    }

    /// Process logs using a registered processor
    #[allow(unused)]
    pub fn process_logs(
        processor_type: &str,
        config: &serde_yaml::Value,
        reader: Box<dyn std::io::Read + Send>,
    ) -> Result<()> {
        let entry = {
            let registry = LOGS_REGISTRY.read().unwrap();
            registry.get(processor_type).cloned()
        };

        if let Some(entry) = entry {
            (entry.function)(config, reader)
        } else {
            Err(anyhow::anyhow!(
                "No processor registered for type: {}",
                processor_type
            ))
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsConfig {
    /// Global settings for log collection
    #[serde(default)]
    pub global: GlobalSettings,
    /// List of log sources to tail
    pub sources: Vec<LogSource>,
    /// List of outputs for forwarding logs
    pub outputs: Vec<LogOutput>,
    /// Processing pipeline configuration
    #[serde(default)]
    pub processing: ProcessingConfig,
}

impl LogsConfig {
    /// Merge another LogsConfig into this one
    pub fn merge(&mut self, other: LogsConfig) {
        // Merge global settings (other takes precedence for simple fields)
        if !other.global.enabled {
            self.global.enabled = other.global.enabled;
        }
        if other.global.buffer_size != default_buffer_size() {
            self.global.buffer_size = other.global.buffer_size;
        }
        if other.global.flush_interval != default_flush_interval() {
            self.global.flush_interval = other.global.flush_interval;
        }
        // Merge labels (other labels take precedence)
        for (key, value) in other.global.labels {
            self.global.labels.insert(key, value);
        }

        // Merge sources and outputs (extend the lists)
        self.sources.extend(other.sources);
        self.outputs.extend(other.outputs);

        // Merge processing config (other takes precedence)
        if other.processing.enabled {
            self.processing.enabled = true;
        }
        self.processing
            .global_filters
            .extend(other.processing.global_filters);
        self.processing
            .transformations
            .extend(other.processing.transformations);
    }
}

/// Global settings for log collection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalSettings {
    /// Whether log collection is enabled globally
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Default buffer size for log lines
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Default flush interval (seconds)
    #[serde(default = "default_flush_interval")]
    pub flush_interval: u64,
    /// Global labels to add to all log entries
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Log source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource {
    /// Unique name for this log source
    pub name: String,
    /// Type of log source (file, plugin, etc.)
    #[serde(default = "default_source_type")]
    pub source_type: String,
    /// Whether this source is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Path(s) to log files to tail
    pub paths: Vec<String>,
    /// File reading options
    #[serde(default)]
    pub file_options: FileOptions,
    /// Parser configuration
    #[serde(default)]
    pub parser: ParserConfig,
    /// Filters to apply
    #[serde(default)]
    pub filters: Vec<FilterConfig>,
    /// Output destinations for this source (if not using global outputs)
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Additional labels for this source
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Plugin name (for plugin sources)
    #[serde(default)]
    pub plugin_name: Option<String>,
    /// Plugin source name (for plugin sources)
    #[serde(default)]
    pub plugin_source_name: Option<String>,
}

impl Default for LogSource {
    fn default() -> Self {
        Self {
            name: String::new(),
            source_type: "file".to_string(),
            enabled: true,
            paths: Vec::new(),
            file_options: FileOptions::default(),
            parser: ParserConfig::default(),
            filters: Vec::new(),
            outputs: Vec::new(),
            labels: HashMap::new(),
            plugin_name: None,
            plugin_source_name: None,
        }
    }
}

/// File reading options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileOptions {
    /// Start reading from beginning of file (default: end)
    #[serde(default)]
    pub from_beginning: bool,
    /// Follow file rotation
    #[serde(default = "default_true")]
    pub follow_rotated: bool,
    /// Maximum file size to read (bytes, 0 = unlimited)
    #[serde(default)]
    pub max_file_size: u64,
    /// Encoding of the log file
    #[serde(default)]
    pub encoding: FileEncoding,
    /// How to handle file not found errors
    #[serde(default)]
    pub missing_file_handling: MissingFileHandling,
}

/// File encoding
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FileEncoding {
    /// UTF-8 encoding
    #[default]
    Utf8,
    /// ASCII encoding
    Ascii,
    /// Latin-1 encoding
    Latin1,
}

/// How to handle missing files
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MissingFileHandling {
    /// Skip and continue
    Skip,
    /// Warn and continue
    #[default]
    Warn,
    /// Error and stop
    Error,
}

/// Parser configuration for log entries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParserConfig {
    /// Parser type
    #[serde(default)]
    pub parser_type: ParserType,
    /// Time format for timestamp parsing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_format: Option<String>,
    /// Custom regex pattern for parsing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Field mappings for structured logs
    #[serde(default)]
    pub field_map: HashMap<String, String>,
    /// Multiline log handling
    #[serde(default)]
    pub multiline: MultilineConfig,
}

/// Parser types
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParserType {
    /// Plain text (no parsing)
    #[default]
    Plain,
    /// JSON structured logs
    Json,
    /// Key-value pairs (key=value format)
    KeyValue,
    /// Apache common log format
    ApacheCommon,
    /// Apache combined log format
    ApacheCombined,
    /// Nginx access log format
    Nginx,
    /// Syslog format (RFC 3164/5424)
    Syslog,
    /// Custom regex pattern
    Regex,
    /// Plugin-provided parser
    Plugin(PluginParser),
}

/// Multiline log configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultilineConfig {
    /// Whether multiline parsing is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Pattern to match lines that indicate the start of a new log entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_pattern: Option<String>,
    /// Pattern to match lines that should be combined with the previous line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_pattern: Option<String>,
    /// Pattern to match lines that should end a multiline entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_pattern: Option<String>,
    /// How to handle multiline matching
    #[serde(default)]
    pub match_type: MultilineMatchType,
    /// Whether to negate the pattern match (invert the logic)
    #[serde(default)]
    pub negate: bool,
    /// Maximum number of lines per multiline entry
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    /// Timeout for multiline assembly (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// Plugin-provided parser configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginParser {
    /// Plugin parser name
    pub name: String,
    /// Parser-specific configuration
    #[serde(flatten)]
    pub config: serde_yaml::Value,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FilterConfig {
    /// Include only lines matching regex
    Include {
        pattern: String,
        case_sensitive: Option<bool>,
    },
    /// Exclude lines matching regex
    Exclude {
        pattern: String,
        case_sensitive: Option<bool>,
    },
    /// Include lines containing any of the specified strings
    Contains {
        values: Vec<String>,
        case_sensitive: Option<bool>,
    },
    /// Exclude lines containing any of the specified strings
    NotContains {
        values: Vec<String>,
        case_sensitive: Option<bool>,
    },
    /// Include lines where field matches value
    FieldMatch {
        field: String,
        value: String,
        case_sensitive: Option<bool>,
    },
    /// Drop lines above a certain rate (rate limiting)
    RateLimit { events_per_second: u32 },
    /// Plugin-provided filter
    Plugin(PluginFilter),
}

/// Plugin-provided filter configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginFilter {
    /// Plugin filter name
    pub name: String,
    /// Filter-specific configuration
    #[serde(flatten)]
    pub config: serde_yaml::Value,
}

/// Log output configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LogOutput {
    /// Write to local file
    File(FileOutput),
    /// Write to S3 bucket
    S3(S3Output),
    /// Send to HTTP endpoint (ELK stack, etc.)
    Http(HttpOutput),
    /// Send to syslog
    Syslog(SyslogOutput),
    /// Send to stdout/stderr (for debugging)
    Console(ConsoleOutput),
    /// Plugin-provided output
    Plugin(PluginOutput),
}

/// Plugin-provided output configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginOutput {
    /// Plugin name that provides this output
    pub plugin_name: String,
    /// Plugin output name
    pub output_name: String,
    /// Combined name for display (plugin_name/output_name)
    pub name: String,
    /// Whether this output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output-specific configuration
    #[serde(flatten)]
    pub config: serde_yaml::Value,
}

/// File output configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FileOutput {
    /// Output destination name
    pub name: String,
    /// Whether this output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output directory path
    pub path: String,
    /// Filename pattern (can include date/time variables)
    #[serde(default = "default_filename_pattern")]
    pub filename_pattern: String,
    /// File rotation configuration
    #[serde(default)]
    pub rotation: RotationConfig,
    /// Compression configuration
    #[serde(default)]
    pub compression: CompressionConfig,
}

/// S3 output configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct S3Output {
    /// Output destination name
    pub name: String,
    /// Whether this output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// S3 bucket name
    pub bucket: String,
    /// S3 region
    pub region: String,
    /// S3 key prefix
    #[serde(default = "default_s3_prefix")]
    pub prefix: String,
    /// Upload interval (seconds)
    #[serde(default = "default_upload_interval")]
    pub upload_interval: u64,
    /// Maximum batch size before upload
    #[serde(default = "default_s3_batch_size")]
    pub batch_size: usize,
    /// Compression configuration
    #[serde(default)]
    pub compression: CompressionConfig,
    /// S3 credentials (if not using environment/default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
}

/// HTTP output configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct HttpOutput {
    /// Output destination name
    pub name: String,
    /// Whether this output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// HTTP endpoint URL
    pub url: String,
    /// HTTP method
    #[serde(default = "default_http_method")]
    pub method: String,
    /// HTTP headers to include
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Authentication configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<HttpAuth>,
    /// Batch configuration
    #[serde(default)]
    pub batch: BatchConfig,
    /// Compression configuration
    #[serde(default)]
    pub compression: CompressionConfig,
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
}

/// HTTP authentication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HttpAuth {
    /// Basic authentication
    Basic { username: String, password: String },
    /// Bearer token authentication
    Bearer { token: String },
    /// API key authentication
    ApiKey { header_name: String, key: String },
}

/// Syslog output configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyslogOutput {
    /// Output destination name
    pub name: String,
    /// Whether this output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Syslog facility
    #[serde(default = "default_syslog_facility")]
    pub facility: String,
    /// Syslog severity
    #[serde(default = "default_syslog_severity")]
    pub severity: String,
    /// Syslog tag
    #[serde(default = "default_syslog_tag")]
    pub tag: String,
    /// Syslog server (host:port)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// Protocol (tcp/udp)
    #[serde(default)]
    pub protocol: SyslogProtocol,
}

/// Syslog protocol
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SyslogProtocol {
    /// UDP protocol
    #[default]
    Udp,
    /// TCP protocol
    Tcp,
}

/// Console output configuration (for debugging)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsoleOutput {
    /// Output destination name
    pub name: String,
    /// Whether this output is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output to stdout or stderr
    #[serde(default)]
    pub target: ConsoleTarget,
}

/// Console target
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleTarget {
    /// Standard output
    #[default]
    Stdout,
    /// Standard error
    Stderr,
}

/// File rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RotationConfig {
    /// Rotation strategy
    #[serde(default)]
    pub strategy: RotationStrategy,
    /// Maximum file size before rotation (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<u64>,
    /// Maximum age before rotation (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,
    /// Maximum number of files to keep
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

/// Rotation strategies
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RotationStrategy {
    /// No rotation
    None,
    /// Rotate by size
    #[default]
    Size,
    /// Rotate by time
    Time,
    /// Rotate by size or time (whichever comes first)
    SizeOrTime,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CompressionConfig {
    /// Whether compression is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Compression algorithm
    #[serde(default)]
    pub algorithm: CompressionAlgorithm,
    /// Compression level (1-9, higher = better compression but slower)
    #[serde(default = "default_compression_level")]
    pub level: u32,
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    /// Gzip compression
    #[default]
    Gzip,
    /// Zlib compression
    Zlib,
    /// No compression
    None,
}

/// Batch configuration for HTTP outputs
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BatchConfig {
    /// Maximum batch size (number of log entries)
    #[serde(default = "default_batch_size")]
    pub max_size: usize,
    /// Maximum batch age (seconds)
    #[serde(default = "default_batch_age")]
    pub max_age: u64,
    /// Maximum batch size in bytes
    #[serde(default = "default_batch_bytes")]
    pub max_bytes: usize,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_attempts: u32,
    /// Initial backoff delay (seconds)
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff: u64,
    /// Maximum backoff delay (seconds)
    #[serde(default = "default_max_backoff")]
    pub max_backoff: u64,
}

/// Processing pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessingConfig {
    /// Whether processing pipeline is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Global filters applied to all sources
    #[serde(default)]
    pub global_filters: Vec<FilterConfig>,
    /// Field transformations
    #[serde(default)]
    pub transformations: Vec<TransformationConfig>,
}

/// Field transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransformationConfig {
    /// Add a field with a static value
    AddField { field: String, value: String },
    /// Remove a field
    RemoveField { field: String },
    /// Rename a field
    RenameField { from: String, to: String },
    /// Copy field value to another field
    CopyField { from: String, to: String },
    /// Set field value conditionally
    SetFieldIf {
        field: String,
        value: String,
        condition: ConditionConfig,
    },
}

/// Condition configuration for transformations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionConfig {
    /// Field to check
    pub field: String,
    /// Operator for comparison
    pub op: ConditionOperator,
    /// Value to compare against
    pub value: String,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConditionOperator {
    /// Equal to
    Eq,
    /// Not equal to
    Ne,
    /// Contains substring
    Contains,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Matches regex
    Matches,
    /// Exists (field is present)
    Exists,
}

// Default value functions
fn default_true() -> bool {
    true
}
fn default_source_type() -> String {
    "file".to_string()
}
fn default_buffer_size() -> usize {
    8192
}
fn default_flush_interval() -> u64 {
    30
}
fn default_max_lines() -> usize {
    100
}
fn default_filename_pattern() -> String {
    "%Y-%m-%d-%H-%M-%S.log".to_string()
}
fn default_s3_prefix() -> String {
    "logs/".to_string()
}
fn default_upload_interval() -> u64 {
    300
}
fn default_s3_batch_size() -> usize {
    1000
}
fn default_http_method() -> String {
    "POST".to_string()
}
fn default_syslog_facility() -> String {
    "local0".to_string()
}
fn default_syslog_severity() -> String {
    "info".to_string()
}
fn default_syslog_tag() -> String {
    "driftless".to_string()
}
fn default_max_files() -> usize {
    10
}
pub fn default_compression_level() -> u32 {
    6
}

/// Extract plugin name from configuration
///
/// Prefers an explicit 'plugin' field from the configuration,
/// falling back to the provided name to preserve existing behavior.
pub fn extract_plugin_name<'a>(config: &'a serde_yaml::Value, fallback_name: &'a str) -> &'a str {
    match config {
        serde_yaml::Value::Mapping(map) => map
            .get(serde_yaml::Value::String("plugin".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or(fallback_name),
        _ => fallback_name,
    }
}
fn default_batch_size() -> usize {
    100
}
fn default_batch_age() -> u64 {
    30
}
fn default_batch_bytes() -> usize {
    1024 * 1024
}
fn default_max_retries() -> u32 {
    3
}
fn default_initial_backoff() -> u64 {
    1
}
fn default_max_backoff() -> u64 {
    60
}

// Public exports
#[allow(unused)]
pub use console_log_output::{create_console_output, ConsoleLogOutput};
#[allow(unused)]
pub use file_log_output::{create_file_output, FileLogOutput, LogOutputWriter};
#[allow(unused)]
pub use file_log_source::{FileLogSource, MultilineMatchType};
#[allow(unused)]
pub use http_log_output::{create_http_output, HttpLogOutput};
#[allow(unused)]
pub use log_filters::{create_filter, LogFilter};
#[allow(unused)]
pub use log_parsers::{create_parser, LogEntry, LogParser};
#[allow(unused)]
pub use orchestrator::LogOrchestrator;
#[allow(unused)]
pub use plugin_log_output::{create_plugin_output, PluginLogOutput};
#[allow(unused)]
pub use s3_log_output::{create_s3_output, S3LogOutput};
#[allow(unused)]
pub use shipper::LogEntry as ShipperLogEntry;
#[allow(unused)]
pub use syslog_log_output::{create_syslog_output, SyslogLogOutput};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_file_source() {
        let yaml = r#"
name: "nginx_access"
enabled: true
paths:
  - "/var/log/nginx/access.log"
  - "/var/log/nginx/access.log.1"
parser:
  parser_type: "apachecombined"
filters:
  - type: "exclude"
    pattern: ".*bot.*"
outputs:
  - "elk_cluster"
  - "local_archive"
"#;

        let source: LogSource = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(source.name, "nginx_access");
        assert!(source.enabled);
        assert_eq!(source.paths.len(), 2);
        assert!(matches!(
            source.parser.parser_type,
            ParserType::ApacheCombined
        ));
        assert_eq!(source.filters.len(), 1);
        assert_eq!(source.outputs.len(), 2);
    }

    #[test]
    fn test_deserialize_http_output() {
        let yaml = r#"
type: "http"
name: "elk_cluster"
enabled: true
url: "http://elasticsearch:9200/_bulk"
method: "POST"
headers:
  Content-Type: "application/x-ndjson"
auth:
  type: "basic"
  username: "elastic"
  password: "changeme"
batch:
  max_size: 1000
  max_age: 30
compression:
  enabled: true
  algorithm: "gzip"
  level: 6
"#;

        let output: LogOutput = serde_yaml::from_str(yaml).unwrap();
        match output {
            LogOutput::Http(http) => {
                assert_eq!(http.name, "elk_cluster");
                assert!(http.enabled);
                assert_eq!(http.url, "http://elasticsearch:9200/_bulk");
                assert_eq!(http.method, "POST");
                assert_eq!(
                    http.headers.get("Content-Type").unwrap(),
                    "application/x-ndjson"
                );
                assert!(matches!(
                    http.auth.as_ref().unwrap(),
                    HttpAuth::Basic { .. }
                ));
                assert_eq!(http.batch.max_size, 1000);
                assert!(http.compression.enabled);
                assert!(matches!(
                    http.compression.algorithm,
                    CompressionAlgorithm::Gzip
                ));
            }
            _ => panic!("Expected HTTP output"),
        }
    }

    #[test]
    fn test_deserialize_s3_output() {
        let yaml = r#"
type: "s3"
name: "log_archive"
enabled: true
bucket: "my-logs-bucket"
region: "us-west-2"
prefix: "nginx/"
upload_interval: 3600
compression:
  enabled: true
  algorithm: "gzip"
"#;

        let output: LogOutput = serde_yaml::from_str(yaml).unwrap();
        match output {
            LogOutput::S3(s3) => {
                assert_eq!(s3.name, "log_archive");
                assert!(s3.enabled);
                assert_eq!(s3.bucket, "my-logs-bucket");
                assert_eq!(s3.region, "us-west-2");
                assert_eq!(s3.prefix, "nginx/");
                assert_eq!(s3.upload_interval, 3600);
                assert!(s3.compression.enabled);
            }
            _ => panic!("Expected S3 output"),
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        use crate::logs::LogOrchestrator;

        // Create a minimal logs config
        let config = LogsConfig {
            global: Default::default(),
            sources: vec![],
            outputs: vec![],
            processing: Default::default(),
        };

        // Test that we can create an orchestrator
        let _orchestrator = LogOrchestrator::new(config);
        // Just test that creation succeeds
    }
}
