//! Log parsers
//!
//! This module provides parsers for different log formats including plain text,
//! JSON, key-value pairs, Apache/Nginx logs, syslog, and custom regex patterns.
//!
//! # Examples
//!
//! ## JSON Parser
//!
//! ```rust
//! use crate::logs::log_parsers::{LogParser, JsonParser};
//!
//! let parser = JsonParser::new();
//! let log_line = r#"{"timestamp": "2023-01-01T12:00:00Z", "level": "INFO", "message": "Hello world"}"#;
//! let entry = parser.parse(log_line).unwrap();
//! assert_eq!(entry.fields.get("level"), Some(&serde_json::Value::String("INFO".to_string())));
//! ```
//!
//! ## Key-Value Parser
//!
//! ```rust
//! use crate::logs::log_parsers::{LogParser, KeyValueParser};
//!
//! let parser = KeyValueParser::new();
//! let log_line = "timestamp=2023-01-01T12:00:00Z level=INFO message=\"Hello world\"";
//! let entry = parser.parse(log_line).unwrap();
//! assert_eq!(entry.fields.get("level"), Some(&serde_json::Value::String("INFO".to_string())));
//! ```

use crate::logs::{ParserConfig, ParserType};
use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Parsed log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Raw log line
    pub raw: String,
    /// Parsed timestamp (if available)
    pub timestamp: Option<DateTime<Utc>>,
    /// Parsed fields
    pub fields: HashMap<String, Value>,
    /// Log level (if detected)
    pub level: Option<String>,
    /// Log message (if extracted)
    pub message: Option<String>,
}

impl LogEntry {
    /// Create a new log entry from raw text
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            timestamp: None,
            fields: HashMap::new(),
            level: None,
            message: None,
        }
    }

    /// Create a log entry with parsed fields
    pub fn with_fields(raw: String, fields: HashMap<String, Value>) -> Self {
        Self {
            raw,
            timestamp: None,
            fields,
            level: None,
            message: None,
        }
    }
}

/// Trait for log parsers
pub trait LogParser: Send + Sync {
    /// Parse a log line into a LogEntry
    fn parse(&self, line: &str) -> Result<LogEntry>;

    /// Get the parser type
    fn parser_type(&self) -> ParserType;
}

/// Plain text parser (no parsing, just wraps the line)
pub struct PlainParser;

impl PlainParser {
    /// Create a new plain text parser
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for PlainParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        Ok(LogEntry::new(line.to_string()))
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Plain
    }
}

/// JSON parser
pub struct JsonParser;

impl JsonParser {
    /// Create a new JSON parser
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for JsonParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("Failed to parse JSON: {}", line))?;

        let mut entry = LogEntry::new(line.to_string());

        if let Value::Object(map) = &value {
            // Extract common fields
            if let Some(Value::String(ts)) = map.get("timestamp") {
                if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
                    entry.timestamp = Some(dt.with_timezone(&Utc));
                }
            }

            if let Some(Value::String(lvl)) = map.get("level") {
                entry.level = Some(lvl.clone());
            }

            if let Some(Value::String(msg)) = map.get("message") {
                entry.message = Some(msg.clone());
            }

            // Store all fields
            for (key, val) in map {
                entry.fields.insert(key.clone(), val.clone());
            }
        }

        Ok(entry)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Json
    }
}

/// Key-value parser (key=value format)
pub struct KeyValueParser {
    separator: String,
}

impl KeyValueParser {
    /// Create a new key-value parser
    pub fn new() -> Self {
        Self {
            separator: "=".to_string(),
        }
    }

    /// Create a key-value parser with custom separator
    pub fn with_separator(separator: String) -> Self {
        Self { separator }
    }
}

