//! S3 log output implementation
//!
//! This module provides S3-based log output with batched uploads, compression,
//! and configurable prefixes and regions.
//!
//! # Examples
//!
//! ```rust
//! use crate::logs::{S3Output, s3_log_output::S3LogOutput};
//!
//! let config = S3Output {
//!     name: "s3-logs".to_string(),
//!     enabled: true,
//!     bucket: "my-logs-bucket".to_string(),
//!     region: "us-east-1".to_string(),
//!     prefix: "logs/".to_string(),
//!     upload_interval: 300,
//!     ..Default::default()
//! };
//!
//! let mut output = S3LogOutput::new(config).await.unwrap();
//! let entry = crate::logs::ShipperLogEntry::new("log message".to_string(), "test".to_string());
//! output.write_entry(&entry).await.unwrap();
//! output.close().await.unwrap();
//! ```

use crate::logs::{CompressionAlgorithm, CompressionConfig, S3Output, ShipperLogEntry};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use std::io::Write;
use std::str::FromStr;

/// S3-based log output with batched uploads and compression
pub struct S3LogOutput {
    config: S3Output,
    bucket: Bucket,
    buffer: Vec<String>,
    last_upload: DateTime<Utc>,
    upload_task: Option<tokio::task::JoinHandle<()>>,
}

impl S3LogOutput {
    /// Create a new S3 log output
    pub async fn new(config: S3Output) -> Result<Self> {
        let region = Region::from_str(&config.region)
            .map_err(|e| anyhow!("Invalid S3 region '{}': {}", config.region, e))?;

        let credentials = if let (Some(access_key), Some(secret_key)) =
            (&config.access_key, &config.secret_key)
        {
            Credentials::new(Some(access_key), Some(secret_key), None, None, None)
                .map_err(|e| anyhow!("Invalid S3 credentials: {}", e))?
        } else {
            Credentials::default()
                .map_err(|e| anyhow!("Failed to load default S3 credentials: {}", e))?
        };

        let bucket = Bucket::new(&config.bucket, region, credentials)
            .map_err(|e| anyhow!("Failed to create S3 bucket client: {}", e))?;

        Ok(Self {
            config,
            bucket,
            buffer: Vec::new(),
            last_upload: Utc::now(),
            upload_task: None,
        })
    }

    /// Format a log entry as JSON
    fn format_entry(entry: &ShipperLogEntry) -> String {
        use serde_json::json;

        let mut map = serde_json::Map::new();
        map.insert("message".to_string(), json!(entry.message));
        map.insert("source".to_string(), json!(entry.source));
        map.insert(
            "timestamp".to_string(),
            json!(entry
                .timestamp
                .map(|t| t.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339())),
        );

        if !entry.fields.is_empty() {
            map.insert("fields".to_string(), json!(entry.fields));
        }

        if !entry.labels.is_empty() {
            map.insert("labels".to_string(), json!(entry.labels));
        }

        serde_json::to_string(&map).unwrap_or_else(|_| {
            format!(
                "{{\"message\":{},\"source\":\"{}\"}}",
                json!(entry.message),
                entry.source
            )
        })
    }

    /// Compress data if configured
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.compression.algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Gzip | CompressionAlgorithm::Zlib => {
                let mut encoder = GzEncoder::new(
                    Vec::new(),
                    flate2::Compression::new(self.config.compression.level),
                );
                encoder.write_all(data)?;
                encoder.finish().context("Failed to compress data")
            }
        }
    }

    /// Generate S3 key for upload
    fn generate_key(&self, timestamp: &DateTime<Utc>) -> String {
        let timestamp_str = timestamp.format("%Y/%m/%d/%H/%M/%S").to_string();
        let random_suffix = format!("{:x}", rand::random::<u32>());
        let extension = match self.config.compression.algorithm {
            CompressionAlgorithm::None => "jsonl",
            CompressionAlgorithm::Gzip => "jsonl.gz",
            CompressionAlgorithm::Zlib => "jsonl.gz",
        };

        format!(
            "{}{}-{}.{}",
            self.config.prefix, timestamp_str, random_suffix, extension
        )
    }

    /// Generate S3 key for upload (static version for testing)
    fn generate_key_static(config: &S3Output, timestamp: &DateTime<Utc>) -> String {
        let timestamp_str = timestamp.format("%Y/%m/%d/%H/%M/%S").to_string();
        let random_suffix = format!("{:x}", rand::random::<u32>());
        let extension = match config.compression.algorithm {
            CompressionAlgorithm::None => "jsonl",
            CompressionAlgorithm::Gzip => "jsonl.gz",
            CompressionAlgorithm::Zlib => "jsonl.gz",
        };

        format!(
            "{}{}-{}.{}",
            config.prefix, timestamp_str, random_suffix, extension
        )
    }

    /// Upload buffered data to S3
    async fn upload_batch(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let data = self.buffer.join("\n") + "\n";
        let compressed_data = self.compress_data(data.as_bytes())?;
        let key = self.generate_key(&Utc::now());

        self.bucket
            .put_object_with_content_type(&key, &compressed_data, "application/json")
            .await
            .map_err(|e| anyhow!("Failed to upload to S3: {}", e))?;

        self.buffer.clear();
        self.last_upload = Utc::now();

        Ok(())
    }

    /// Check if upload is needed based on time or buffer size
    fn should_upload(&self) -> bool {
        let time_since_last = Utc::now().signed_duration_since(self.last_upload);
        time_since_last.num_seconds() >= self.config.upload_interval as i64
            || self.buffer.len() >= 1000 // TODO: Make batch size configurable
    }
}

