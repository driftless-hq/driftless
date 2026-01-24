//! Log filters
//!
//! This module provides filtering capabilities for log entries including include/exclude patterns,
//! field matching, rate limiting, and content-based filtering.

use crate::logs::{FilterConfig, LogEntry};
use anyhow::{Context, Result};
use regex::Regex;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Trait for log filters
pub trait LogFilter: Send + Sync {
    /// Apply the filter to a log entry
    /// Returns true if the entry should be kept, false if it should be dropped
    fn filter(&self, entry: &LogEntry) -> Result<bool>;
}

/// Include filter - only keep entries matching the regex pattern
pub struct IncludeFilter {
    regex: Regex,
}

impl IncludeFilter {
    /// Create a new include filter
    pub fn new(pattern: &str, case_sensitive: bool) -> Result<Self> {
        let regex = if case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        }
        .with_context(|| format!("Invalid regex pattern: {}", pattern))?;

        Ok(Self { regex })
    }

    /// Create from FilterConfig
    pub fn from_config(config: &FilterConfig) -> Result<Self> {
        match config {
            FilterConfig::Include {
                pattern,
                case_sensitive,
            } => Self::new(pattern, case_sensitive.unwrap_or(true)),
            _ => Err(anyhow::anyhow!("Invalid config type for IncludeFilter")),
        }
    }
}

impl LogFilter for IncludeFilter {
    fn filter(&self, entry: &LogEntry) -> Result<bool> {
        Ok(self.regex.is_match(&entry.raw))
    }
}

/// Exclude filter - drop entries matching the regex pattern
pub struct ExcludeFilter {
    regex: Regex,
}

impl ExcludeFilter {
    /// Create a new exclude filter
    pub fn new(pattern: &str, case_sensitive: bool) -> Result<Self> {
        let regex = if case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){}", pattern))
        }
        .with_context(|| format!("Invalid regex pattern: {}", pattern))?;

        Ok(Self { regex })
    }

    /// Create from FilterConfig
    pub fn from_config(config: &FilterConfig) -> Result<Self> {
        match config {
            FilterConfig::Exclude {
                pattern,
                case_sensitive,
            } => Self::new(pattern, case_sensitive.unwrap_or(true)),
            _ => Err(anyhow::anyhow!("Invalid config type for ExcludeFilter")),
        }
    }
}

impl LogFilter for ExcludeFilter {
    fn filter(&self, entry: &LogEntry) -> Result<bool> {
        Ok(!self.regex.is_match(&entry.raw))
    }
}

/// Contains filter - keep entries containing any of the specified strings
pub struct ContainsFilter {
    values: Vec<String>,
    case_sensitive: bool,
}

impl ContainsFilter {
    /// Create a new contains filter
    pub fn new(values: Vec<String>, case_sensitive: bool) -> Self {
        Self {
            values,
            case_sensitive,
        }
    }

    /// Create from FilterConfig
    pub fn from_config(config: &FilterConfig) -> Result<Self> {
        match config {
            FilterConfig::Contains {
                values,
                case_sensitive,
            } => Ok(Self::new(values.clone(), case_sensitive.unwrap_or(true))),
            _ => Err(anyhow::anyhow!("Invalid config type for ContainsFilter")),
        }
    }
}