impl LogParser for KeyValueParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        let mut entry = LogEntry::new(line.to_string());
        let mut fields = HashMap::new();

        // Simple key-value parsing that handles quoted values
        let chars = line.chars().peekable();
        let mut current_key = String::new();
        let mut current_value = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        let mut parsing_key = true;

        for ch in chars {
            match ch {
                '=' if parsing_key && !in_quotes => {
                    parsing_key = false;
                }
                '"' | '\'' if parsing_key => {
                    // Skip quotes in keys (unlikely but possible)
                    current_key.push(ch);
                }
                '"' | '\'' if !parsing_key => {
                    if !in_quotes {
                        in_quotes = true;
                        quote_char = ch;
                    } else if ch == quote_char {
                        in_quotes = false;
                        quote_char = '"';
                    } else {
                        current_value.push(ch);
                    }
                }
                ' ' | '\t' if !in_quotes && !parsing_key && !current_value.is_empty() => {
                    // End of key-value pair
                    if !current_key.is_empty() {
                        let value = current_value.trim().trim_matches('"').trim_matches('\'');

                        // Try to parse as JSON value
                        let json_value = if let Ok(num) = value.parse::<i64>() {
                            Value::Number(num.into())
                        } else if let Ok(num) = value.parse::<f64>() {
                            if let Some(num_val) = serde_json::Number::from_f64(num) {
                                Value::Number(num_val)
                            } else {
                                Value::String(value.to_string())
                            }
                        } else if value.eq_ignore_ascii_case("true") {
                            Value::Bool(true)
                        } else if value.eq_ignore_ascii_case("false") {
                            Value::Bool(false)
                        } else if value.eq_ignore_ascii_case("null") {
                            Value::Null
                        } else {
                            Value::String(value.to_string())
                        };

                        fields.insert(current_key.clone(), json_value);

                        // Extract common fields
                        match current_key.as_str() {
                            "timestamp" | "time" | "@timestamp" => {
                                if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
                                    entry.timestamp = Some(dt.with_timezone(&Utc));
                                }
                            }
                            "level" | "severity" => {
                                entry.level = Some(value.to_string());
                            }
                            "message" | "msg" => {
                                entry.message = Some(value.to_string());
                            }
                            _ => {}
                        }
                    }
                    current_key.clear();
                    current_value.clear();
                    parsing_key = true;
                }
                _ => {
                    if parsing_key {
                        current_key.push(ch);
                    } else {
                        current_value.push(ch);
                    }
                }
            }
        }

        // Handle the last key-value pair
        if !current_key.is_empty() {
            let value = current_value.trim().trim_matches('"').trim_matches('\'');

            // Try to parse as JSON value
            let json_value = if let Ok(num) = value.parse::<i64>() {
                Value::Number(num.into())
            } else if let Ok(num) = value.parse::<f64>() {
                if let Some(num_val) = serde_json::Number::from_f64(num) {
                    Value::Number(num_val)
                } else {
                    Value::String(value.to_string())
                }
            } else if value.eq_ignore_ascii_case("true") {
                Value::Bool(true)
            } else if value.eq_ignore_ascii_case("false") {
                Value::Bool(false)
            } else if value.eq_ignore_ascii_case("null") {
                Value::Null
            } else {
                Value::String(value.to_string())
            };

            fields.insert(current_key.clone(), json_value);

            // Extract common fields
            match current_key.as_str() {
                "timestamp" | "time" | "@timestamp" => {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
                        entry.timestamp = Some(dt.with_timezone(&Utc));
                    }
                }
                "level" | "severity" => {
                    entry.level = Some(value.to_string());
                }
                "message" | "msg" => {
                    entry.message = Some(value.to_string());
                }
                _ => {}
            }
        }

        entry.fields = fields;
        Ok(entry)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::KeyValue
    }
}

/// Apache common log format parser
/// Format: %h %l %u %t \"%r\" %>s %b
pub struct ApacheCommonParser;

impl ApacheCommonParser {
    /// Create a new Apache common log parser
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for ApacheCommonParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Apache common log format regex
        // %h %l %u %t \"%r\" %>s %b
        let pattern = r##"^(?P<host>\S+) (?P<ident>\S+) (?P<user>\S+) \[(?P<timestamp>[^\]]+)\] "(?P<request>[^"]*)" (?P<status>\d+) (?P<bytes>\S+)$"##;

