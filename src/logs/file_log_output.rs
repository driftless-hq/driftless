//! File log output
//!
//! This module provides file-based log output functionality with support for
//! file rotation, compression, and timestamp-based filename patterns.

use crate::logs::{CompressionAlgorithm, FileOutput, RotationStrategy, ShipperLogEntry};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Trait for log output writers
#[async_trait::async_trait]
pub trait LogOutputWriter: Send {
    /// Write a log entry
    async fn write_entry(&mut self, entry: &ShipperLogEntry) -> Result<()>;

    /// Flush any buffered data
    #[allow(dead_code)]
    async fn flush(&mut self) -> Result<()>;

    /// Close the output
    #[allow(dead_code)]
    async fn close(self) -> Result<()>;
}

/// File-based log output with rotation and compression
pub struct FileLogOutput {
    config: FileOutput,
    current_file: Option<FileWriter>,
    rotation_state: RotationState,
}

struct FileWriter {
    file: Option<File>,
    path: PathBuf,
    compressor: Option<Box<dyn Write + Send>>,
    bytes_written: u64,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
struct RotationState {
    current_size: u64,
    last_rotation: DateTime<Utc>,
    file_count: usize,
}

impl FileLogOutput {
    /// Create a new file log output
    pub fn new(config: FileOutput) -> Result<Self> {
        let rotation_state = RotationState {
            current_size: 0,
            last_rotation: Utc::now(),
            file_count: Self::count_existing_files(&config.path)?,
        };

        Ok(Self {
            config,
            current_file: None,
            rotation_state,
        })
    }

    /// Count existing rotated files
    fn count_existing_files(base_path: &str) -> Result<usize> {
        let path = Path::new(base_path);
        if let Some(parent) = path.parent() {
            if parent.exists() {
                let base_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("log");

                let count = fs::read_dir(parent)?
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        entry
                            .file_name()
                            .to_str()
                            .map(|name| name.starts_with(base_name))
                            .unwrap_or(false)
                    })
                    .count();
                Ok(count)
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    /// Get the current filename based on pattern and timestamp
    fn get_filename(&self, timestamp: &DateTime<Utc>) -> String {
        let pattern = &self.config.filename_pattern;

        // Simple pattern replacement - in a real implementation, this would use
        // a proper templating engine or strftime
        let mut filename = pattern
            .replace("%Y", &timestamp.format("%Y").to_string())
            .replace("%m", &timestamp.format("%m").to_string())
            .replace("%d", &timestamp.format("%d").to_string())
            .replace("%H", &timestamp.format("%H").to_string())
            .replace("%M", &timestamp.format("%M").to_string())
            .replace("%S", &timestamp.format("%S").to_string());

        // If we have rotated files, append the rotation number
        if self.rotation_state.file_count > 0 {
            let stem = Path::new(&filename)
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            let extension = Path::new(&filename)
                .extension()
                .unwrap_or_default()
                .to_string_lossy();
            if extension.is_empty() {
                filename = format!("{}.{}", stem, self.rotation_state.file_count);
            } else {
                filename = format!("{}.{}.{}", stem, self.rotation_state.file_count, extension);
            }
        }

        filename
    }

    /// Check if rotation is needed
    fn needs_rotation(&self, entry_size: u64) -> bool {
        match self.config.rotation.strategy {
            RotationStrategy::None => false,
            RotationStrategy::Size => {
                if let Some(max_size) = self.config.rotation.max_size {
                    self.rotation_state.current_size + entry_size > max_size
                } else {
                    false
                }
            }
            RotationStrategy::Time => {
                if let Some(max_age) = self.config.rotation.max_age {
                    let age = Utc::now().signed_duration_since(self.rotation_state.last_rotation);
                    age.num_seconds() as u64 >= max_age
                } else {
                    false
                }
            }
            RotationStrategy::SizeOrTime => {
                let size_rotate = if let Some(max_size) = self.config.rotation.max_size {
                    self.rotation_state.current_size + entry_size > max_size
                } else {
                    false
                };

                let time_rotate = if let Some(max_age) = self.config.rotation.max_age {
                    let age = Utc::now().signed_duration_since(self.rotation_state.last_rotation);
                    age.num_seconds() as u64 >= max_age
                } else {
                    false
                };

                size_rotate || time_rotate
            }
        }
    }

    /// Perform file rotation
    fn rotate_file(&mut self) -> Result<()> {
        if let Some(writer) = self.current_file.take() {
            writer.close()?;
        }

        // Clean up old files if we exceed max_files
        self.cleanup_old_files()?;

        self.rotation_state.last_rotation = Utc::now();
        self.rotation_state.current_size = 0;
        self.rotation_state.file_count += 1;

        Ok(())
    }

    /// Clean up old rotated files
    fn cleanup_old_files(&self) -> Result<()> {
        if self.config.rotation.max_files == 0 {
            return Ok(());
        }

        let path = Path::new(&self.config.path);
        if let Some(parent) = path.parent() {
            let base_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("log");

            let mut files: Vec<_> = fs::read_dir(parent)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_str()
                        .map(|name| name.starts_with(base_name))
                        .unwrap_or(false)
                })
                .collect();

            // Sort by modification time (oldest first)
            files.sort_by_key(|entry| {
                entry
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });

            // Remove oldest files if we exceed the limit
            let to_remove = files.len().saturating_sub(self.config.rotation.max_files);
            for file in files.into_iter().take(to_remove) {
                let _ = fs::remove_file(file.path());
            }
        }

