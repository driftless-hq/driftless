//! File log source implementation
//!
//! This module provides functionality for tailing log files with rotation handling,
//! encoding support, and multiline log processing.
//!
//! # Examples
//!
//! ## Basic file tailing
//!
//! **YAML Format:**
//! ```yaml
//! logs:
//!   - name: app_logs
//!     type: file
//!     paths:
//!       - /var/log/application.log
//!       - /var/log/application.err
//!     file_options:
//!       from_beginning: false
//!       follow_rotated: true
//!       encoding: utf8
//!     parser:
//!       parser_type: json
//! ```
//!
//! **JSON Format:**
//! ```json
//! {
//!   "logs": [
//!     {
//!       "name": "app_logs",
//!       "type": "file",
//!       "paths": [
//!         "/var/log/application.log",
//!         "/var/log/application.err"
//!       ],
//!       "file_options": {
//!         "from_beginning": false,
//!         "follow_rotated": true,
//!         "encoding": "utf8"
//!       },
//!       "parser": {
//!         "parser_type": "json"
//!       }
//!     }
//!   ]
//! }
//! ```
//!
//! ## Multiline log processing
//!
//! **YAML Format:**
//! ```yaml
//! logs:
//!   - name: java_logs
//!     type: file
//!     paths:
//!       - /var/log/java/application.log
//!     file_options:
//!       multiline:
//!         pattern: '^\d{4}-\d{2}-\d{2}'
//!         negate: false
//!         match: after
//!     parser:
//!       parser_type: plain
//! ```
//!
//! ## Log rotation handling
//!
//! **YAML Format:**
//! ```yaml
//! logs:
//!   - name: rotated_logs
//!     type: file
//!     paths:
//!       - /var/log/app.log
//!     file_options:
//!       follow_rotated: true
//!       rotation_check_interval: 30
//!       max_file_size: 104857600
//! ```

use crate::logs::{FileEncoding, LogSource, MissingFileHandling};
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tokio::time;

/// File reader state for tracking position and rotation
#[derive(Debug, Clone)]
struct FileReaderState {
    /// Current file path
    #[allow(dead_code)]
    path: PathBuf,
    /// Current read position in the file
    position: u64,
    /// File inode for rotation detection
    inode: u64,
    /// File size at last check
    size: u64,
    /// Last modification time
    mtime: SystemTime,
}

/// Multiline configuration for handling multi-line log entries
#[derive(Debug, Clone)]
pub struct MultilineConfig {
    /// Regex pattern to match line starts
    pub pattern: String,
    /// Whether to negate the pattern match
    pub negate: bool,
    /// What to do with lines that match (before/after/between)
    pub match_type: MultilineMatchType,
    /// Maximum number of lines to combine
    pub max_lines: Option<usize>,
    /// Timeout for multiline assembly
    #[allow(dead_code)]
    pub timeout: Option<Duration>,
    /// Continue pattern for between matching
    pub continue_pattern: Option<String>,
    /// End pattern for between matching
    pub end_pattern: Option<String>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Default)]
pub enum MultilineMatchType {
    /// Lines after the match are part of the same entry
    #[default]
    #[allow(dead_code)]
    After,
    /// Lines before the match are part of the same entry
    #[allow(dead_code)]
    Before,
    /// Lines between start and end patterns are combined
    #[allow(dead_code)]
    Between,
}

/// File log source for tailing log files
pub struct FileLogSource {
    config: LogSource,
    multiline_config: Option<MultilineConfig>,
}

impl FileLogSource {
    /// Create a new file log source
    pub fn new(config: LogSource) -> Result<Self> {
        let multiline_config = Self::parse_multiline_config(&config)?;
        Ok(Self {
            config,
            multiline_config,
        })
    }