        let re =
            Regex::new(pattern).with_context(|| "Failed to compile Apache common log regex")?;

        let mut entry = LogEntry::new(line.to_string());

        if let Some(caps) = re.captures(line) {
            let mut fields = HashMap::new();

            if let Some(host) = caps.name("host") {
                fields.insert("host".to_string(), Value::String(host.as_str().to_string()));
            }

            if let Some(ident) = caps.name("ident") {
                fields.insert(
                    "ident".to_string(),
                    Value::String(ident.as_str().to_string()),
                );
            }

            if let Some(user) = caps.name("user") {
                fields.insert("user".to_string(), Value::String(user.as_str().to_string()));
            }

            if let Some(ts) = caps.name("timestamp") {
                // Parse Apache timestamp format: 10/Oct/2000:13:55:36 -0700
                if let Ok(dt) = DateTime::parse_from_str(ts.as_str(), "%d/%b/%Y:%H:%M:%S %z") {
                    entry.timestamp = Some(dt.with_timezone(&Utc));
                    fields.insert(
                        "timestamp".to_string(),
                        Value::String(ts.as_str().to_string()),
                    );
                }
            }

            if let Some(request) = caps.name("request") {
                fields.insert(
                    "request".to_string(),
                    Value::String(request.as_str().to_string()),
                );
                entry.message = Some(request.as_str().to_string());
            }

            if let Some(status) = caps.name("status") {
                if let Ok(code) = status.as_str().parse::<i64>() {
                    fields.insert("status".to_string(), Value::Number(code.into()));
                }
            }

            if let Some(bytes) = caps.name("bytes") {
                if bytes.as_str() != "-" {
                    if let Ok(size) = bytes.as_str().parse::<i64>() {
                        fields.insert("bytes".to_string(), Value::Number(size.into()));
                    }
                }
            }

            entry.fields = fields;
        } else {
            // If regex doesn't match, treat as plain text
            entry.message = Some(line.to_string());
        }

        Ok(entry)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::ApacheCommon
    }
}

/// Apache combined log format parser
/// Format: %h %l %u %t \"%r\" %>s %b \"%{Referer}i\" \"%{User-agent}i\"
pub struct ApacheCombinedParser;

impl ApacheCombinedParser {
    /// Create a new Apache combined log parser
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for ApacheCombinedParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Apache combined log format regex
        // %h %l %u %t \"%r\" %>s %b \"%{Referer}i\" \"%{User-agent}i\"
        let pattern = r##"^(?P<host>\S+) (?P<ident>\S+) (?P<user>\S+) \[(?P<timestamp>[^\]]+)\] "(?P<request>[^"]*)" (?P<status>\d+) (?P<bytes>\S+) "(?P<referer>[^"]*)" "(?P<user_agent>[^"]*)"$"##;

        let re =
            Regex::new(pattern).with_context(|| "Failed to compile Apache combined log regex")?;

        let mut entry = LogEntry::new(line.to_string());

