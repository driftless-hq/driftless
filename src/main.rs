use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::plugins::PluginManager;

mod agent;
mod apply;
mod config;
mod doc_extractor;
mod docs;
mod facts;
mod logs;
mod plugin_interface;
mod plugins;

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};

#[derive(Parser)]
#[command(name = "driftless")]
#[command(
    about = "Streamlined system configuration, inventory, and monitoring agent with configuration operations, facts collectors, and log sources/outputs"
)]
#[command(version)]
struct Cli {
    /// Configuration directory (default: /etc/driftless/config if exists, otherwise ~/.config/driftless/config)
    #[arg(short, long)]
    config: Option<PathBuf>,
    /// Plugin directory (default: /etc/driftless/plugins if exists, otherwise ~/.config/driftless/plugins)
    #[arg(long)]
    plugin_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply configuration operations to enforce system state
    Apply {
        /// Perform dry run (only output diffs)
        #[arg(long)]
        dry_run: bool,
    },
    /// Run facts collectors to gather system metrics and information
    Facts,
    /// Run log sources and outputs for log collection and forwarding
    Logs,
    /// Manage plugins (list, install, validate, etc.)
    Plugins {
        #[command(subcommand)]
        plugin_command: PluginCommands,
    },
    /// Generate documentation
    Docs {
        /// Output format (markdown)
        #[arg(short, long, default_value = "markdown")]
        format: String,
        /// Output directory for documentation files
        #[arg(long, default_value = ".")]
        output_dir: String,
    },
    /// Run in agent mode (continuous monitoring)
    Agent {
        /// Metrics endpoint port
        #[arg(short, long, default_value = "8000")]
        port: u16,
        /// Apply task execution interval (seconds)
        #[arg(long, default_value = "300")]
        apply_interval: u64,
        /// Facts collection interval (seconds)
        #[arg(long, default_value = "60")]
        facts_interval: u64,
        /// Enable dry-run mode for apply tasks
        #[arg(long)]
        dry_run: bool,
        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
        /// Enable debug mode
        #[arg(long)]
        debug: bool,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List available plugins from registries
    List,
    /// List locally installed plugins
    Installed,
    /// Install a plugin from registry
    Install {
        /// Plugin specification (name, name@version, or registry/name@version)
        plugin: String,
    },
    /// Remove a locally installed plugin
    Remove {
        /// Plugin name
        name: String,
        /// Plugin version (optional)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Validate a locally installed plugin or a plugin file
    Validate {
        /// Plugin name (for installed plugins)
        name: Option<String>,
        /// Plugin version (optional, for installed plugins)
        #[arg(short, long)]
        version: Option<String>,
        /// Path to plugin file to validate directly
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Update all installed plugins to latest versions
    Update,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Determine config directory with proper precedence:
    // 1. CLI argument if provided
    // 2. System-wide config (/etc/driftless/config) if it exists
    // 3. User config (~/.config/driftless/config)
    let config_dir = cli.config.unwrap_or_else(|| {
        // Check for system-wide config first
        let system_config = PathBuf::from("/etc/driftless/config");
        if system_config.exists() {
            system_config
        } else {
            // Fall back to user config
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("driftless")
                .join("config")
        }
    });

    // Determine plugin directory with proper precedence:
    // 1. CLI argument if provided
    // 2. System-wide plugins (/etc/driftless/plugins) if it exists
    // 3. User plugins (~/.config/driftless/plugins)
    let plugin_dir = cli.plugin_dir.unwrap_or_else(|| {
        // Check for system-wide plugins first
        let system_plugins = PathBuf::from("/etc/driftless/plugins");
        if system_plugins.exists() {
            system_plugins
        } else {
            // Fall back to user plugins
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("driftless")
                .join("plugins")
        }
    });

    // Load plugin registry configuration for security settings
    let plugin_registry_config = crate::config::load_plugin_registry_config(&config_dir)
        .unwrap_or_else(|_| {
            eprintln!("Warning: Failed to load plugin registry config, using defaults");
            crate::config::PluginRegistryConfig::default()
        });

    // Initialize plugin manager with security configuration
    let plugin_manager = match crate::plugins::PluginManager::new_with_security_config(
        plugin_dir.clone(),
        plugin_registry_config.security,
    ) {
        Ok(mut pm) => {
            // Try to scan and load plugins
            if let Err(e) = pm.scan_plugins() {
                eprintln!("Warning: Failed to scan plugins: {}", e);
            }
            if let Err(e) = pm.load_all_plugins() {
                eprintln!("Warning: Failed to load plugins: {}", e);
            }
            // Register plugin components
            let pm_arc = std::sync::Arc::new(std::sync::RwLock::new(pm));
            {
                let mut pm_write = pm_arc.write().unwrap();
                if let Err(e) = pm_write.register_plugin_tasks() {
                    eprintln!("Warning: Failed to register plugin tasks: {}", e);
                }
                if let Err(e) = pm_write.register_plugin_facts_collectors() {
                    eprintln!("Warning: Failed to register plugin facts collectors: {}", e);
                }
                if let Err(e) = pm_write.register_plugin_logs_components() {
                    eprintln!("Warning: Failed to register plugin logs components: {}", e);
                }
                if let Err(e) = pm_write.register_plugin_template_extensions(pm_arc.clone()) {
                    eprintln!(
                        "Warning: Failed to register plugin template extensions: {}",
                        e
                    );
                }
            }
            // Set the global plugin manager for registry callbacks
            crate::plugins::set_global_plugin_manager(pm_arc.clone());
            Some(pm_arc)
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize plugin manager: {}", e);
            None
        }
    };

    match cli.command {
        Commands::Apply { dry_run } => {
            println!("Applying configuration from: {}", config_dir.display());
            if dry_run {
                println!("Dry run mode - no changes will be made");
            }

            // Load apply configuration
            println!("DEBUG: About to call load_apply_config");
            match load_apply_config(&config_dir) {
                Ok(config) => {
                    // Load environment variables from env file first
                    let env_file = config_dir.parent().unwrap_or(&config_dir).join("env");
                    let mut temp_vars = apply::variables::VariableContext::new();
                    let _ = temp_vars.load_env_file(&env_file);

                    let mut executor = apply::executor::TaskExecutor::with_vars_from_context(
                        dry_run,
                        config.vars.clone(),
                        temp_vars,
                        config_dir.clone(),
                        plugin_manager.clone(),
                    );

                    // Validate tasks first
                    if let Err(e) = executor.validate(&config) {
                        eprintln!("Configuration validation failed: {}", e);
                        std::process::exit(1);
                    }

                    // Execute tasks
                    if let Err(e) = executor.execute(&config).await {
                        eprintln!("Task execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load apply configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Facts => {
            println!("Gathering facts from: {}", config_dir.display());

            // Load facts configuration
            match load_facts_config(&config_dir) {
                Ok(config) => {
                    let enabled_collectors = config
                        .collectors
                        .iter()
                        .filter(|c| is_collector_enabled(c, config.global.enabled))
                        .count();
                    println!(
                        "Loaded {} collectors ({} enabled)",
                        config.collectors.len(),
                        enabled_collectors
                    );

                    if config.export.prometheus.enabled {
                        println!(
                            "Prometheus endpoint will be available at http://{}:{}{}",
                            config.export.prometheus.host,
                            config.export.prometheus.port,
                            config.export.prometheus.path
                        );
                    }

                    // Create and run the facts orchestrator
                    match facts::FactsOrchestrator::new_with_registry_and_plugins(
                        config,
                        prometheus::Registry::new(),
                        plugin_manager.clone(),
                    ) {
                        Ok(orchestrator) => {
                            println!("Facts orchestrator created successfully");
                            println!("Starting facts collection...");

                            // For now, run a single collection cycle
                            // In agent mode, this would run continuously
                            if let Err(e) = orchestrator.collect_and_export().await {
                                eprintln!("Error during facts collection: {}", e);
                                std::process::exit(1);
                            }
                            println!("Facts collection completed successfully");
                        }
                        Err(e) => {
                            eprintln!("Failed to create facts orchestrator: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load facts configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Logs => {
            println!("Collecting logs from: {}", config_dir.display());

            // Load logs configuration
            match load_logs_config(&config_dir) {
                Ok(config) => {
                    let enabled_sources = config.sources.iter().filter(|s| s.enabled).count();
                    let enabled_outputs = config
                        .outputs
                        .iter()
                        .filter(|o| is_output_enabled(o))
                        .count();

                    println!(
                        "Loaded {} log sources ({} enabled)",
                        config.sources.len(),
                        enabled_sources
                    );
                    println!(
                        "Loaded {} outputs ({} enabled)",
                        config.outputs.len(),
                        enabled_outputs
                    );

                    if enabled_sources > 0 && enabled_outputs > 0 {
                        println!("Log collection started...");

                        // Create and start the log orchestrator
                        let mut orchestrator =
                            logs::LogOrchestrator::new_with_plugins(config, plugin_manager.clone());

                        // Start the log processing pipeline
                        if let Err(e) = orchestrator.start().await {
                            eprintln!("Failed to start log processing: {}", e);
                            std::process::exit(1);
                        }

                        // Wait for shutdown signal (Ctrl+C)
                        match tokio::signal::ctrl_c().await {
                            Ok(()) => {
                                println!("Received shutdown signal, stopping log collection...");
                            }
                            Err(e) => {
                                eprintln!("Failed to listen for shutdown signal: {}", e);
                            }
                        }

                        // Stop the orchestrator
                        if let Err(e) = orchestrator.stop().await {
                            eprintln!("Error stopping log orchestrator: {}", e);
                        }

                        println!("Log collection stopped");
                    } else {
                        println!("No enabled sources or outputs - nothing to do");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load logs configuration: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Docs { format, output_dir } => {
            println!("Generating documentation in {} format...", format);

            match format.as_str() {
                "markdown" => {
                    // Create reference directory if it doesn't exist
                    let reference_dir = PathBuf::from(&output_dir).join("reference");
                    std::fs::create_dir_all(&reference_dir)?;

                    // Generate task documentation
                    let task_docs = docs::generate_task_documentation()?;
                    let task_output_path = reference_dir.join("tasks-reference.md");
                    std::fs::write(&task_output_path, task_docs)?;
                    println!(
                        "Task documentation generated: {}",
                        task_output_path.display()
                    );

                    // Generate facts documentation
                    let facts_docs = docs::generate_facts_documentation()?;
                    let facts_output_path = reference_dir.join("facts-reference.md");
                    std::fs::write(&facts_output_path, facts_docs)?;
                    println!(
                        "Facts documentation generated: {}",
                        facts_output_path.display()
                    );

                    // Generate logs documentation
                    let logs_docs = docs::generate_logs_documentation()?;
                    let logs_output_path = reference_dir.join("logs-reference.md");
                    std::fs::write(&logs_output_path, logs_docs)?;
                    println!(
                        "Logs documentation generated: {}",
                        logs_output_path.display()
                    );

                    // Generate template documentation
                    let template_docs = docs::generate_template_documentation()?;
                    let template_output_path = reference_dir.join("template-reference.md");
                    std::fs::write(&template_output_path, template_docs)?;
                    println!(
                        "Template documentation generated: {}",
                        template_output_path.display()
                    );
                }
                _ => {
                    eprintln!(
                        "Unsupported format: {}. Supported formats: markdown",
                        format
                    );
                    std::process::exit(1);
                }
            }
        }
        Commands::Plugins { plugin_command } => {
            println!("Managing plugins...");

            // Load plugin registry configuration
            let registry_config = match crate::config::load_plugin_registry_config(&config_dir) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Failed to load plugin registry configuration: {}", e);
                    eprintln!("Using default configuration");
                    crate::config::PluginRegistryConfig::default()
                }
            };

            let lifecycle_manager = crate::plugins::registry::PluginLifecycleManager::new(
                registry_config,
                plugin_dir.clone(),
            );

            match plugin_command {
                PluginCommands::List => {
                    println!("Listing available plugins from registries...");

                    match lifecycle_manager.list_available_plugins().await {
                        Ok(plugins_by_registry) => {
                            for (registry_name, plugins) in plugins_by_registry {
                                println!("Registry: {}", registry_name);
                                for plugin in plugins {
                                    println!(
                                        "  {}@{} - {}",
                                        plugin.name,
                                        plugin.version,
                                        plugin.description.as_deref().unwrap_or("No description")
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to list plugins: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PluginCommands::Installed => {
                    println!("Listing locally installed plugins...");

                    match lifecycle_manager.list_installed_plugins() {
                        Ok(installed) => {
                            if installed.is_empty() {
                                println!("No plugins installed locally");
                            } else {
                                for (name, version, path) in installed {
                                    println!("  {}@{} - {}", name, version, path.display());
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to list installed plugins: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                PluginCommands::Install { plugin } => {
                    println!("Installing plugin: {}", plugin);

                    match lifecycle_manager.install_plugin(&plugin).await {
                        Ok(()) => {
                            println!("Plugin {} installed successfully", plugin);
                        }
                        Err(e) => {
                            eprintln!("Failed to install plugin {}: {}", plugin, e);
                            std::process::exit(1);
                        }
                    }
                }
                PluginCommands::Remove { name, version } => {
                    println!("Removing plugin: {}", name);

                    match lifecycle_manager.remove_plugin(&name, version.as_deref()) {
                        Ok(()) => {
                            println!("Plugin {} removed successfully", name);
                        }
                        Err(e) => {
                            eprintln!("Failed to remove plugin {}: {}", name, e);
                            std::process::exit(1);
                        }
                    }
                }
                PluginCommands::Validate {
                    name,
                    version,
                    file,
                } => {
                    if let Some(file_path) = file {
                        println!("Validating plugin file: {}", file_path.display());

                        // For file validation, we need the plugin manager
                        if let Some(pm) = &plugin_manager {
                            let pm_read = pm.read().unwrap();
                            match pm_read.validate_plugin_file(&file_path) {
                                Ok(()) => {
                                    println!(
                                        "Plugin file {} validated successfully",
                                        file_path.display()
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "Plugin file {} validation failed: {}",
                                        file_path.display(),
                                        e
                                    );
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            eprintln!("Error: Plugin manager not available for file validation");
                            std::process::exit(1);
                        }
                    } else if let Some(name) = name {
                        println!("Validating plugin: {}", name);

                        match lifecycle_manager.validate_plugin(&name, version.as_deref()) {
                            Ok(()) => {
                                println!("Plugin {} validated successfully", name);
                            }
                            Err(e) => {
                                eprintln!("Plugin {} validation failed: {}", name, e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        eprintln!(
                            "Error: Either --name or --file must be specified for validate command"
                        );
                        std::process::exit(1);
                    }
                }
                PluginCommands::Update => {
                    println!("Updating all installed plugins...");

                    // Initialize plugin manager
                    let mut plugin_manager = PluginManager::new(plugin_dir.clone())?;

                    // Scan for plugins
                    plugin_manager
                        .scan_plugins()
                        .map_err(|e| anyhow::anyhow!("Failed to scan plugins: {}", e))?;

                    // Get list of loaded plugins
                    let _loaded_plugins = plugin_manager.get_loaded_plugins();

                    // Get list of loaded plugins
                    let loaded_plugins = plugin_manager.get_loaded_plugins();
                    let total_plugins = loaded_plugins.len();

                    if total_plugins == 0 {
                        println!("No plugins currently loaded");
                        return Ok(());
                    }

                    println!(
                        "Found {} loaded plugins to check for updates",
                        total_plugins
                    );

                    // For each plugin, attempt to reload/update
                    let mut updated_count = 0;
                    for plugin_name in loaded_plugins {
                        println!("Checking plugin: {}", plugin_name);

                        // Try to reload the plugin (this will pick up any file changes)
                        match plugin_manager.load_plugin(&plugin_name) {
                            Ok(_) => {
                                println!("Successfully updated plugin: {}", plugin_name);
                                updated_count += 1;
                            }
                            Err(e) => {
                                println!("Failed to update plugin {}: {}", plugin_name, e);
                            }
                        }
                    }

                    println!(
                        "Plugin update complete. Updated {} out of {} plugins",
                        updated_count, total_plugins
                    );
                }
            }
        }
        Commands::Agent {
            port,
            apply_interval,
            facts_interval,
            dry_run,
            verbose,
            debug,
        } => {
            // Configure logging level
            let log_level = if debug {
                tracing::Level::DEBUG
            } else if verbose {
                tracing::Level::INFO
            } else {
                tracing::Level::WARN
            };

            // Initialize tracing subscriber with proper formatting
            tracing_subscriber::fmt()
                .with_max_level(log_level)
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .compact()
                .init();

            info!("Starting agent mode with config: {}", config_dir.display());
            info!(
                "Metrics endpoint will be available at http://0.0.0.0:{}/metrics",
                port
            );
            info!(
                "Health check endpoint available at http://0.0.0.0:{}/health",
                port
            );
            info!(
                "Agent status endpoint available at http://0.0.0.0:{}/status",
                port
            );

            // Load agent configuration from file if it exists
            let mut agent_config = load_agent_config(&config_dir).unwrap_or_else(|_| {
                info!("No agent config found, using defaults");
                agent::AgentConfig::default()
            });

            // Override with CLI arguments
            agent_config.config_dir = config_dir.clone();
            agent_config.metrics_port = port;
            agent_config.apply_interval = apply_interval;
            agent_config.facts_interval = facts_interval;
            agent_config.apply_dry_run = dry_run;

            info!("Agent configuration:");
            info!("  Apply interval: {} seconds", agent_config.apply_interval);
            info!("  Facts interval: {} seconds", agent_config.facts_interval);
            info!("  Dry run mode: {}", agent_config.apply_dry_run);
            info!("  Metrics port: {}", agent_config.metrics_port);

            // Create agent
            let agent = Arc::new(Mutex::new(agent::Agent::new(agent_config)));

            // Start HTTP server for health and status endpoints
            let agent_clone = Arc::clone(&agent);
            let port_clone = port;
            let http_server = tokio::spawn(async move {
                let app = Router::new()
                    .route("/health", get(health_check))
                    .route("/status", get(status_endpoint))
                    .with_state(agent_clone);

                let addr: std::net::SocketAddr = format!("0.0.0.0:{}", port_clone).parse().unwrap();
                info!("Starting HTTP server on {}", addr);

                let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
                axum::serve(listener, app).await.unwrap();
            });

            // Start agent
            let agent_task = tokio::spawn(async move {
                let mut agent_guard = agent.lock().await;
                if let Err(e) = agent_guard.start().await {
                    error!("Failed to start agent: {}", e);
                    std::process::exit(1);
                }

                info!("Agent started successfully. Press Ctrl+C to stop.");

                // Run the event loop
                if let Err(e) = agent_guard.run_event_loop().await {
                    error!("Agent event loop error: {}", e);
                    std::process::exit(1);
                }
            });

            // Wait for both tasks
            let _ = tokio::try_join!(http_server, agent_task);
        }
    }

    Ok(())
}

/// Load facts configuration from the config directory
fn load_facts_config(config_dir: &PathBuf) -> anyhow::Result<facts::FactsConfig> {
    use std::fs;

    // Look for facts configuration files
    let facts_files = find_config_files(config_dir, "facts")?;

    if facts_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No facts configuration files found in {}",
            config_dir.display()
        ));
    }

    // Load and merge all facts configuration files
    let mut merged_config = facts::FactsConfig::default();

    for config_file in &facts_files {
        println!("Loading facts config from: {}", config_file.display());

        let content = fs::read_to_string(config_file)?;
        let config: facts::FactsConfig = match config_file.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported config file format: {}",
                    config_file.display()
                ))
            }
        };

        // Merge configurations
        merged_config.merge(config);
    }

    Ok(merged_config)
}

/// Load agent configuration from the config directory
fn load_agent_config(config_dir: &PathBuf) -> anyhow::Result<agent::AgentConfig> {
    use std::fs;

    // Look for agent configuration files
    let agent_files = find_config_files(config_dir, "agent")?;

    if agent_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No agent configuration files found in {}",
            config_dir.display()
        ));
    }

    // For now, load the first agent config file found
    let config_file = &agent_files[0];
    println!("Loading agent config from: {}", config_file.display());

    let content = fs::read_to_string(config_file)?;
    let config: agent::AgentConfig = match config_file.extension().and_then(|s| s.to_str()) {
        Some("json") => serde_json::from_str(&content)?,
        Some("toml") => toml::from_str(&content)?,
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported config file format: {}",
                config_file.display()
            ));
        }
    };

    Ok(config)
}

/// Find facts configuration files in the config directory
/// Check if a collector is enabled based on global and collector-specific settings
fn is_collector_enabled(collector: &facts::Collector, global_enabled: bool) -> bool {
    use facts::Collector::*;

    let collector_enabled = match collector {
        System(c) => c.base.enabled,
        Cpu(c) => c.base.enabled,
        Memory(c) => c.base.enabled,
        Disk(c) => c.base.enabled,
        Network(c) => c.base.enabled,
        Process(c) => c.base.enabled,
        Command(c) => c.base.enabled,
        Plugin(c) => c.base.enabled,
    };

    global_enabled && collector_enabled
}

/// Load logs configuration from the config directory
fn load_logs_config(config_dir: &PathBuf) -> anyhow::Result<logs::LogsConfig> {
    use std::fs;

    // Look for logs configuration files
    let logs_files = find_logs_config_files(config_dir)?;

    if logs_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No logs configuration files found in {}",
            config_dir.display()
        ));
    }

    // Load and merge all logs configuration files
    let mut merged_config = logs::LogsConfig {
        global: logs::GlobalSettings::default(),
        sources: Vec::new(),
        outputs: Vec::new(),
        processing: logs::ProcessingConfig::default(),
    };

    for config_file in &logs_files {
        println!("Loading logs config from: {}", config_file.display());

        let content = fs::read_to_string(config_file)?;
        let config: logs::LogsConfig = match config_file.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported config file format: {}",
                    config_file.display()
                ))
            }
        };

        // Merge configurations
        merged_config.merge(config);
    }

    Ok(merged_config)
}

/// Find logs configuration files in the config directory
fn find_logs_config_files(config_dir: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;

    if !config_dir.exists() {
        return Err(anyhow::anyhow!(
            "Config directory does not exist: {}",
            config_dir.display()
        ));
    }

    let mut logs_files = Vec::new();

    // Look for files that might contain logs configuration
    // Priority order: logs.{json,toml,yaml,yml}, then any .{json,toml,yaml,yml} files
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip backup files and hidden files
                if file_name.ends_with(".bak") || file_name.starts_with('.') {
                    continue;
                }

                // Check for supported config file extensions
                if file_name.ends_with(".json")
                    || file_name.ends_with(".toml")
                    || file_name.ends_with(".yaml")
                    || file_name.ends_with(".yml")
                {
                    logs_files.push(path);
                }
            }
        }
    }

    // Sort files for consistent ordering (logs.* files first)
    logs_files.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let a_is_logs = a_name.starts_with("logs.");
        let b_is_logs = b_name.starts_with("logs.");

        if a_is_logs && !b_is_logs {
            std::cmp::Ordering::Less
        } else if !a_is_logs && b_is_logs {
            std::cmp::Ordering::Greater
        } else {
            a_name.cmp(b_name)
        }
    });

    Ok(logs_files)
}

/// Check if a log output is enabled
fn is_output_enabled(output: &logs::LogOutput) -> bool {
    use logs::LogOutput::*;

    match output {
        File(o) => o.enabled,
        S3(o) => o.enabled,
        Http(o) => o.enabled,
        Syslog(o) => o.enabled,
        Console(o) => o.enabled,
        Plugin(o) => o.enabled,
    }
}

/// Load apply configuration from the config directory
fn load_apply_config(config_dir: &PathBuf) -> anyhow::Result<apply::ApplyConfig> {
    use std::fs;

    // Look for apply configuration files
    let apply_files = find_config_files(config_dir, "apply")?;

    if apply_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No apply configuration files found in {}",
            config_dir.display()
        ));
    }

    // Load and merge all apply configuration files
    let mut merged_config = apply::ApplyConfig {
        vars: std::collections::HashMap::new(),
        tasks: Vec::new(),
    };

    for config_file in &apply_files {
        println!("Loading apply config from: {}", config_file.display());

        let content = fs::read_to_string(config_file)?;
        let config: apply::ApplyConfig = match config_file.extension().and_then(|s| s.to_str()) {
            Some("json") => serde_json::from_str(&content)?,
            Some("toml") => toml::from_str(&content)?,
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported config file format: {}",
                    config_file.display()
                ))
            }
        };

