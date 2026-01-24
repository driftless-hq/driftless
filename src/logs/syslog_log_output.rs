//! Syslog log output implementation
//!
//! This module provides syslog-based log output with RFC 3164/5424 compliance,
//! configurable facilities/priorities, and support for both UDP and TCP protocols.
//!
//! # Examples
//!
//! ```rust
//! use crate::logs::{SyslogOutput, syslog_log_output::SyslogLogOutput};
//!
//! let config = SyslogOutput {
//!     name: "syslog-logs".to_string(),
//!     enabled: true,
//!     facility: "local0".to_string(),
//!     severity: "info".to_string(),
//!     tag: "driftless".to_string(),
//!     server: Some("127.0.0.1:514".to_string()),
//!     protocol: crate::logs::SyslogProtocol::Udp,
//! };
//!
//! let mut output = SyslogLogOutput::new(config).await.unwrap();
//! let entry = crate::logs::ShipperLogEntry::new("log message".to_string(), "test".to_string());
//! output.write_entry(&entry).await.unwrap();
//! output.close().await.unwrap();
//! ```

use crate::logs::{ShipperLogEntry, SyslogOutput, SyslogProtocol};
use anyhow::{Context, Result};
use chrono::Utc;
use std::net::UdpSocket;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

/// Syslog-based log output with RFC 3164/5424 compliance
pub struct SyslogLogOutput {
    config: SyslogOutput,
    udp_socket: Option<UdpSocket>,
    tcp_stream: Option<BufWriter<TcpStream>>,
}

impl SyslogLogOutput {
    /// Create a new syslog log output
    pub async fn new(config: SyslogOutput) -> Result<Self> {
        let mut output = Self {
            config,
            udp_socket: None,
            tcp_stream: None,
        };

        if let Some(server) = &output.config.server {
            match output.config.protocol {
                SyslogProtocol::Udp => {
                    let socket =
                        UdpSocket::bind("0.0.0.0:0").context("Failed to bind UDP socket")?;
                    output.udp_socket = Some(socket);
                }
                SyslogProtocol::Tcp => {
                    let stream = TcpStream::connect(server)
                        .await
                        .context(format!("Failed to connect to syslog server: {}", server))?;
                    output.tcp_stream = Some(BufWriter::new(stream));
                }
            }
        }

        Ok(output)
    }

    /// Format a log entry as a syslog message (RFC 3164)
    fn format_rfc3164(&self, entry: &ShipperLogEntry) -> String {
        let timestamp = entry
            .timestamp
            .unwrap_or(Utc::now())
            .format("%b %e %H:%M:%S");

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let facility_code = self.facility_to_code(&self.config.facility);
        let severity_code = self.severity_to_code(&self.config.severity);
        let priority = (facility_code << 3) | severity_code;

        format!(
            "<{}>{} {} {}[{}]: {}",
            priority,
            timestamp,
            hostname,
            self.config.tag,
            std::process::id(),
            entry.message
        )
    }

    /// Format a log entry as a syslog message (RFC 5424)
    fn format_rfc5424(&self, entry: &ShipperLogEntry) -> String {
        let timestamp = entry.timestamp.unwrap_or(Utc::now()).to_rfc3339();

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "-".to_string());

        let facility_code = self.facility_to_code(&self.config.facility);
        let severity_code = self.severity_to_code(&self.config.severity);
        let priority = (facility_code << 3) | severity_code;

        let app_name = &self.config.tag;
        let procid = std::process::id().to_string();
        let version = 1;
        let msgid = "-";
        let structured_data = "-";

