//! HTTP log output implementation
//!
//! This module provides HTTP-based log output with batching, authentication,
//! retry logic, and compression support.

#[allow(unused_imports)]
use crate::logs::{BatchConfig, CompressionAlgorithm, HttpAuth, HttpOutput, ShipperLogEntry};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use flate2::write::GzEncoder;
use reqwest::{Client, Method, RequestBuilder};
use serde_json::json;
use std::io::Write;
use std::time::Duration;
use tokio::time;

/// HTTP-based log output with batching, authentication, and retry logic
pub struct HttpLogOutput {
    config: HttpOutput,
    client: Client,
    buffer: Vec<String>,
    buffer_size_bytes: usize,
    last_batch_time: DateTime<Utc>,
}

impl HttpLogOutput {
    /// Create a new HTTP log output
    #[allow(dead_code)]
    pub async fn new(config: HttpOutput) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            config,
            client,
            buffer: Vec::new(),
            buffer_size_bytes: 0,
            last_batch_time: Utc::now(),
        })
    }

    /// Format a log entry as JSON
    fn format_entry(entry: &ShipperLogEntry) -> String {
        let mut map = serde_json::json!({
            "message": entry.message,
            "source": entry.source,
            "timestamp": entry.timestamp.map(|t| t.to_rfc3339()).unwrap_or_else(|| Utc::now().to_rfc3339())
        });

        if let serde_json::Value::Object(ref mut obj) = map {
            if !entry.fields.is_empty() {
                obj.insert("fields".to_string(), json!(entry.fields));
            }
            if !entry.labels.is_empty() {
                obj.insert("labels".to_string(), json!(entry.labels));
            }
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

    /// Check if batch should be sent based on size, age, or byte limits
    fn should_send_batch(&self) -> bool {
        let now = Utc::now();
        let age_seconds = (now - self.last_batch_time).num_seconds() as u64;

        self.buffer.len() >= self.config.batch.max_size
            || age_seconds >= self.config.batch.max_age
            || self.buffer_size_bytes >= self.config.batch.max_bytes
    }

    /// Build HTTP request with authentication and headers
    fn build_request(&self, data: Vec<u8>) -> Result<RequestBuilder> {
        let method = Method::from_bytes(self.config.method.as_bytes()).unwrap_or(Method::POST);

        let mut request = self.client.request(method, &self.config.url);

        // Add custom headers
        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        // Add authentication
        if let Some(ref auth) = self.config.auth {
            match auth {
                HttpAuth::Basic { username, password } => {
                    use base64::prelude::*;
                    let credentials = BASE64_STANDARD.encode(format!("{}:{}", username, password));
                    request = request.header("Authorization", format!("Basic {}", credentials));
                }
                HttpAuth::Bearer { token } => {
                    request = request.header("Authorization", format!("Bearer {}", token));
                }
                HttpAuth::ApiKey { header_name, key } => {
                    request = request.header(header_name, key);
                }
            }
        }

        // Add compression header if compressing
        if self.config.compression.algorithm != CompressionAlgorithm::None {
            request = request.header("Content-Encoding", "gzip");
        }

        // Set content type
        request = request.header("Content-Type", "application/json");

        Ok(request.body(data))
    }

    /// Send batch with retry logic
    async fn send_batch_with_retry(&self, data: Vec<u8>) -> Result<()> {
        let mut attempt = 0;
        let mut backoff = self.config.retry.initial_backoff;

        loop {
            match self.send_request(data.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempt += 1;
                    if attempt >= self.config.retry.max_attempts {
                        return Err(anyhow!(
                            "Failed to send batch after {} attempts: {}",
                            attempt,
                            e
                        ));
                    }

                    if backoff > self.config.retry.max_backoff {
                        backoff = self.config.retry.max_backoff;
                    }

                    time::sleep(Duration::from_secs(backoff)).await;
                    backoff *= 2; // Exponential backoff
                }
            }
        }
    }

    /// Send HTTP request
    async fn send_request(&self, data: Vec<u8>) -> Result<()> {
        let request = self.build_request(data)?;
        let response = request
            .send()
            .await
            .context("Failed to send HTTP request")?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "HTTP request failed with status {}: {}",
                status,
                body
            ));
        }

        Ok(())
    }

    /// Send current buffer as batch
    async fn send_batch(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let data = self.buffer.join("\n") + "\n";
        let compressed_data = self.compress_data(data.as_bytes())?;

        self.send_batch_with_retry(compressed_data).await?;

        self.buffer.clear();
        self.buffer_size_bytes = 0;
        self.last_batch_time = Utc::now();

        Ok(())
    }
}

#[async_trait::async_trait]
impl super::LogOutputWriter for HttpLogOutput {
    async fn write_entry(&mut self, entry: &ShipperLogEntry) -> Result<()> {
        let formatted = Self::format_entry(entry);
        let entry_size = formatted.len();

        self.buffer.push(formatted);
        self.buffer_size_bytes += entry_size;

        if self.should_send_batch() {
            self.send_batch().await?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        if !self.buffer.is_empty() {
            self.send_batch().await?;
        }
        Ok(())
    }

    async fn close(mut self) -> Result<()> {
        self.flush().await
    }
}

/// Create a new HTTP log output
#[allow(dead_code)]
pub async fn create_http_output(config: HttpOutput) -> Result<Box<dyn super::LogOutputWriter>> {
    let output = HttpLogOutput::new(config).await?;
    Ok(Box::new(output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_format_entry() {
        let entry = ShipperLogEntry::new("test message".to_string(), "test_source".to_string());
        let formatted = HttpLogOutput::format_entry(&entry);

        assert!(formatted.contains("\"message\":\"test message\""));
        assert!(formatted.contains("\"source\":\"test_source\""));
        assert!(formatted.contains("\"timestamp\""));
    }

    #[tokio::test]
    async fn test_should_send_batch() {
        let config = HttpOutput {
            name: "test".to_string(),
            enabled: true,
            url: "http://example.com".to_string(),
            method: "POST".to_string(),
            headers: HashMap::new(),
            auth: None,
            batch: BatchConfig {
                max_size: 2,
                max_age: 60,
                max_bytes: 100,
            },
            compression: Default::default(),
            retry: Default::default(),
        };

        let output = HttpLogOutput {
            config,
            client: reqwest::Client::new(),
            buffer: vec!["test".to_string()],
            buffer_size_bytes: 50,
            last_batch_time: Utc::now() - chrono::Duration::seconds(30),
        };

        // Should not send yet (size = 1, age = 30s, bytes = 50)
        assert!(!output.should_send_batch());
    }

    #[tokio::test]
    async fn test_compression() {
        let config = HttpOutput {
            name: "test".to_string(),
            enabled: true,
            url: "http://example.com".to_string(),
            method: "POST".to_string(),
            headers: HashMap::new(),
            auth: None,
            batch: Default::default(),
            compression: crate::logs::CompressionConfig {
                enabled: true,
                algorithm: CompressionAlgorithm::Gzip,
                level: 6,
            },
            retry: Default::default(),
        };

        let output = HttpLogOutput {
            config,
            client: reqwest::Client::new(),
            buffer: Vec::new(),
            buffer_size_bytes: 0,
            last_batch_time: Utc::now(),
        };

        let data = b"test data for compression that should be long enough to actually compress smaller than the original";
        let compressed = output.compress_data(data).unwrap();
        assert!(compressed.len() <= data.len()); // Compressed should be same or smaller
    }
}