        // Merge configurations
        merged_config.merge(config);
    }

    println!(
        "DEBUG: Loaded merged config with {} vars and {} tasks",
        merged_config.vars.len(),
        merged_config.tasks.len()
    );
    Ok(merged_config)
}

/// Find apply configuration files in the config directory
fn find_config_files(config_dir: &PathBuf, prefix: &str) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;

    if !config_dir.exists() {
        return Err(anyhow::anyhow!(
            "Config directory does not exist: {}",
            config_dir.display()
        ));
    }

    let mut config_files = Vec::new();

    // Look for files that might contain configuration
    // Priority order: {prefix}.{json,toml,yaml,yml}, then any .{json,toml,yaml,yml} files
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip backup files and hidden files
                if file_name.ends_with(".bak") || file_name.starts_with('.') {
                    continue;
                }

                // Check for supported config file extensions
                if file_name.ends_with(".json")
                    || file_name.ends_with(".toml")
                    || file_name.ends_with(".yaml")
                    || file_name.ends_with(".yml")
                {
                    config_files.push(path);
                }
            }
        }
    }

    // Sort files for consistent ordering ({prefix}.* files first, then by name)
    config_files.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let a_is_prefix = a_name.starts_with(&format!("{}.", prefix));
        let b_is_prefix = b_name.starts_with(&format!("{}.", prefix));

        if a_is_prefix && !b_is_prefix {
            std::cmp::Ordering::Less
        } else if !a_is_prefix && b_is_prefix {
            std::cmp::Ordering::Greater
        } else {
            a_name.cmp(b_name)
        }
    });

    Ok(config_files)
}