        if let Some(caps) = re.captures(line) {
            let mut fields = HashMap::new();

            if let Some(host) = caps.name("host") {
                fields.insert("host".to_string(), Value::String(host.as_str().to_string()));
            }

            if let Some(ident) = caps.name("ident") {
                fields.insert(
                    "ident".to_string(),
                    Value::String(ident.as_str().to_string()),
                );
            }

            if let Some(user) = caps.name("user") {
                fields.insert("user".to_string(), Value::String(user.as_str().to_string()));
            }

            if let Some(ts) = caps.name("timestamp") {
                // Parse Apache timestamp format: 10/Oct/2000:13:55:36 -0700
                if let Ok(dt) = DateTime::parse_from_str(ts.as_str(), "%d/%b/%Y:%H:%M:%S %z") {
                    entry.timestamp = Some(dt.with_timezone(&Utc));
                    fields.insert(
                        "timestamp".to_string(),
                        Value::String(ts.as_str().to_string()),
                    );
                }
            }

            if let Some(request) = caps.name("request") {
                fields.insert(
                    "request".to_string(),
                    Value::String(request.as_str().to_string()),
                );
                entry.message = Some(request.as_str().to_string());
            }

            if let Some(status) = caps.name("status") {
                if let Ok(code) = status.as_str().parse::<i64>() {
                    fields.insert("status".to_string(), Value::Number(code.into()));
                }
            }

            if let Some(bytes) = caps.name("bytes") {
                if bytes.as_str() != "-" {
                    if let Ok(size) = bytes.as_str().parse::<i64>() {
                        fields.insert("bytes".to_string(), Value::Number(size.into()));
                    }
                }
            }

            if let Some(referer) = caps.name("referer") {
                if !referer.as_str().is_empty() && referer.as_str() != "-" {
                    fields.insert(
                        "referer".to_string(),
                        Value::String(referer.as_str().to_string()),
                    );
                }
            }

            if let Some(user_agent) = caps.name("user_agent") {
                if !user_agent.as_str().is_empty() && user_agent.as_str() != "-" {
                    fields.insert(
                        "user_agent".to_string(),
                        Value::String(user_agent.as_str().to_string()),
                    );
                }
            }

            entry.fields = fields;
        } else {
            // If regex doesn't match, treat as plain text
            entry.message = Some(line.to_string());
        }

        Ok(entry)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::ApacheCombined
    }
}

/// Nginx access log parser (similar to Apache combined)
pub struct NginxParser;

impl NginxParser {
    /// Create a new Nginx log parser
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for NginxParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        // Use the same parsing as Apache combined for now
        // In a real implementation, this might have Nginx-specific formats
        ApacheCombinedParser::new().parse(line)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Nginx
    }
}

/// Syslog parser (RFC 3164/5424)
pub struct SyslogParser;

impl SyslogParser {
    /// Create a new syslog parser
    pub fn new() -> Self {
        Self
    }
}

