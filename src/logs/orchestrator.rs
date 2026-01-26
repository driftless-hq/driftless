//! Log Processing Pipeline Orchestrator
//!
//! This module orchestrates the complete log processing pipeline, coordinating
//! log sources, parsers, filters, and outputs with proper buffering and error handling.

use crate::logs::{
    create_console_output, create_file_output, create_filter, create_http_output, create_parser,
    create_plugin_output, create_s3_output, create_syslog_output, FileLogSource, FilterConfig,
    LogEntry, LogOutput, LogOutputWriter, LogSource, LogsConfig, ParserConfig, ShipperLogEntry,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

/// Raw log entry with source information
#[derive(Debug, Clone)]
pub struct RawLogEntry {
    /// Raw log line
    pub line: String,
    /// Source name that generated this entry
    pub source: String,
    /// Labels associated with this source
    pub labels: HashMap<String, String>,
}

/// Log Processing Pipeline Orchestrator
///
/// Coordinates the complete log processing pipeline with async task management,
/// buffering, and error handling.
pub struct LogOrchestrator {
    config: LogsConfig,
    running: bool,
    source_tasks: Vec<JoinHandle<Result<()>>>,
    parser_tasks: Vec<JoinHandle<Result<()>>>,
    filter_tasks: Vec<JoinHandle<Result<()>>>,
    output_tasks: Vec<JoinHandle<Result<()>>>,
    shutdown_sender: Option<mpsc::Sender<()>>,
    #[allow(dead_code)]
    plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
}

impl LogOrchestrator {
    /// Create a new log orchestrator
    #[allow(dead_code)]
    pub fn new(config: LogsConfig) -> Self {
        Self::new_with_plugins(config, None)
    }

    /// Create a new log orchestrator with plugins
    pub fn new_with_plugins(
        config: LogsConfig,
        plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
    ) -> Self {
        Self {
            config,
            running: false,
            source_tasks: Vec::new(),
            parser_tasks: Vec::new(),
            filter_tasks: Vec::new(),
            output_tasks: Vec::new(),
            shutdown_sender: None,
            plugin_manager,
        }
    }

    /// Start the log processing pipeline
    pub async fn start(&mut self) -> Result<()> {
        if self.running {
            return Err(anyhow!("Orchestrator is already running"));
        }

        println!("Starting log processing pipeline...");

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_sender = Some(shutdown_tx);

        // Create channels for pipeline communication
        let (raw_lines_tx, raw_lines_rx) =
            mpsc::channel::<RawLogEntry>(self.config.global.buffer_size);
        let (parsed_entries_tx, parsed_entries_rx) =
            mpsc::channel::<LogEntry>(self.config.global.buffer_size);
        let (filtered_entries_tx, filtered_entries_rx) =
            mpsc::channel::<LogEntry>(self.config.global.buffer_size);
        let (output_entries_tx, output_entries_rx) =
            mpsc::channel::<ShipperLogEntry>(self.config.global.buffer_size);

        // Create concurrency semaphore for parser tasks
        let parser_semaphore = Arc::new(Semaphore::new(10)); // Default concurrent parsers

        // Start source tasks
        self.start_source_tasks(raw_lines_tx.clone()).await?;

        // Start parser tasks
        self.start_parser_tasks(raw_lines_rx, parsed_entries_tx.clone(), parser_semaphore)
            .await?;

        // Start filter tasks
        self.start_filter_tasks(parsed_entries_rx, filtered_entries_tx.clone())
            .await?;

        // Start output tasks
        self.start_output_tasks(filtered_entries_rx, output_entries_tx.clone())
            .await?;

        // Start shipper tasks
        self.start_shipper_tasks(output_entries_rx).await?;

        self.running = true;
        println!("Log processing pipeline started successfully");

        // Wait for shutdown signal
        let _ = shutdown_rx.recv().await;
        println!("Shutdown signal received, stopping pipeline...");

        Ok(())
    }

    /// Stop the log processing pipeline
    pub async fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Ok(());
        }

        println!("Stopping log processing pipeline...");

        // Send shutdown signal
        if let Some(sender) = &self.shutdown_sender {
            let _ = sender.send(()).await;
        }

        // Wait for tasks to complete with timeout
        let shutdown_timeout = Duration::from_secs(30);
        let shutdown_result = timeout(shutdown_timeout, self.wait_for_tasks()).await;

        match shutdown_result {
            Ok(_) => println!("Pipeline stopped gracefully"),
            Err(_) => println!("Pipeline shutdown timed out, forcing stop"),
        }

        self.running = false;
        Ok(())
    }

    /// Start source tasks for each configured log source
    async fn start_source_tasks(&mut self, lines_sender: mpsc::Sender<RawLogEntry>) -> Result<()> {
        for source in &self.config.sources {
            if !source.enabled {
                continue;
            }

            let source_clone = source.clone();
            let sender_clone = lines_sender.clone();

            let plugin_manager = self.plugin_manager.clone();
            let task = tokio::spawn(async move {
                Self::run_source_task(source_clone, sender_clone, plugin_manager).await
            });

            self.source_tasks.push(task);
        }

        Ok(())
    }

    /// Run a single source task
    async fn run_source_task(
        source: LogSource,
        lines_sender: mpsc::Sender<RawLogEntry>,
        plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
    ) -> Result<()> {
        // Create an internal channel for raw strings
        let (string_tx, mut string_rx) = mpsc::channel::<String>(100);

        // Spawn a task to convert strings to RawLogEntry
        let source_name = source.name.clone();
        let source_labels = source.labels.clone();
        let converter_sender = lines_sender.clone();
        tokio::spawn(async move {
            while let Some(line) = string_rx.recv().await {
                let raw_entry = RawLogEntry {
                    line,
                    source: source_name.clone(),
                    labels: source_labels.clone(),
                };
                if converter_sender.send(raw_entry).await.is_err() {
                    break; // Receiver closed
                }
            }
        });

        // Support different source types
        match source.source_type.as_str() {
            "file" => {
                let file_source = FileLogSource::new(source.clone())?;
                file_source.start_tailing(string_tx).await?;
            }
            "plugin" => {
                // Handle plugin sources
                if let (Some(plugin_name), Some(plugin_source_name)) =
                    (&source.plugin_name, &source.plugin_source_name)
                {
                    if let Some(pm) = plugin_manager {
                        let pm_clone = pm.clone();
                        let plugin_name = plugin_name.clone();
                        let plugin_source_name = plugin_source_name.clone();
                        let config = serde_json::to_value(&source).unwrap_or_default();

                        // Run plugin source in a task
                        tokio::spawn(async move {
                            loop {
                                // Call the plugin to get log data and handle result immediately
                                let entries = {
                                    let pm_read = pm_clone.read().unwrap();
                                    match pm_read.execute_log_source(
                                        &plugin_name,
                                        &plugin_source_name,
                                        &config,
                                    ) {
                                        Ok(entries) => Some(entries),
                                        Err(e) => {
                                            let error_msg = format!("{}", e);
                                            eprintln!(
                                                "Error executing plugin log source {}:{}: {}",
                                                plugin_name, plugin_source_name, error_msg
                                            );
                                            None
                                        }
                                    }
                                };

                                if let Some(entries) = entries {
                                    for entry in entries {
                                        if string_tx.send(entry.raw.clone()).await.is_err() {
                                            return; // Channel closed
                                        }
                                    }
                                    tokio::time::sleep(Duration::from_millis(100)).await;
                                // Polling interval
                                } else {
                                    // Error occurred, wait before retrying
                                    tokio::time::sleep(Duration::from_secs(5)).await;
                                }
                            }
                        });
                    } else {
                        eprintln!(
                            "Plugin manager not available for plugin source: {}",
                            source.name
                        );
                    }
                } else {
                    eprintln!(
                        "Plugin source missing plugin_name or plugin_source_name: {}",
                        source.name
                    );
                }
            }
            _ => {
                // Default to file source for backward compatibility
                let file_source = FileLogSource::new(source.clone())?;
                file_source.start_tailing(string_tx).await?;
            }
        }
        Ok(())
    }

    /// Start parser tasks
    async fn start_parser_tasks(
        &mut self,
        lines_receiver: mpsc::Receiver<RawLogEntry>,
        entries_sender: mpsc::Sender<LogEntry>,
        semaphore: Arc<Semaphore>,
    ) -> Result<()> {
        // Collect all parser configs from sources
        let mut all_parser_configs = Vec::new();
        for source in &self.config.sources {
            if source.enabled {
                all_parser_configs.push(source.parser.clone());
            }
        }

        let plugin_manager = self.plugin_manager.clone();
        let task = tokio::spawn(async move {
            Self::run_parser_pipeline(
                lines_receiver,
                entries_sender,
                all_parser_configs,
                semaphore,
                plugin_manager,
            )
            .await
        });

        self.parser_tasks.push(task);
        Ok(())
    }

    /// Run the parser pipeline
    async fn run_parser_pipeline(
        mut lines_receiver: mpsc::Receiver<RawLogEntry>,
        entries_sender: mpsc::Sender<LogEntry>,
        parser_configs: Vec<ParserConfig>,
        semaphore: Arc<Semaphore>,
        plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
    ) -> Result<()> {
        // Create parsers from configs
        let mut parsers = Vec::new();
        for config in parser_configs {
            let parser = create_parser(&config, plugin_manager.clone())?;
            parsers.push(parser);
        }

        // If no parsers configured, use default plain parser
        if parsers.is_empty() {
            parsers.push(create_parser(
                &ParserConfig::default(),
                plugin_manager.clone(),
            )?);
        }

        while let Some(raw_entry) = lines_receiver.recv().await {
            let _permit = semaphore.acquire().await?;

            for parser in &parsers {
                match parser.parse(&raw_entry.line) {
                    Ok(mut entry) => {
                        // Populate source and labels from the raw entry
                        entry.source = raw_entry.source.clone();
                        entry.labels = raw_entry.labels.clone();

                        if entries_sender.send(entry).await.is_err() {
                            break; // Channel closed
                        }
                    }
                    Err(e) => {
                        eprintln!("Parser error: {}", e);
                        // Continue with other parsers
                    }
                }
            }
        }

        Ok(())
    }

    /// Start filter tasks
    async fn start_filter_tasks(
        &mut self,
        entries_receiver: mpsc::Receiver<LogEntry>,
        filtered_sender: mpsc::Sender<LogEntry>,
    ) -> Result<()> {
        // Collect all filter configs from sources and global filters
        let mut all_filter_configs = self.config.processing.global_filters.clone();
        for source in &self.config.sources {
            if source.enabled {
                all_filter_configs.extend(source.filters.clone());
            }
        }

        let plugin_manager = self.plugin_manager.clone();
        let task = tokio::spawn(async move {
            Self::run_filter_pipeline(
                entries_receiver,
                filtered_sender,
                all_filter_configs,
                plugin_manager,
            )
            .await
        });

        self.filter_tasks.push(task);
        Ok(())
    }

    /// Run the filter pipeline
    async fn run_filter_pipeline(
        mut entries_receiver: mpsc::Receiver<LogEntry>,
        filtered_sender: mpsc::Sender<LogEntry>,
        filter_configs: Vec<FilterConfig>,
        plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
    ) -> Result<()> {
        // Create filters from configs
        let mut filters = Vec::new();
        for config in filter_configs {
            let filter = create_filter(&config, plugin_manager.clone())?;
            filters.push(filter);
        }

        while let Some(entry) = entries_receiver.recv().await {
            let mut should_forward = true;

            for filter in &filters {
                match filter.filter(&entry) {
                    Ok(result) => {
                        if !result {
                            should_forward = false;
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Filter error: {}", e);
                        // Continue with other filters
                    }
                }
            }

            if should_forward && filtered_sender.send(entry).await.is_err() {
                break; // Channel closed
            }
        }

        Ok(())
    }

    /// Start output preparation tasks
    async fn start_output_tasks(
        &mut self,
        filtered_receiver: mpsc::Receiver<LogEntry>,
        output_sender: mpsc::Sender<ShipperLogEntry>,
    ) -> Result<()> {
        let task = tokio::spawn(async move {
            Self::run_output_preparation(filtered_receiver, output_sender).await
        });

        self.output_tasks.push(task);
        Ok(())
    }

    /// Run output preparation
    async fn run_output_preparation(
        mut filtered_receiver: mpsc::Receiver<LogEntry>,
        output_sender: mpsc::Sender<ShipperLogEntry>,
    ) -> Result<()> {
        while let Some(entry) = filtered_receiver.recv().await {
            // Convert to shipper log entry format
            let shipper_entry = ShipperLogEntry {
                message: entry.message.unwrap_or_else(|| entry.raw.clone()),
                timestamp: entry.timestamp,
                fields: entry.fields,
                source: entry.source.clone(), // Track actual source through pipeline
                labels: entry.labels.clone(), // Track labels through pipeline
            };

            if output_sender.send(shipper_entry).await.is_err() {
                break; // Channel closed
            }
        }

        Ok(())
    }

    /// Start shipper tasks for each output
    async fn start_shipper_tasks(
        &mut self,
        output_receiver: mpsc::Receiver<ShipperLogEntry>,
    ) -> Result<()> {
        let outputs = self.config.outputs.clone();

        let plugin_manager = self.plugin_manager.clone();
        let task = tokio::spawn(async move {
            Self::run_shipper_tasks(output_receiver, outputs, plugin_manager).await
        });

        self.output_tasks.push(task);
        Ok(())
    }

    /// Run shipper tasks for all outputs
    async fn run_shipper_tasks(
        mut receiver: mpsc::Receiver<ShipperLogEntry>,
        outputs: Vec<LogOutput>,
        plugin_manager: Option<Arc<RwLock<crate::plugins::PluginManager>>>,
    ) -> Result<()> {
        // Create writers for each output
        let mut writers = Vec::new();
        for output in outputs {
            let writer: Box<dyn LogOutputWriter> = match &output {
                LogOutput::File(_) => {
                    let config = match &output {
                        LogOutput::File(f) => f.clone(),
                        _ => unreachable!(),
                    };
                    create_file_output(config)?
                }
                LogOutput::S3(_) => {
                    let config = match &output {
                        LogOutput::S3(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    create_s3_output(config).await?
                }
                LogOutput::Http(_) => {
                    let config = match &output {
                        LogOutput::Http(h) => h.clone(),
                        _ => unreachable!(),
                    };
                    create_http_output(config).await?
                }
                LogOutput::Syslog(_) => {
                    let config = match &output {
                        LogOutput::Syslog(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    create_syslog_output(config)?
                }
                LogOutput::Console(_) => {
                    let config = match &output {
                        LogOutput::Console(c) => c.clone(),
                        _ => unreachable!(),
                    };
                    create_console_output(config)?
                }
                LogOutput::Plugin(_) => {
                    let config = match &output {
                        LogOutput::Plugin(p) => p.clone(),
                        _ => unreachable!(),
                    };
                    if let Some(pm) = &plugin_manager {
                        create_plugin_output(config, pm.clone()).await?
                    } else {
                        return Err(anyhow!("Plugin manager required for plugin outputs"));
                    }
                }
            };
            writers.push(writer);
        }

        while let Some(entry) = receiver.recv().await {
            for writer in &mut writers {
                writer.write_entry(&entry).await?;
            }
        }

        Ok(())
    }

    /// Wait for all tasks to complete
    async fn wait_for_tasks(&mut self) {
        // Wait for source tasks
        for task in self.source_tasks.drain(..) {
            let _ = task.await;
        }

        // Wait for parser tasks
        for task in self.parser_tasks.drain(..) {
            let _ = task.await;
        }

        // Wait for filter tasks
        for task in self.filter_tasks.drain(..) {
            let _ = task.await;
        }

        // Wait for output tasks
        for task in self.output_tasks.drain(..) {
            let _ = task.await;
        }
    }

    /// Get the number of configured sources
    pub fn source_count(&self) -> usize {
        self.config.sources.len()
    }

    /// Get the number of configured outputs
    pub fn output_count(&self) -> usize {
        self.config.outputs.len()
    }
}