        Ok(())
    }

    /// Ensure we have a current file to write to
    fn ensure_file(&mut self, timestamp: &DateTime<Utc>) -> Result<()> {
        if self.current_file.is_none() || self.needs_rotation(0) {
            if self.current_file.is_some() {
                self.rotate_file()?;
            }

            let filename = self.get_filename(timestamp);
            let filepath = Path::new(&self.config.path).with_file_name(filename);

            // Create directory if it doesn't exist
            if let Some(parent) = filepath.parent() {
                fs::create_dir_all(parent)?;
            }

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&filepath)?;

            let (file_for_writer, compressor): (Option<File>, Option<Box<dyn Write + Send>>) =
                if self.config.compression.enabled {
                    match self.config.compression.algorithm {
                        CompressionAlgorithm::Gzip | CompressionAlgorithm::Zlib => {
                            let encoder = GzEncoder::new(
                                file,
                                flate2::Compression::new(self.config.compression.level),
                            );
                            (None, Some(Box::new(encoder)))
                        }
                        CompressionAlgorithm::None => (Some(file), None),
                    }
                } else {
                    (Some(file), None)
                };

            self.current_file = Some(FileWriter {
                file: file_for_writer,
                path: filepath,
                compressor,
                bytes_written: 0,
                created_at: *timestamp,
            });
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl LogOutputWriter for FileLogOutput {
    async fn write_entry(&mut self, entry: &ShipperLogEntry) -> Result<()> {
        let timestamp = entry.timestamp.unwrap_or(Utc::now());

        // Format the log entry first to know its size
        let formatted = format_entry(entry);
        let bytes = formatted.as_bytes();

        // Check if we need to rotate before writing this entry
        if self.needs_rotation(bytes.len() as u64) {
            self.rotate_file()?;
            self.current_file = None; // Force file recreation
        }

        self.ensure_file(&timestamp)?;

        if let Some(writer) = &mut self.current_file {
            // Write to the appropriate destination
            let write_result = if let Some(compressor) = &mut writer.compressor {
                compressor.write_all(bytes)
            } else if let Some(file) = &mut writer.file {
                file.write_all(bytes)
            } else {
                return Err(anyhow!("No file or compressor available for writing"));
            };

            write_result
                .with_context(|| format!("Failed to write to file: {}", writer.path.display()))?;

            writer.bytes_written += bytes.len() as u64;
            self.rotation_state.current_size += bytes.len() as u64;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if let Some(writer) = &mut self.current_file {
            if let Some(compressor) = &mut writer.compressor {
                compressor.flush()?;
            } else if let Some(file) = &mut writer.file {
                file.flush()?;
            }
        }
        Ok(())
    }

    async fn close(self) -> Result<()> {
        if let Some(writer) = self.current_file {
            if let Some(mut compressor) = writer.compressor {
                compressor.flush()?;
            }
            if let Some(file) = writer.file {
                file.sync_all()?;
            }
        }
        Ok(())
    }
}

impl FileWriter {
    fn close(self) -> Result<()> {
        if let Some(mut compressor) = self.compressor {
            compressor.flush()?;
            // For compressed files, the compressor owns the file handle
        } else if let Some(file) = self.file {
            file.sync_all()?;
        }
        Ok(())
    }
}

/// Format a log entry for output
fn format_entry(entry: &ShipperLogEntry) -> String {
    // Simple JSON format for now - could be configurable
    let mut map = serde_json::Map::new();

    map.insert(
        "timestamp".to_string(),
        serde_json::Value::String(
            entry
                .timestamp
                .map(|t| t.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
        ),
    );
    map.insert(
        "message".to_string(),
        serde_json::Value::String(entry.message.clone()),
    );
    map.insert(
        "source".to_string(),
        serde_json::Value::String(entry.source.clone()),
    );

    if !entry.fields.is_empty() {
        map.insert(
            "fields".to_string(),
            serde_json::Value::Object(entry.fields.clone().into_iter().collect()),
        );
    }

    if !entry.labels.is_empty() {
        map.insert(
            "labels".to_string(),
            serde_json::Value::Object(
                entry
                    .labels
                    .clone()
                    .into_iter()
                    .map(|(k, v)| (k, serde_json::Value::String(v)))
                    .collect(),
            ),
        );
    }

    format!(
        "{}\n",
        serde_json::to_string(&map).unwrap_or_else(|_| entry.message.clone())
    )
}

/// Create a file output writer from configuration
pub fn create_file_output(config: FileOutput) -> Result<Box<dyn LogOutputWriter>> {
    let output = FileLogOutput::new(config)?;
    Ok(Box::new(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::{CompressionConfig, RotationConfig, ShipperLogEntry};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_output_basic() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let config = FileOutput {
            name: "test".to_string(),
            enabled: true,
            path: log_path.to_string_lossy().to_string(),
            filename_pattern: "test.log".to_string(),
            rotation: RotationConfig::default(),
            compression: CompressionConfig::default(),
        };

        let mut output = FileLogOutput::new(config).unwrap();
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());

        output.write_entry(&entry).await.unwrap();
        output.flush().await.unwrap();
        output.close().await.unwrap();

        // Check file was created and contains the message
        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("test message"));
        assert!(content.contains("test_source"));
    }

    #[tokio::test]
    async fn test_file_output_with_compression() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log.gz");

        let config = FileOutput {
            name: "test".to_string(),
            enabled: true,
            path: log_path.to_string_lossy().to_string(),
            filename_pattern: "test.log.gz".to_string(),
            rotation: RotationConfig::default(),
            compression: CompressionConfig {
                enabled: true,
                algorithm: CompressionAlgorithm::Gzip,
                level: 6,
            },
        };

        let mut output = FileLogOutput::new(config).unwrap();
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());

        output.write_entry(&entry).await.unwrap();
        output.close().await.unwrap();

        // Check compressed file was created
        assert!(log_path.exists());
        let metadata = fs::metadata(&log_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_filename_pattern() {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path().join("app.log");

        let config = FileOutput {
            name: "test".to_string(),
            enabled: true,
            path: base_path.to_string_lossy().to_string(),
            filename_pattern: "%Y-%m-%d-app.log".to_string(),
            rotation: RotationConfig::default(),
            compression: CompressionConfig::default(),
        };

        let output = FileLogOutput::new(config).unwrap();
        let timestamp = Utc::now();
        let filename = output.get_filename(&timestamp);

        // Should contain date components
        assert!(filename.contains(&timestamp.format("%Y").to_string()));
        assert!(filename.contains(&timestamp.format("%m").to_string()));
        assert!(filename.contains(&timestamp.format("%d").to_string()));
        assert!(filename.contains("app.log"));
    }

    #[tokio::test]
    async fn test_rotation_by_size() {
        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");

        let config = FileOutput {
            name: "test".to_string(),
            enabled: true,
            path: log_path.to_string_lossy().to_string(),
            filename_pattern: "test.log".to_string(),
            rotation: RotationConfig {
                strategy: RotationStrategy::Size,
                max_size: Some(100), // Very small for testing
                max_age: None,
                max_files: 5,
            },
            compression: CompressionConfig::default(),
        };

        let mut output = FileLogOutput::new(config).unwrap();

        // Write entries until rotation occurs
        for i in 0..10 {
            let entry = ShipperLogEntry::new(format!("message {}", i), "test".to_string());
            output.write_entry(&entry).await.unwrap();
        }

        output.close().await.unwrap();

        // Should have created multiple files due to rotation
        let files: Vec<_> = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert!(files.len() > 1, "Should have rotated files");
    }
}