impl LogParser for SyslogParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        let mut entry = LogEntry::new(line.to_string());

        // RFC 3164 format: <priority>timestamp hostname tag[pid]: message
        // RFC 5424 format: <priority>version timestamp hostname app-name procid msgid [structured-data] message

        let rfc3164_pattern = r##"^<(?P<priority>\d+)>(?P<timestamp>\w+ \d+ \d+:\d+:\d+) (?P<hostname>[^ ]+) (?P<tag>[^\[]+)\[(?P<pid>\d+)\]: (?P<message>.+)$"##;
        let rfc5424_pattern = r##"^<(?P<priority>\d+)>(?P<version>\d+) (?P<timestamp>[^ ]+) (?P<hostname>[^ ]+) (?P<app>[^ ]+) (?P<procid>[^ ]+) (?P<msgid>[^ ]+) (?P<structured>(\[.*?\])?) (?P<message>.+)$"##;

        let mut fields = HashMap::new();

        // Try RFC 5424 first, then RFC 3164
        if let Ok(re) = Regex::new(rfc5424_pattern) {
            if let Some(caps) = re.captures(line) {
                if let Some(priority) = caps.name("priority") {
                    if let Ok(pri) = priority.as_str().parse::<i64>() {
                        fields.insert("priority".to_string(), Value::Number(pri.into()));
                        // Extract facility and severity from priority
                        let facility = pri / 8;
                        let severity = pri % 8;
                        fields.insert("facility".to_string(), Value::Number(facility.into()));
                        fields.insert("severity".to_string(), Value::Number(severity.into()));
                    }
                }

                if let Some(version) = caps.name("version") {
                    fields.insert(
                        "version".to_string(),
                        Value::String(version.as_str().to_string()),
                    );
                }

                if let Some(ts) = caps.name("timestamp") {
                    // RFC 5424 timestamp format: 2003-10-11T22:14:15.003Z
                    if let Ok(dt) = DateTime::parse_from_rfc3339(ts.as_str()) {
                        entry.timestamp = Some(dt.with_timezone(&Utc));
                        fields.insert(
                            "timestamp".to_string(),
                            Value::String(ts.as_str().to_string()),
                        );
                    }
                }

                if let Some(hostname) = caps.name("hostname") {
                    fields.insert(
                        "hostname".to_string(),
                        Value::String(hostname.as_str().to_string()),
                    );
                }

                if let Some(app) = caps.name("app") {
                    fields.insert("app".to_string(), Value::String(app.as_str().to_string()));
                }

                if let Some(procid) = caps.name("procid") {
                    fields.insert(
                        "procid".to_string(),
                        Value::String(procid.as_str().to_string()),
                    );
                }

                if let Some(msgid) = caps.name("msgid") {
                    fields.insert(
                        "msgid".to_string(),
                        Value::String(msgid.as_str().to_string()),
                    );
                }

                if let Some(message) = caps.name("message") {
                    entry.message = Some(message.as_str().to_string());
                }

                entry.fields = fields;
                return Ok(entry);
            }
        }

        // Try RFC 3164
        if let Ok(re) = Regex::new(rfc3164_pattern) {
            if let Some(caps) = re.captures(line) {
                if let Some(priority) = caps.name("priority") {
                    if let Ok(pri) = priority.as_str().parse::<i64>() {
                        fields.insert("priority".to_string(), Value::Number(pri.into()));
                        // Extract facility and severity from priority
                        let facility = pri / 8;
                        let severity = pri % 8;
                        fields.insert("facility".to_string(), Value::Number(facility.into()));
                        fields.insert("severity".to_string(), Value::Number(severity.into()));
                    }
                }

                if let Some(ts) = caps.name("timestamp") {
                    // RFC 3164 timestamp format: Oct 11 22:14:15 (no year)
                    // Assume current year for parsing
                    let now = Utc::now();
                    let year = now.year();
                    let ts_with_year = format!("{} {}", year, ts.as_str());
                    if let Ok(naive) =
                        NaiveDateTime::parse_from_str(&ts_with_year, "%Y %b %e %H:%M:%S")
                    {
                        // Assume UTC since no timezone info
                        if let Some(dt) = naive.and_local_timezone(Utc).single() {
                            entry.timestamp = Some(dt);
                            fields.insert(
                                "timestamp".to_string(),
                                Value::String(ts.as_str().to_string()),
                            );
                        }
                    }
                }

                if let Some(hostname) = caps.name("hostname") {
                    fields.insert(
                        "hostname".to_string(),
                        Value::String(hostname.as_str().to_string()),
                    );
                }

                if let Some(tag) = caps.name("tag") {
                    fields.insert("tag".to_string(), Value::String(tag.as_str().to_string()));
                }

                if let Some(pid) = caps.name("pid") {
                    if let Ok(p) = pid.as_str().parse::<i64>() {
                        fields.insert("pid".to_string(), Value::Number(p.into()));
                    }
                }

                if let Some(message) = caps.name("message") {
                    entry.message = Some(message.as_str().to_string());
                }

                entry.fields = fields;
                return Ok(entry);
            }
        }

        // If no pattern matches, treat as plain text
        entry.message = Some(line.to_string());
        Ok(entry)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Syslog
    }
}

/// Custom regex parser
pub struct RegexParser {
    pattern: Regex,
    field_names: Vec<String>,
}

