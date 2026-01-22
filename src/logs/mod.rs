use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main logs configuration schema
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
}


/// Multiline log configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MultilineConfig {
    /// Whether multiline parsing is enabled
    #[serde(default)]
    pub enabled: bool,
    /// Pattern to match the start of a multiline log entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_pattern: Option<String>,
    /// Maximum number of lines per multiline entry
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
}

/// Filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FilterConfig {
    /// Include only lines matching regex
    Include { pattern: String, case_sensitive: Option<bool> },
    /// Exclude lines matching regex
    Exclude { pattern: String, case_sensitive: Option<bool> },
    /// Include lines containing any of the specified strings
    Contains { values: Vec<String>, case_sensitive: Option<bool> },
    /// Exclude lines containing any of the specified strings
    NotContains { values: Vec<String>, case_sensitive: Option<bool> },
    /// Include lines where field matches value
    FieldMatch { field: String, value: String, case_sensitive: Option<bool> },
    /// Drop lines above a certain rate (rate limiting)
    RateLimit { events_per_second: u32 },
}

/// Log output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// File output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SyslogProtocol {
    /// UDP protocol
    #[default]
    Udp,
    /// TCP protocol
    Tcp,
}


/// Console output configuration (for debugging)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleTarget {
    /// Standard output
    #[default]
    Stdout,
    /// Standard error
    Stderr,
}


/// File rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    SetFieldIf { field: String, value: String, condition: ConditionConfig },
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
fn default_true() -> bool { true }
fn default_buffer_size() -> usize { 8192 }
fn default_flush_interval() -> u64 { 30 }
fn default_max_lines() -> usize { 100 }
fn default_filename_pattern() -> String { "%Y-%m-%d-%H-%M-%S.log".to_string() }
fn default_s3_prefix() -> String { "logs/".to_string() }
fn default_upload_interval() -> u64 { 300 }
fn default_http_method() -> String { "POST".to_string() }
fn default_syslog_facility() -> String { "local0".to_string() }
fn default_syslog_severity() -> String { "info".to_string() }
fn default_syslog_tag() -> String { "driftless".to_string() }
fn default_max_files() -> usize { 10 }
pub fn default_compression_level() -> u32 { 6 }
fn default_batch_size() -> usize { 100 }
fn default_batch_age() -> u64 { 30 }
fn default_batch_bytes() -> usize { 1024 * 1024 }
fn default_max_retries() -> u32 { 3 }
fn default_initial_backoff() -> u64 { 1 }
fn default_max_backoff() -> u64 { 60 }

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
        assert!(matches!(source.parser.parser_type, ParserType::ApacheCombined));
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
                assert_eq!(http.headers.get("Content-Type").unwrap(), "application/x-ndjson");
                assert!(matches!(http.auth.as_ref().unwrap(), HttpAuth::Basic { .. }));
                assert_eq!(http.batch.max_size, 1000);
                assert!(http.compression.enabled);
                assert!(matches!(http.compression.algorithm, CompressionAlgorithm::Gzip));
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
}