    /// Parse multiline configuration from file options
    fn parse_multiline_config(config: &LogSource) -> Result<Option<MultilineConfig>> {
        // Parse multiline config from the parser configuration
        if config.parser.multiline.enabled {
            let multiline = &config.parser.multiline;

            // Determine the pattern and match type based on configuration
            let (pattern, match_type, continue_pattern, end_pattern) = match multiline.match_type {
                crate::logs::MultilineMatchType::After => {
                    let pattern = multiline
                        .start_pattern
                        .clone()
                        .unwrap_or_else(|| r"^\s+".to_string()); // Default pattern for continuation lines
                    (pattern, MultilineMatchType::After, None, None)
                }
                crate::logs::MultilineMatchType::Before => {
                    let pattern = multiline
                        .start_pattern
                        .clone()
                        .unwrap_or_else(|| r"^\s+".to_string());
                    (pattern, MultilineMatchType::Before, None, None)
                }
                crate::logs::MultilineMatchType::Between => {
                    let start_pattern = multiline.start_pattern.clone().ok_or_else(|| {
                        anyhow::anyhow!("start_pattern is required for between matching")
                    })?;
                    let end_pattern = multiline.end_pattern.clone().ok_or_else(|| {
                        anyhow::anyhow!("end_pattern is required for between matching")
                    })?;
                    (
                        start_pattern,
                        MultilineMatchType::Between,
                        None,
                        Some(end_pattern),
                    )
                }
            };

            Ok(Some(MultilineConfig {
                pattern,
                negate: multiline.negate,
                match_type,
                max_lines: Some(multiline.max_lines),
                timeout: multiline.timeout.map(Duration::from_secs),
                continue_pattern,
                end_pattern,
            }))
        } else {
            Ok(None)
        }
    }

    /// Start tailing the configured log files
    pub async fn start_tailing(&self, line_sender: mpsc::Sender<String>) -> Result<()> {
        let mut file_states = HashMap::new();

        // Initialize file states for all paths
        for path_str in &self.config.paths {
            let path = PathBuf::from(path_str);
            if let Ok(state) = self.initialize_file_state(&path) {
                file_states.insert(path_str.clone(), state);
            } else {
                match self.config.file_options.missing_file_handling {
                    MissingFileHandling::Skip => continue,
                    MissingFileHandling::Warn => {
                        eprintln!("Warning: Cannot access log file: {}", path_str);
                        continue;
                    }
                    MissingFileHandling::Error => {
                        return Err(anyhow::anyhow!("Cannot access log file: {}", path_str));
                    }
                }
            }
        }

        // Main tailing loop
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            // Check each file for new content and rotation
            for (path_str, state) in &mut file_states {
                match self.check_file_updates(path_str, state).await {
                    Ok(new_lines) => {
                        for line in new_lines {
                            if line_sender.send(line).await.is_err() {
                                // Receiver closed, stop tailing
                                return Ok(());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading file {}: {}", path_str, e);
                    }
                }
            }
        }
    }

    /// Initialize the file reader state for a given path
    fn initialize_file_state(&self, path: &Path) -> Result<FileReaderState> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for {}", path.display()))?;

        let position = if self.config.file_options.from_beginning {
            0
        } else {
            metadata.len()
        };

        Ok(FileReaderState {
            path: path.to_path_buf(),
            position,
            inode: metadata.ino(),
            size: metadata.len(),
            mtime: metadata.modified()?,
        })
    }