impl RegexParser {
    /// Create a new regex parser
    pub fn new(pattern: &str, field_names: Vec<String>) -> Result<Self> {
        let pattern =
            Regex::new(pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;

        Ok(Self {
            pattern,
            field_names,
        })
    }

    /// Create a regex parser from ParserConfig
    pub fn from_config(config: &ParserConfig) -> Result<Self> {
        let pattern = config
            .pattern
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Regex pattern is required for regex parser"))?;

        // Extract field names from field_map keys
        let field_names = config.field_map.keys().cloned().collect();

        Self::new(pattern, field_names)
    }
}

impl LogParser for RegexParser {
    fn parse(&self, line: &str) -> Result<LogEntry> {
        let mut entry = LogEntry::new(line.to_string());

        if let Some(caps) = self.pattern.captures(line) {
            let mut fields = HashMap::new();

            // Extract named capture groups
            for name in &self.field_names {
                if let Some(mat) = caps.name(name) {
                    let value = mat.as_str();

                    // Try to parse as JSON value
                    let json_value = if let Ok(num) = value.parse::<i64>() {
                        Value::Number(num.into())
                    } else if let Ok(num) = value.parse::<f64>() {
                        if let Some(num_val) = serde_json::Number::from_f64(num) {
                            Value::Number(num_val)
                        } else {
                            Value::String(value.to_string())
                        }
                    } else if value.eq_ignore_ascii_case("true") {
                        Value::Bool(true)
                    } else if value.eq_ignore_ascii_case("false") {
                        Value::Bool(false)
                    } else if value.eq_ignore_ascii_case("null") {
                        Value::Null
                    } else {
                        Value::String(value.to_string())
                    };

                    fields.insert(name.clone(), json_value);

                    // Extract common fields
                    match name.as_str() {
                        "timestamp" | "time" | "@timestamp" => {
                            if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
                                entry.timestamp = Some(dt.with_timezone(&Utc));
                            }
                        }
                        "level" | "severity" => {
                            entry.level = Some(value.to_string());
                        }
                        "message" | "msg" => {
                            entry.message = Some(value.to_string());
                        }
                        _ => {}
                    }
                }
            }

            // Also extract numbered capture groups
            for (i, mat) in caps.iter().enumerate() {
                if let Some(mat) = mat {
                    if i > 0 {
                        // Skip the full match at index 0
                        let value = mat.as_str();
                        fields.insert(format!("field{}", i), Value::String(value.to_string()));
                    }
                }
            }

            entry.fields = fields;
        }

        Ok(entry)
    }

    fn parser_type(&self) -> ParserType {
        ParserType::Regex
    }
}