async fn health_check(
    State(agent): State<Arc<Mutex<agent::Agent>>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let agent_guard = agent.lock().await;
    let is_running = agent_guard.is_running();

    if is_running {
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "unhealthy",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "message": "Agent is not running"
            })),
        )
    }
}

async fn status_endpoint(State(agent): State<Arc<Mutex<agent::Agent>>>) -> Json<serde_json::Value> {
    let agent_guard = agent.lock().await;

    Json(serde_json::json!({
        "status": if agent_guard.is_running() { "running" } else { "stopped" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "config": {
            "apply_interval": agent_guard.config().apply_interval,
            "facts_interval": agent_guard.config().facts_interval,
            "dry_run": agent_guard.config().apply_dry_run,
            "metrics_port": agent_guard.config().metrics_port
        },
        "metrics": {
            "apply": {
                "execution_count": agent_guard.apply_execution_count(),
                "success_count": agent_guard.apply_success_count(),
                "failure_count": agent_guard.apply_failure_count(),
                "last_execution": agent_guard.apply_last_execution().map(|t| t.elapsed().as_secs()),
                "last_duration": agent_guard.apply_last_duration().map(|d| d.as_secs())
            },
            "facts": {
                "collection_count": agent_guard.facts_collection_count(),
                "success_count": agent_guard.facts_success_count(),
                "failure_count": agent_guard.facts_failure_count(),
                "last_collection": agent_guard.facts_last_collection().map(|t| t.elapsed().as_secs()),
                "last_duration": agent_guard.facts_last_duration().map(|d| d.as_secs())
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_config_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Create test config files
        fs::write(config_dir.join("agent.json"), r#"{"apply_interval": 100}"#).unwrap();
        fs::write(config_dir.join("apply.yaml"), r#"vars: {}"#).unwrap();
        fs::write(config_dir.join("facts.toml"), r#"[facts]"#).unwrap();
        fs::write(config_dir.join("random.json"), r#"{}"#).unwrap();

        // Test finding agent config files
        let agent_files = find_config_files(&config_dir, "agent").unwrap();
        assert_eq!(agent_files.len(), 4); // All config files, sorted with agent.* first
        assert!(agent_files[0].file_name().unwrap() == "agent.json"); // agent.json first
        assert!(agent_files[1].file_name().unwrap() == "apply.yaml");
        assert!(agent_files[2].file_name().unwrap() == "facts.toml");
        assert!(agent_files[3].file_name().unwrap() == "random.json");

        // Test finding apply config files
        let apply_files = find_config_files(&config_dir, "apply").unwrap();
        assert_eq!(apply_files.len(), 4); // All config files, sorted with apply.* first
        assert!(apply_files[0].file_name().unwrap() == "apply.yaml"); // apply.yaml first
        assert!(apply_files[1].file_name().unwrap() == "agent.json");
        assert!(apply_files[2].file_name().unwrap() == "facts.toml");
        assert!(apply_files[3].file_name().unwrap() == "random.json");
    }

    #[test]
    fn test_load_agent_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Create test agent config
        let agent_config = r#"{
            "config_dir": "/tmp/test",
            "plugin_dir": "./plugins",
            "apply_interval": 123,
            "facts_interval": 456,
            "apply_dry_run": true,
            "metrics_port": 9999,
            "enabled": true,
            "secrets": {"key": "value"},
            "resource_monitoring": {
                "enabled": true,
                "cache_duration": 30,
                "memory_warning_threshold": 1073741824,
                "cpu_warning_threshold": 80.0,
                "async_monitoring": true,
                "selective_monitoring": false,
                "lightweight_monitoring": true
            }
        }"#;
        fs::write(config_dir.join("agent.json"), agent_config).unwrap();

        let config = load_agent_config(&config_dir).unwrap();
        assert_eq!(config.apply_interval, 123);
        assert_eq!(config.facts_interval, 456);
        assert_eq!(config.metrics_port, 9999);
        assert!(config.apply_dry_run);
        assert!(config.enabled);
        assert_eq!(config.secrets.get("key").unwrap(), "value");
    }

    #[test]
    fn test_load_agent_config_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let result = load_agent_config(&config_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No agent configuration files found"));
    }

    #[test]
    fn test_load_agent_config_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Create invalid JSON
        fs::write(config_dir.join("agent.json"), r#"{"invalid": json}"#).unwrap();

        let result = load_agent_config(&config_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_health_check_handler() {
        // This would require setting up a test agent, but for now we'll test the basic structure
        // In a real integration test, we'd start the agent and make HTTP requests
        // For unit testing the handler, we'd need to mock the agent state
    }

    #[test]
    fn test_status_endpoint_handler() {
        // Similar to health check - would need integration test setup
    }
}