        // RFC 5424 format: <priority>version timestamp hostname app-name procid msgid [structured-data] msg
        format!(
            "<{}>{} {} {} {} {} {} {} {}",
            priority,
            version,
            timestamp,
            hostname,
            app_name,
            procid,
            msgid,
            structured_data,
            entry.message
        )
    }

    /// Convert facility name to numeric code
    fn facility_to_code(&self, facility: &str) -> u8 {
        match facility.to_lowercase().as_str() {
            "kern" => 0,
            "user" => 1,
            "mail" => 2,
            "daemon" => 3,
            "auth" => 4,
            "syslog" => 5,
            "lpr" => 6,
            "news" => 7,
            "uucp" => 8,
            "cron" => 9,
            "authpriv" => 10,
            "ftp" => 11,
            "ntp" => 12,
            "security" => 13,
            "console" => 14,
            "solaris-cron" => 15,
            "local0" => 16,
            "local1" => 17,
            "local2" => 18,
            "local3" => 19,
            "local4" => 20,
            "local5" => 21,
            "local6" => 22,
            "local7" => 23,
            _ => 16, // default to local0
        }
    }

    /// Convert severity name to numeric code
    fn severity_to_code(&self, severity: &str) -> u8 {
        match severity.to_lowercase().as_str() {
            "emerg" | "emergency" => 0,
            "alert" => 1,
            "crit" | "critical" => 2,
            "err" | "error" => 3,
            "warn" | "warning" => 4,
            "notice" => 5,
            "info" | "informational" => 6,
            "debug" => 7,
            _ => 6, // default to info
        }
    }

    /// Send a message via UDP
    async fn send_udp(&mut self, message: &str) -> Result<()> {
        if let (Some(socket), Some(server)) = (&self.udp_socket, &self.config.server) {
            socket
                .send_to(message.as_bytes(), server)
                .context("Failed to send UDP syslog message")?;
        }
        Ok(())
    }

    /// Send a message via TCP
    async fn send_tcp(&mut self, message: &str) -> Result<()> {
        if let Some(stream) = &mut self.tcp_stream {
            stream
                .write_all(message.as_bytes())
                .await
                .context("Failed to write TCP syslog message")?;
            stream
                .write_all(b"\n")
                .await
                .context("Failed to write newline after TCP syslog message")?;
            stream
                .flush()
                .await
                .context("Failed to flush TCP syslog message")?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl super::LogOutputWriter for SyslogLogOutput {
    /// Write a log entry to syslog
    async fn write_entry(&mut self, entry: &ShipperLogEntry) -> Result<()> {
        // Use RFC 5424 format by default (more modern)
        let message = self.format_rfc5424(entry);

        match self.config.protocol {
            SyslogProtocol::Udp => {
                self.send_udp(&message).await?;
            }
            SyslogProtocol::Tcp => {
                self.send_tcp(&message).await?;
            }
        }

        Ok(())
    }

    /// Flush any buffered data (TCP only)
    async fn flush(&mut self) -> Result<()> {
        if let Some(stream) = &mut self.tcp_stream {
            stream
                .flush()
                .await
                .context("Failed to flush TCP syslog stream")?;
        }
        Ok(())
    }

    /// Close the syslog output
    async fn close(mut self) -> Result<()> {
        self.flush().await?;
        // TCP stream will be closed when dropped
        Ok(())
    }
}

/// Create a syslog output instance
pub fn create_syslog_output(config: SyslogOutput) -> Result<Box<dyn super::LogOutputWriter>> {
    Ok(Box::new(tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async { SyslogLogOutput::new(config).await })
    })?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::ShipperLogEntry;

    #[test]
    fn test_format_rfc3164() {
        let config = SyslogOutput {
            name: "test".to_string(),
            enabled: true,
            facility: "local0".to_string(),
            severity: "info".to_string(),
            tag: "testapp".to_string(),
            server: None,
            protocol: SyslogProtocol::Udp,
        };

        let output = SyslogLogOutput {
            config,
            udp_socket: None,
            tcp_stream: None,
        };

        let entry = ShipperLogEntry::new("test message".to_string(), "test".to_string());
        let formatted = output.format_rfc3164(&entry);

        // Should start with priority <134> (local0.info = 16*8 + 6 = 134)
        assert!(formatted.starts_with("<134>"));
        assert!(formatted.contains("testapp"));
        assert!(formatted.contains("test message"));
    }

    #[test]
    fn test_format_rfc5424() {
        let config = SyslogOutput {
            name: "test".to_string(),
            enabled: true,
            facility: "local0".to_string(),
            severity: "info".to_string(),
            tag: "testapp".to_string(),
            server: None,
            protocol: SyslogProtocol::Udp,
        };

        let output = SyslogLogOutput {
            config,
            udp_socket: None,
            tcp_stream: None,
        };

        let entry = ShipperLogEntry::new("test message".to_string(), "test".to_string());
        let formatted = output.format_rfc5424(&entry);

        // Should start with priority <134> and version 1
        assert!(formatted.starts_with("<134>1 "));
        assert!(formatted.contains("testapp"));
        assert!(formatted.contains("test message"));
    }

    #[test]
    fn test_facility_to_code() {
        let config = SyslogOutput {
            name: "test".to_string(),
            enabled: true,
            facility: "local0".to_string(),
            severity: "info".to_string(),
            tag: "testapp".to_string(),
            server: None,
            protocol: SyslogProtocol::Udp,
        };
        let output = SyslogLogOutput {
            config,
            udp_socket: None,
            tcp_stream: None,
        };

        assert_eq!(output.facility_to_code("kern"), 0);
        assert_eq!(output.facility_to_code("user"), 1);
        assert_eq!(output.facility_to_code("local0"), 16);
        assert_eq!(output.facility_to_code("local7"), 23);
        assert_eq!(output.facility_to_code("unknown"), 16); // default
    }

    #[test]
    fn test_severity_to_code() {
        let config = SyslogOutput {
            name: "test".to_string(),
            enabled: true,
            facility: "local0".to_string(),
            severity: "info".to_string(),
            tag: "testapp".to_string(),
            server: None,
            protocol: SyslogProtocol::Udp,
        };
        let output = SyslogLogOutput {
            config,
            udp_socket: None,
            tcp_stream: None,
        };

        assert_eq!(output.severity_to_code("emerg"), 0);
        assert_eq!(output.severity_to_code("alert"), 1);
        assert_eq!(output.severity_to_code("crit"), 2);
        assert_eq!(output.severity_to_code("err"), 3);
        assert_eq!(output.severity_to_code("warning"), 4);
        assert_eq!(output.severity_to_code("notice"), 5);
        assert_eq!(output.severity_to_code("info"), 6);
        assert_eq!(output.severity_to_code("debug"), 7);
        assert_eq!(output.severity_to_code("unknown"), 6); // default
    }
}