/// Create a parser instance based on configuration
pub fn create_parser(config: &ParserConfig) -> Result<Box<dyn LogParser>> {
    match &config.parser_type {
        ParserType::Plain => Ok(Box::new(PlainParser::new())),
        ParserType::Json => Ok(Box::new(JsonParser::new())),
        ParserType::KeyValue => Ok(Box::new(KeyValueParser::new())),
        ParserType::ApacheCommon => Ok(Box::new(ApacheCommonParser::new())),
        ParserType::ApacheCombined => Ok(Box::new(ApacheCombinedParser::new())),
        ParserType::Nginx => Ok(Box::new(NginxParser::new())),
        ParserType::Syslog => Ok(Box::new(SyslogParser::new())),
        ParserType::Regex => {
            let parser = RegexParser::from_config(config)?;
            Ok(Box::new(parser))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_parser() {
        let parser = PlainParser::new();
        let line = "This is a plain log message";
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert!(entry.fields.is_empty());
        assert!(entry.timestamp.is_none());
        assert!(entry.level.is_none());
        assert!(entry.message.is_none());
    }

    #[test]
    fn test_json_parser() {
        let parser = JsonParser::new();
        let line = r#"{"timestamp": "2023-01-01T12:00:00Z", "level": "INFO", "message": "Hello world", "user_id": 123}"#;
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert!(entry.timestamp.is_some());
        assert_eq!(entry.level.as_ref().unwrap(), "INFO");
        assert_eq!(entry.message.as_ref().unwrap(), "Hello world");
        assert_eq!(
            entry.fields.get("user_id"),
            Some(&Value::Number(123.into()))
        );
    }

    #[test]
    fn test_key_value_parser() {
        let parser = KeyValueParser::new();
        let line = r#"timestamp=2023-01-01T12:00:00Z level=INFO message="Hello world" user_id=123"#;
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert!(entry.timestamp.is_some());
        assert_eq!(entry.level.as_ref().unwrap(), "INFO");
        assert_eq!(entry.message.as_ref().unwrap(), "Hello world");
        assert_eq!(
            entry.fields.get("user_id"),
            Some(&Value::Number(123.into()))
        );
    }

    #[test]
    fn test_apache_common_parser() {
        let parser = ApacheCommonParser::new();
        let line =
            r#"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326"#;
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert!(entry.timestamp.is_some());
        assert_eq!(
            entry.fields.get("host"),
            Some(&Value::String("127.0.0.1".to_string()))
        );
        assert_eq!(entry.fields.get("status"), Some(&Value::Number(200.into())));
        assert_eq!(entry.fields.get("bytes"), Some(&Value::Number(2326.into())));
        assert_eq!(
            entry.message.as_ref().unwrap(),
            "GET /apache_pb.gif HTTP/1.0"
        );
    }

    #[test]
    fn test_apache_combined_parser() {
        let parser = ApacheCombinedParser::new();
        let line = r#"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] "GET /apache_pb.gif HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08""#;
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert!(entry.timestamp.is_some());
        assert_eq!(
            entry.fields.get("host"),
            Some(&Value::String("127.0.0.1".to_string()))
        );
        assert_eq!(entry.fields.get("status"), Some(&Value::Number(200.into())));
        assert_eq!(
            entry.fields.get("referer"),
            Some(&Value::String(
                "http://www.example.com/start.html".to_string()
            ))
        );
        assert!(entry.fields.get("user_agent").is_some());
    }

    #[test]
    fn test_syslog_parser_rfc3164() {
        let parser = SyslogParser::new();
        let line =
            r#"<34>Oct 11 22:14:15 mymachine su[123]: 'su root' failed for user on /dev/pts/8"#;
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert!(entry.timestamp.is_some());
        assert_eq!(
            entry.fields.get("priority"),
            Some(&Value::Number(34.into()))
        );
        assert_eq!(entry.fields.get("facility"), Some(&Value::Number(4.into())));
        assert_eq!(entry.fields.get("severity"), Some(&Value::Number(2.into())));
        assert_eq!(
            entry.fields.get("hostname"),
            Some(&Value::String("mymachine".to_string()))
        );
        assert_eq!(
            entry.fields.get("tag"),
            Some(&Value::String("su".to_string()))
        );
        assert_eq!(entry.fields.get("pid"), Some(&Value::Number(123.into())));
    }

    #[test]
    fn test_regex_parser() {
        let parser = RegexParser::new(
            r##"^(?P<timestamp>\d{4}-\d{2}-\d{2}) (?P<level>\w+) (?P<message>.+)$"##,
            vec![
                "timestamp".to_string(),
                "level".to_string(),
                "message".to_string(),
            ],
        )
        .unwrap();
        let line = "2023-01-01 INFO Hello world";
        let entry = parser.parse(line).unwrap();

        assert_eq!(entry.raw, line);
        assert_eq!(
            entry.fields.get("timestamp"),
            Some(&Value::String("2023-01-01".to_string()))
        );
        assert_eq!(
            entry.fields.get("level"),
            Some(&Value::String("INFO".to_string()))
        );
        assert_eq!(
            entry.fields.get("message"),
            Some(&Value::String("Hello world".to_string()))
        );
        assert_eq!(entry.level.as_ref().unwrap(), "INFO");
        assert_eq!(entry.message.as_ref().unwrap(), "Hello world");
    }

    #[test]
    fn test_create_parser() {
        let config = ParserConfig {
            parser_type: ParserType::Json,
            ..Default::default()
        };

        let parser = create_parser(&config).unwrap();
        assert_eq!(parser.parser_type(), ParserType::Json);

        let config = ParserConfig {
            parser_type: ParserType::Plain,
            ..Default::default()
        };

        let parser = create_parser(&config).unwrap();
        assert_eq!(parser.parser_type(), ParserType::Plain);
    }
}