#[async_trait::async_trait]
impl super::LogOutputWriter for S3LogOutput {
    async fn write_entry(&mut self, entry: &ShipperLogEntry) -> Result<()> {
        let formatted = Self::format_entry(entry);
        self.buffer.push(formatted);

        if self.should_upload() {
            self.upload_batch().await?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.upload_batch().await?;
        }
        Ok(())
    }

    async fn close(mut self) -> Result<()> {
        if let Some(task) = self.upload_task.take() {
            task.abort();
        }
        self.flush().await
    }
}

/// Create a new S3 log output
pub async fn create_s3_output(config: S3Output) -> Result<Box<dyn super::LogOutputWriter>> {
    let output = S3LogOutput::new(config).await?;
    Ok(Box::new(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::file_log_output::LogOutputWriter;
    use std::env;
    use tokio::test;

    // Note: These tests require valid AWS credentials and will be skipped in CI
    // unless AWS credentials are provided

    #[test]
    async fn test_s3_output_basic() {
        // Skip if no AWS credentials
        if env::var("AWS_ACCESS_KEY_ID").is_err() && env::var("AWS_PROFILE").is_err() {
            println!("Skipping S3 test - no AWS credentials available");
            return;
        }

        let config = S3Output {
            name: "test-s3".to_string(),
            enabled: true,
            bucket: env::var("TEST_S3_BUCKET").unwrap_or_else(|_| "test-bucket".to_string()),
            region: "us-east-1".to_string(),
            prefix: "test-logs/".to_string(),
            upload_interval: 1, // Upload immediately for testing
            compression: CompressionConfig::default(),
            access_key: None,
            secret_key: None,
        };

        let mut output = S3LogOutput::new(config).await.unwrap();
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());

        output.write_entry(&entry).await.unwrap();
        output.close().await.unwrap();
    }

    #[test]
    async fn test_s3_output_compression() {
        if env::var("AWS_ACCESS_KEY_ID").is_err() && env::var("AWS_PROFILE").is_err() {
            println!("Skipping S3 compression test - no AWS credentials available");
            return;
        }

        let config = S3Output {
            name: "test-s3-compressed".to_string(),
            enabled: true,
            bucket: env::var("TEST_S3_BUCKET").unwrap_or_else(|_| "test-bucket".to_string()),
            region: "us-east-1".to_string(),
            prefix: "test-logs-compressed/".to_string(),
            upload_interval: 1,
            compression: CompressionConfig {
                enabled: true,
                algorithm: CompressionAlgorithm::Gzip,
                level: 6,
            },
            access_key: None,
            secret_key: None,
        };

        let mut output = S3LogOutput::new(config).await.unwrap();
        let entry = ShipperLogEntry::new(
            "test compressed message".to_string(),
            "test_source".to_string(),
        );

        output.write_entry(&entry).await.unwrap();
        output.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_format_entry() {
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());
        let formatted = S3LogOutput::format_entry(&entry);

        assert!(formatted.contains("\"message\":\"test message\""));
        assert!(formatted.contains("\"source\":\"test_source\""));
        assert!(formatted.contains("\"timestamp\""));
    }

    #[tokio::test]
    async fn test_generate_key() {
        let config = S3Output {
            name: "test".to_string(),
            enabled: true,
            bucket: "test-bucket".to_string(),
            region: "us-east-1".to_string(),
            prefix: "logs/".to_string(),
            upload_interval: 300,
            compression: CompressionConfig::default(),
            access_key: None,
            secret_key: None,
        };

        let timestamp = Utc::now();
        let key = S3LogOutput::generate_key_static(&config, &timestamp);

        assert!(key.starts_with("logs/"));
        assert!(key.contains(".jsonl"));
        assert!(key.contains(&timestamp.format("%Y").to_string()));
    }
}