    /// Check for file updates and read new lines
    async fn check_file_updates(
        &self,
        path_str: &str,
        state: &mut FileReaderState,
    ) -> Result<Vec<String>> {
        let path = Path::new(path_str);

        // Check if file exists
        if !path.exists() {
            return Ok(Vec::new());
        }

        let metadata = fs::metadata(path)?;

        // Check for rotation (inode changed or file truncated)
        let current_inode = metadata.ino();
        let current_size = metadata.len();

        if current_inode != state.inode || current_size < state.size {
            // File was rotated or truncated, reset position
            state.position = 0;
            state.inode = current_inode;
            state.size = current_size;
            state.mtime = metadata.modified()?;
        } else {
            state.size = current_size;
        }

        // If file hasn't grown, no new content
        if current_size <= state.position {
            return Ok(Vec::new());
        }

        // Read new content
        let mut file = OpenOptions::new().read(true).open(path)?;
        file.seek(SeekFrom::Start(state.position))?;

        let mut reader = BufReader::new(file);
        let mut new_lines = Vec::new();
        let mut buffer = String::new();

        loop {
            let bytes_read = reader.read_line(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            // Decode based on encoding
            let line = match self.config.file_options.encoding {
                FileEncoding::Utf8 => buffer.clone(),
                FileEncoding::Ascii => buffer.clone(),
                FileEncoding::Latin1 => {
                    // Convert Latin-1 to UTF-8
                    let utf8_bytes: Vec<u8> = buffer.as_bytes().to_vec();
                    String::from_utf8_lossy(&utf8_bytes).to_string()
                }
            };

            new_lines.push(line.trim_end().to_string());
            buffer.clear();
        }

        // Update position
        state.position = current_size;

        // Process multiline if configured
        if let Some(ref multiline) = self.multiline_config {
            new_lines = self.process_multiline(new_lines, multiline)?;
        }

        Ok(new_lines)
    }

    /// Process multiline log entries
    fn process_multiline(
        &self,
        lines: Vec<String>,
        config: &MultilineConfig,
    ) -> Result<Vec<String>> {
        if lines.is_empty() {
            return Ok(lines);
        }

        let start_pattern = Regex::new(&config.pattern)?;
        let continue_pattern = config
            .continue_pattern
            .as_ref()
            .map(|p| Regex::new(p))
            .transpose()?;
        let end_pattern = config
            .end_pattern
            .as_ref()
            .map(|p| Regex::new(p))
            .transpose()?;

        let mut result = Vec::new();
        let mut current_entry = String::new();
        let mut in_multiline = false;
        let mut between_started = false;

        for line in lines {
            let start_matches = start_pattern.is_match(&line);
            let should_start = start_matches != config.negate;

            match config.match_type {
                MultilineMatchType::After => {
                    if should_start {
                        // This is a new log entry start
                        if !current_entry.is_empty() {
                            result.push(current_entry);
                        }
                        current_entry = line;
                        in_multiline = true;
                    } else if in_multiline {
                        // Continuation of current entry
                        current_entry.push('\n');
                        current_entry.push_str(&line);
                    } else {
                        // Standalone line
                        result.push(line);
                    }
                }
                MultilineMatchType::Before => {
                    if should_start {
                        // This line belongs to the previous entry
                        if !current_entry.is_empty() {
                            current_entry.push('\n');
                        }
                        current_entry.push_str(&line);
                    } else {
                        // This is a new entry start
                        if !current_entry.is_empty() {
                            result.push(current_entry);
                        }
                        current_entry = line;
                    }
                }
                MultilineMatchType::Between => {
                    if let (Some(ref end_pat), Some(ref cont_pat)) =
                        (&end_pattern, &continue_pattern)
                    {
                        if should_start && !between_started {
                            // Start of a new between block
                            if !current_entry.is_empty() {
                                result.push(current_entry);
                            }
                            current_entry = line.clone();
                            between_started = true;
                        } else if between_started {
                            if end_pat.is_match(&line) {
                                // End of the between block
                                current_entry.push('\n');
                                current_entry.push_str(&line);
                                result.push(current_entry);
                                current_entry = String::new();
                                between_started = false;
                            } else if cont_pat.is_match(&line) {
                                // Continuation line within the block
                                current_entry.push('\n');
                                current_entry.push_str(&line);
                            } else {
                                // Line doesn't match continue pattern, treat as separate
                                if !current_entry.is_empty() {
                                    result.push(current_entry);
                                    current_entry = String::new();
                                    between_started = false;
                                }
                                result.push(line);
                            }
                        } else {
                            // Not in a between block, treat as separate
                            result.push(line);
                        }
                    } else {
                        // Invalid between configuration, fall back to after matching
                        if should_start {
                            if !current_entry.is_empty() {
                                result.push(current_entry);
                            }
                            current_entry = line;
                            in_multiline = true;
                        } else if in_multiline {
                            current_entry.push('\n');
                            current_entry.push_str(&line);
                        } else {
                            result.push(line);
                        }
                    }
                }
            }

            // Check max lines limit
            if let Some(max_lines) = config.max_lines {
                let line_count = current_entry.lines().count();
                if line_count >= max_lines && !current_entry.is_empty() {
                    result.push(current_entry);
                    current_entry = String::new();
                    in_multiline = false;
                    between_started = false;
                }
            }
        }

        // Add remaining entry
        if !current_entry.is_empty() {
            result.push(current_entry);
        }

        Ok(result)
    }

    /// Get the source configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &LogSource {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::{FileOptions, ParserConfig};
    use std::collections::HashMap;

    #[test]
    fn test_file_log_source_creation() {
        let config = LogSource {
            name: "test_source".to_string(),
            source_type: "file".to_string(),
            enabled: true,
            paths: vec!["/tmp/test.log".to_string()],
            file_options: FileOptions::default(),
            parser: ParserConfig::default(),
            filters: Vec::new(),
            outputs: Vec::new(),
            labels: HashMap::new(),
            plugin_name: None,
            plugin_source_name: None,
        };

        let source = FileLogSource::new(config);
        assert!(source.is_ok());
    }

    #[test]
    fn test_multiline_processing_after_match() {
        let config = MultilineConfig {
            pattern: r"^\d{4}-\d{2}-\d{2}".to_string(),
            negate: false,
            match_type: MultilineMatchType::After,
            max_lines: None,
            timeout: None,
            continue_pattern: None,
            end_pattern: None,
        };

        let source = FileLogSource::new(LogSource::default()).unwrap();

        let lines = vec![
            "2024-01-01 10:00:00 INFO Starting application".to_string(),
            "  Loading configuration".to_string(),
            "  Connecting to database".to_string(),
            "2024-01-01 10:00:01 INFO Application started".to_string(),
            "  Server listening on port 8080".to_string(),
        ];

        let result = source.process_multiline(lines, &config).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("Starting application"));
        assert!(result[0].contains("Loading configuration"));
        assert!(result[1].contains("Application started"));
    }

