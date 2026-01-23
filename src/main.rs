use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod apply;
mod doc_extractor;
mod docs;
mod facts;
mod logs;

#[derive(Parser)]
#[command(name = "driftless")]
#[command(
    about = "Streamlined system configuration, inventory, and monitoring agent with configuration operations, facts collectors, and log sources/outputs"
)]
#[command(version)]
struct Cli {
    /// Configuration directory (default: ~/.config/driftless/config)
    #[arg(short, long)]
    config: Option<PathBuf>,

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
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Use default config directory if not specified
    let config_dir = cli.config.unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("driftless")
            .join("config")
    });

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

                    // TODO: Implement actual facts collection and export
                    println!("Facts collection started...");
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
                    } else {
                        println!("No enabled sources or outputs - nothing to do");
                    }

                    // TODO: Implement actual log collection and forwarding
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
                    // Generate task documentation
                    let task_docs = docs::generate_task_documentation()?;
                    let task_output_path = PathBuf::from(&output_dir).join("tasks-reference.md");
                    std::fs::write(&task_output_path, task_docs)?;
                    println!(
                        "Task documentation generated: {}",
                        task_output_path.display()
                    );

                    // Generate facts documentation
                    let facts_docs = docs::generate_facts_documentation()?;
                    let facts_output_path = PathBuf::from(&output_dir).join("facts-reference.md");
                    std::fs::write(&facts_output_path, facts_docs)?;
                    println!(
                        "Facts documentation generated: {}",
                        facts_output_path.display()
                    );

                    // Generate logs documentation
                    let logs_docs = docs::generate_logs_documentation()?;
                    let logs_output_path = PathBuf::from(&output_dir).join("logs-reference.md");
                    std::fs::write(&logs_output_path, logs_docs)?;
                    println!(
                        "Logs documentation generated: {}",
                        logs_output_path.display()
                    );

                    // Generate template documentation
                    let template_docs = docs::generate_template_documentation()?;
                    let template_output_path =
                        PathBuf::from(&output_dir).join("template-reference.md");
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
        Commands::Agent { port } => {
            println!("Starting agent mode with config: {}", config_dir.display());
            println!(
                "Metrics endpoint will be available at http://0.0.0.0:{}/metrics",
                port
            );
            // TODO: Implement agent mode
        }
    }

    Ok(())
}

/// Load facts configuration from the config directory
fn load_facts_config(config_dir: &PathBuf) -> anyhow::Result<facts::FactsConfig> {
    use std::fs;

    // Look for facts configuration files
    let facts_files = find_facts_config_files(config_dir)?;

    if facts_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No facts configuration files found in {}",
            config_dir.display()
        ));
    }

    // For now, load the first facts config file found
    // TODO: Support merging multiple facts config files
    let config_file = &facts_files[0];
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

    Ok(config)
}

/// Find facts configuration files in the config directory
fn find_facts_config_files(config_dir: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;

    if !config_dir.exists() {
        return Err(anyhow::anyhow!(
            "Config directory does not exist: {}",
            config_dir.display()
        ));
    }

    let mut facts_files = Vec::new();

    // Look for files that might contain facts configuration
    // Priority order: facts.{json,toml,yaml,yml}, then any .{json,toml,yaml,yml} files
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
                    facts_files.push(path);
                }
            }
        }
    }

    // Sort files for consistent ordering (facts.* files first)
    facts_files.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let a_is_facts = a_name.starts_with("facts.");
        let b_is_facts = b_name.starts_with("facts.");

        if a_is_facts && !b_is_facts {
            std::cmp::Ordering::Less
        } else if !a_is_facts && b_is_facts {
            std::cmp::Ordering::Greater
        } else {
            a_name.cmp(b_name)
        }
    });

    Ok(facts_files)
}

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

    // For now, load the first logs config file found
    // TODO: Support merging multiple logs config files
    let config_file = &logs_files[0];
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

    Ok(config)
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
    }
}

/// Load apply configuration from the config directory
fn load_apply_config(config_dir: &PathBuf) -> anyhow::Result<apply::ApplyConfig> {
    use std::fs;

    // Look for apply configuration files
    let apply_files = find_apply_config_files(config_dir)?;

    if apply_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No apply configuration files found in {}",
            config_dir.display()
        ));
    }

    // For now, load the first apply config file found
    // TODO: Support merging multiple apply config files
    let config_file = &apply_files[0];
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

    println!("DEBUG: Loaded config with {} vars", config.vars.len());
    for (k, v) in &config.vars {
        println!("DEBUG: Config var {} = {:?}", k, v);
    }
    Ok(config)
}

/// Find apply configuration files in the config directory
fn find_apply_config_files(config_dir: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;

    if !config_dir.exists() {
        return Err(anyhow::anyhow!(
            "Config directory does not exist: {}",
            config_dir.display()
        ));
    }

    let mut apply_files = Vec::new();

    // Look for files that might contain apply configuration
    // Priority order: apply.{json,toml,yaml,yml}, then any .{json,toml,yaml,yml} files
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
                    apply_files.push(path);
                }
            }
        }
    }

    // Sort files for consistent ordering (apply.* files first, then by name)
    apply_files.sort_by(|a, b| {
        let a_name = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let b_name = b.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let a_is_apply = a_name.starts_with("apply.");
        let b_is_apply = b_name.starts_with("apply.");

        if a_is_apply && !b_is_apply {
            std::cmp::Ordering::Less
        } else if !a_is_apply && b_is_apply {
            std::cmp::Ordering::Greater
        } else {
            a_name.cmp(b_name)
        }
    });

    Ok(apply_files)
}