impl LogFilter for ContainsFilter {
    fn filter(&self, entry: &LogEntry) -> Result<bool> {
        let haystack = if self.case_sensitive {
            entry.raw.clone()
        } else {
            entry.raw.to_lowercase()
        };

        for value in &self.values {
            let needle = if self.case_sensitive {
                value.clone()
            } else {
                value.to_lowercase()
            };

            if haystack.contains(&needle) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// NotContains filter - drop entries containing any of the specified strings
pub struct NotContainsFilter {
    values: Vec<String>,
    case_sensitive: bool,
}

impl NotContainsFilter {
    /// Create a new not contains filter
    pub fn new(values: Vec<String>, case_sensitive: bool) -> Self {
        Self {
            values,
            case_sensitive,
        }
    }

    /// Create from FilterConfig
    pub fn from_config(config: &FilterConfig) -> Result<Self> {
        match config {
            FilterConfig::NotContains {
                values,
                case_sensitive,
            } => Ok(Self::new(values.clone(), case_sensitive.unwrap_or(true))),
            _ => Err(anyhow::anyhow!("Invalid config type for NotContainsFilter")),
        }
    }
}

impl LogFilter for NotContainsFilter {
    fn filter(&self, entry: &LogEntry) -> Result<bool> {
        let haystack = if self.case_sensitive {
            entry.raw.clone()
        } else {
            entry.raw.to_lowercase()
        };

        for value in &self.values {
            let needle = if self.case_sensitive {
                value.clone()
            } else {
                value.to_lowercase()
            };

            if haystack.contains(&needle) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// FieldMatch filter - keep entries where a field matches a value
pub struct FieldMatchFilter {
    field: String,
    value: String,
    case_sensitive: bool,
}

impl FieldMatchFilter {
    /// Create a new field match filter
    pub fn new(field: String, value: String, case_sensitive: bool) -> Self {
        Self {
            field,
            value,
            case_sensitive,
        }
    }

    /// Create from FilterConfig
    pub fn from_config(config: &FilterConfig) -> Result<Self> {
        match config {
            FilterConfig::FieldMatch {
                field,
                value,
                case_sensitive,
            } => Ok(Self::new(
                field.clone(),
                value.clone(),
                case_sensitive.unwrap_or(true),
            )),
            _ => Err(anyhow::anyhow!("Invalid config type for FieldMatchFilter")),
        }
    }
}

impl LogFilter for FieldMatchFilter {
    fn filter(&self, entry: &LogEntry) -> Result<bool> {
        // Check parsed fields first
        if let Some(field_value) = entry.fields.get(&self.field) {
            let field_str = match field_value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => field_value.to_string(),
            };

            let matches = if self.case_sensitive {
                field_str == self.value
            } else {
                field_str.to_lowercase() == self.value.to_lowercase()
            };

            if matches {
                return Ok(true);
            }
        }

        // Also check special fields
        let special_value = match self.field.as_str() {
            "level" => entry.level.as_deref(),
            "message" => entry.message.as_deref(),
            "timestamp" => entry.timestamp.as_ref().map(|_| "present"), // Just check if present
            _ => None,
        };

        if let Some(val) = special_value {
            let matches = if self.case_sensitive {
                val == self.value
            } else {
                val.to_lowercase() == self.value.to_lowercase()
            };

            if matches {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// RateLimit filter - drop entries above a certain rate
pub struct RateLimitFilter {
    events_per_second: u32,
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    events: Vec<Instant>,
}

impl RateLimitFilter {
    /// Create a new rate limit filter
    pub fn new(events_per_second: u32) -> Self {
        Self {
            events_per_second,
            state: Arc::new(Mutex::new(RateLimitState { events: Vec::new() })),
        }
    }

    /// Create from FilterConfig
    pub fn from_config(config: &FilterConfig) -> Result<Self> {
        match config {
            FilterConfig::RateLimit { events_per_second } => Ok(Self::new(*events_per_second)),
            _ => Err(anyhow::anyhow!("Invalid config type for RateLimitFilter")),
        }
    }
}

impl LogFilter for RateLimitFilter {
    fn filter(&self, _entry: &LogEntry) -> Result<bool> {
        let mut state = self.state.lock().unwrap();
        let now = Instant::now();

        // Remove events older than 1 second
        let one_second_ago = now - Duration::from_secs(1);
        state.events.retain(|&time| time > one_second_ago);

        // Check if we're under the limit
        if state.events.len() < self.events_per_second as usize {
            state.events.push(now);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Create a filter instance based on configuration
pub fn create_filter(config: &FilterConfig) -> Result<Box<dyn LogFilter>> {
    match config {
        FilterConfig::Include { .. } => {
            let filter = IncludeFilter::from_config(config)?;
            Ok(Box::new(filter))
        }
        FilterConfig::Exclude { .. } => {
            let filter = ExcludeFilter::from_config(config)?;
            Ok(Box::new(filter))
        }
        FilterConfig::Contains { .. } => {
            let filter = ContainsFilter::from_config(config)?;
            Ok(Box::new(filter))
        }
        FilterConfig::NotContains { .. } => {
            let filter = NotContainsFilter::from_config(config)?;
            Ok(Box::new(filter))
        }
        FilterConfig::FieldMatch { .. } => {
            let filter = FieldMatchFilter::from_config(config)?;
            Ok(Box::new(filter))
        }
        FilterConfig::RateLimit { .. } => {
            let filter = RateLimitFilter::from_config(config)?;
            Ok(Box::new(filter))
        }
    }
}

/// Apply a list of filters to a log entry
/// Returns true if the entry passes all filters (should be kept)
#[allow(dead_code)]
pub fn apply_filters(entry: &LogEntry, filters: &[Box<dyn LogFilter>]) -> Result<bool> {
    for filter in filters {
        if !filter.filter(entry)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::log_parsers::LogEntry;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_include_filter() {
        let filter = IncludeFilter::new("ERROR", true).unwrap();
        let entry = LogEntry::new("This is an ERROR message".to_string());

        assert!(filter.filter(&entry).unwrap());

        let entry2 = LogEntry::new("This is an info message".to_string());
        assert!(!filter.filter(&entry2).unwrap());
    }

    #[test]
    fn test_include_filter_case_insensitive() {
        let filter = IncludeFilter::new("error", false).unwrap();
        let entry = LogEntry::new("This is an ERROR message".to_string());

        assert!(filter.filter(&entry).unwrap());
    }

    #[test]
    fn test_exclude_filter() {
        let filter = ExcludeFilter::new("ERROR", true).unwrap();
        let entry = LogEntry::new("This is an ERROR message".to_string());

        assert!(!filter.filter(&entry).unwrap());

        let entry2 = LogEntry::new("This is an info message".to_string());
        assert!(filter.filter(&entry2).unwrap());
    }

    #[test]
    fn test_contains_filter() {
        let filter = ContainsFilter::new(vec!["ERROR".to_string(), "WARN".to_string()], true);
        let entry = LogEntry::new("This is an ERROR message".to_string());

        assert!(filter.filter(&entry).unwrap());

        let entry2 = LogEntry::new("This is an info message".to_string());
        assert!(!filter.filter(&entry2).unwrap());
    }

    #[test]
    fn test_not_contains_filter() {
        let filter = NotContainsFilter::new(vec!["ERROR".to_string()], true);
        let entry = LogEntry::new("This is an ERROR message".to_string());

        assert!(!filter.filter(&entry).unwrap());

        let entry2 = LogEntry::new("This is an info message".to_string());
        assert!(filter.filter(&entry2).unwrap());
    }

    #[test]
    fn test_field_match_filter() {
        let filter = FieldMatchFilter::new("level".to_string(), "INFO".to_string(), true);

        let mut entry = LogEntry::new("log message".to_string());
        entry.level = Some("INFO".to_string());

        assert!(filter.filter(&entry).unwrap());

        entry.level = Some("ERROR".to_string());
        assert!(!filter.filter(&entry).unwrap());
    }

    #[test]
    fn test_field_match_parsed_field() {
        let filter = FieldMatchFilter::new("user_id".to_string(), "123".to_string(), true);

        let mut fields = HashMap::new();
        fields.insert("user_id".to_string(), Value::Number(123.into()));
        let entry = LogEntry::with_fields("log message".to_string(), fields);

        assert!(filter.filter(&entry).unwrap());
    }

    #[test]
    fn test_rate_limit_filter() {
        let filter = RateLimitFilter::new(2); // 2 events per second

        let entry = LogEntry::new("log message".to_string());

        // First two should pass
        assert!(filter.filter(&entry).unwrap());
        assert!(filter.filter(&entry).unwrap());

        // Third should be rate limited
        assert!(!filter.filter(&entry).unwrap());
    }

    #[test]
    fn test_create_filter() {
        let config = FilterConfig::Include {
            pattern: "ERROR".to_string(),
            case_sensitive: Some(true),
        };

        let filter = create_filter(&config).unwrap();
        let entry = LogEntry::new("ERROR message".to_string());
        assert!(filter.filter(&entry).unwrap());
    }
}