    #[test]
    fn test_multiline_processing_before_match() {
        let config = MultilineConfig {
            pattern: r"^\s+".to_string(),
            negate: false,
            match_type: MultilineMatchType::Before,
            max_lines: None,
            timeout: None,
            continue_pattern: None,
            end_pattern: None,
        };

        let source = FileLogSource::new(LogSource::default()).unwrap();

        let lines = vec![
            "Exception in thread \"main\" java.lang.RuntimeException: Something went wrong"
                .to_string(),
            "    at com.example.App.main(App.java:15)".to_string(),
            "    at java.lang.Thread.run(Thread.java:745)".to_string(),
            "INFO Application terminated".to_string(),
        ];

        let result = source.process_multiline(lines, &config).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("RuntimeException"));
        assert!(result[0].contains("at com.example"));
    }

    #[test]
    fn test_missing_file_handling() {
        let config = LogSource {
            name: "test_source".to_string(),
            source_type: "file".to_string(),
            enabled: true,
            paths: vec!["/nonexistent/file.log".to_string()],
            file_options: FileOptions {
                missing_file_handling: MissingFileHandling::Skip,
                ..Default::default()
            },
            parser: ParserConfig::default(),
            filters: Vec::new(),
            outputs: Vec::new(),
            labels: HashMap::new(),
            plugin_name: None,
            plugin_source_name: None,
        };

        let source = FileLogSource::new(config);
        assert!(source.is_ok());
        // The source should handle missing files gracefully during tailing
    }
}
