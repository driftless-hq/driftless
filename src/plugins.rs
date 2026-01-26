use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{error, info, warn};
use wasmtime::{Config, Engine, Instance, Linker, Module, Store, UpdateDeadline};

/// Plugin registry management
pub mod registry;

use crate::config::PluginSecurityConfig;

/// Default fuel limit for plugin execution (1 billion instructions)
const PLUGIN_FUEL_LIMIT: u64 = 1_000_000_000;

/// Type alias for the complex global plugin manager type
type GlobalPluginManager = Arc<RwLock<Option<Arc<RwLock<PluginManager>>>>>;

/// Global plugin manager instance for registry callbacks
static GLOBAL_PLUGIN_MANAGER: once_cell::sync::Lazy<GlobalPluginManager> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Set the global plugin manager instance
pub fn set_global_plugin_manager(manager: Arc<RwLock<PluginManager>>) {
    *GLOBAL_PLUGIN_MANAGER.write().unwrap() = Some(manager);
}

/// Parse a plugin component name in the format "plugin_name.component_name"
///
/// Returns a tuple of (plugin_name, component_name) or an error if the format is invalid.
pub fn parse_plugin_component_name(name: &str) -> Result<(&str, &str), anyhow::Error> {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() == 2 {
        Ok((parts[0], parts[1]))
    } else {
        Err(anyhow::anyhow!(
            "Invalid plugin component name format: {}. Expected 'plugin_name.component_name'",
            name
        ))
    }
}

/// Plugin metadata information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name (derived from filename)
    #[allow(dead_code)]
    pub name: String,
    /// Full path to the plugin file
    pub path: PathBuf,
    /// Plugin version (if available)
    #[allow(dead_code)]
    pub version: Option<String>,
    /// Plugin description (if available)
    #[allow(dead_code)]
    pub description: Option<String>,
    /// Whether the plugin is currently loaded
    pub loaded: bool,
    /// Load error if any
    pub load_error: Option<String>,
}

/// Plugin registry that manages plugin discovery, validation, and loading
pub struct PluginRegistry {
    /// Directory to scan for plugins
    plugin_dir: PathBuf,
    /// Discovered plugins
    discovered_plugins: HashMap<String, PluginInfo>,
    /// Whether the registry has been scanned
    scanned: bool,
}

impl PluginRegistry {
    /// Create a new plugin registry for the given directory
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugin_dir,
            discovered_plugins: HashMap::new(),
            scanned: false,
        }
    }

    /// Scan the plugin directory for plugin files
    pub fn scan_plugins(&mut self, _engine: &Engine) -> Result<(), Box<dyn std::error::Error>> {
        if !self.plugin_dir.exists() {
            // Create the plugin directory if it doesn't exist
            std::fs::create_dir_all(&self.plugin_dir)?;
            info!("Created plugin directory: {}", self.plugin_dir.display());
        }

        self.discovered_plugins.clear();

        // Scan for .wasm files
        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let plugin_name = file_stem.to_string();

                    let plugin_info = PluginInfo {
                        name: plugin_name.clone(),
                        path: path.clone(),
                        version: None,     // Metadata extraction deferred until plugin load
                        description: None, // Metadata extraction deferred until plugin load
                        loaded: false,
                        load_error: None,
                    };

                    self.discovered_plugins.insert(plugin_name, plugin_info);
                }
            }
        }

        self.scanned = true;
        info!(
            "Scanned {} plugins in {}",
            self.discovered_plugins.len(),
            self.plugin_dir.display()
        );

        Ok(())
    }

    /// Get all discovered plugins
    pub fn get_discovered_plugins(&self) -> &HashMap<String, PluginInfo> {
        &self.discovered_plugins
    }

    /// Get a specific plugin info
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginInfo> {
        self.discovered_plugins.get(name)
    }

    /// Check if the registry has been scanned
    pub fn is_scanned(&self) -> bool {
        self.scanned
    }

    /// Get the plugin directory
    pub fn plugin_dir(&self) -> &Path {
        &self.plugin_dir
    }

    /// Update plugin info after successful loading
    pub fn update_plugin_info(
        &mut self,
        plugin_name: &str,
        version: Option<String>,
        description: Option<String>,
    ) {
        if let Some(info) = self.discovered_plugins.get_mut(plugin_name) {
            info.loaded = true;
            info.load_error = None;
            info.version = version;
            info.description = description;
        }
    }

    /// Update plugin load error status
    pub fn update_plugin_load_error(&mut self, plugin_name: &str, error: String) {
        if let Some(info) = self.discovered_plugins.get_mut(plugin_name) {
            info.loaded = false;
            info.load_error = Some(error);
        }
    }
}

/// PluginManager handles loading and instantiating WebAssembly modules securely.
pub struct PluginManager {
    engine: Engine,
    registry: PluginRegistry,
    loaded_plugins: HashMap<String, Module>,
    security_config: PluginSecurityConfig,
    epoch_counter: Arc<std::sync::atomic::AtomicU64>,
}

impl PluginManager {
    /// Creates a new PluginManager with secure default configuration.
    #[allow(dead_code)]
    pub fn new(plugin_dir: PathBuf) -> Result<Self, wasmtime::Error> {
        Self::new_with_security_config(plugin_dir, PluginSecurityConfig::default())
    }

    /// Creates a new PluginManager with custom security configuration.
    pub fn new_with_security_config(
        plugin_dir: PathBuf,
        security_config: PluginSecurityConfig,
    ) -> Result<Self, wasmtime::Error> {
        let mut config = Config::new();

        // Apply security hardening
        config.max_wasm_stack(security_config.max_stack_size);
        config.memory_reservation(security_config.max_memory as u64);
        config.memory_reservation_for_growth(0); // No growth allowed
        config.consume_fuel(true);
        config.epoch_interruption(true);

        // Limit module complexity
        // Note: max_tables, max_memories, max_globals not available in this wasmtime version
        // These limits are enforced at runtime in validate_wasm_module

        // Disable debug features in production
        if !security_config.debug_enabled {
            config.debug_info(false);
            config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        }

        let engine = Engine::new(&config)?;
        let registry = PluginRegistry::new(plugin_dir);
        let epoch_counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        Ok(Self {
            engine,
            registry,
            loaded_plugins: HashMap::new(),
            security_config,
            epoch_counter,
        })
    }

    /// Scan for available plugins
    pub fn scan_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.registry.scan_plugins(&self.engine)
    }

    /// Validate a plugin file with comprehensive security checks
    pub fn validate_plugin(&self, plugin_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let plugin_info = self
            .registry
            .get_plugin_info(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found in registry", plugin_name))?;

        // Check if file exists
        if !plugin_info.path.exists() {
            return Err(format!(
                "Plugin file '{}' does not exist",
                plugin_info.path.display()
            )
            .into());
        }

        // Check file size (prevent zip bombs and extremely large modules)
        let metadata = plugin_info.path.metadata()?;
        let file_size = metadata.len();
        if file_size > 50 * 1024 * 1024 {
            // 50MB limit
            return Err(format!(
                "Plugin file '{}' is too large: {} bytes (max: 50MB)",
                plugin_info.path.display(),
                file_size
            )
            .into());
        }

        // Load and validate WASM module
        let module = match Module::from_file(&self.engine, &plugin_info.path) {
            Ok(module) => module,
            Err(e) => {
                return Err(
                    format!("Invalid WASM file '{}': {}", plugin_info.path.display(), e).into(),
                );
            }
        };

        // Perform security validation on the module
        self.validate_wasm_module(&module, plugin_name)?;

        Ok(())
    }

    /// Perform comprehensive security validation on a WASM module
    fn validate_wasm_module(
        &self,
        module: &Module,
        plugin_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get module information for validation
        let module_info = module.resources_required();

        // Check memory usage
        if module_info.num_memories > self.security_config.max_memories {
            return Err(format!(
                "Plugin '{}' exceeds maximum memory count: {} > {}",
                plugin_name, module_info.num_memories, self.security_config.max_memories
            )
            .into());
        }

        // Check table count
        if module_info.num_tables > self.security_config.max_tables {
            return Err(format!(
                "Plugin '{}' exceeds maximum table count: {} > {}",
                plugin_name, module_info.num_tables, self.security_config.max_tables
            )
            .into());
        }

        // Note: num_globals not available in this wasmtime version
        // Global count validation is skipped

        // Validate that the module doesn't import dangerous functions
        self.validate_module_imports(module, plugin_name)?;

        // Check for potentially dangerous exports
        self.validate_module_exports(module, plugin_name)?;

        Ok(())
    }

    /// Validate a plugin file directly with comprehensive security checks
    pub fn validate_plugin_file(
        &self,
        file_path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if file exists
        if !file_path.exists() {
            return Err(format!("Plugin file '{}' does not exist", file_path.display()).into());
        }

        // Check file size (prevent zip bombs and extremely large modules)
        let metadata = file_path.metadata()?;
        let file_size = metadata.len();
        if file_size > 50 * 1024 * 1024 {
            // 50MB limit
            return Err(format!(
                "Plugin file '{}' is too large: {} bytes (max: 50MB)",
                file_path.display(),
                file_size
            )
            .into());
        }

        // Load and validate WASM module
        let module = match Module::from_file(&self.engine, file_path) {
            Ok(module) => module,
            Err(e) => {
                return Err(format!("Invalid WASM file '{}': {}", file_path.display(), e).into());
            }
        };

        // Perform security validation on the module
        self.validate_wasm_module(&module, file_path.to_string_lossy().as_ref())?;

        Ok(())
    }

    /// Validate module imports for security
    fn validate_module_imports(
        &self,
        module: &Module,
        plugin_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get all imports
        for import in module.imports() {
            let module_name = import.module();
            let name = import.name();

            // Block potentially dangerous imports
            match (module_name, name) {
                // Block WASI if not explicitly allowed
                ("wasi_snapshot_preview1", _) if !self.security_config.allow_wasi => {
                    return Err(format!(
                        "Plugin '{}' imports forbidden WASI function: {}.{}",
                        plugin_name, module_name, name
                    )
                    .into());
                }
                // Block direct system access
                ("env", func_name)
                    if func_name.contains("syscall") || func_name.contains("system") =>
                {
                    return Err(format!(
                        "Plugin '{}' imports forbidden system function: {}.{}",
                        plugin_name, module_name, name
                    )
                    .into());
                }
                // Block file system access
                ("env", func_name) if func_name.contains("fd_") || func_name.contains("path_") => {
                    return Err(format!(
                        "Plugin '{}' imports forbidden file system function: {}.{}",
                        plugin_name, module_name, name
                    )
                    .into());
                }
                // Block network access
                ("env", func_name)
                    if func_name.contains("sock")
                        || func_name.contains("net")
                        || func_name.contains("connect")
                        || func_name.contains("bind")
                        || func_name.contains("listen")
                        || func_name.contains("accept")
                        || func_name.contains("send")
                        || func_name.contains("recv") =>
                {
                    return Err(format!(
                        "Plugin '{}' imports forbidden network function: {}.{}",
                        plugin_name, module_name, name
                    )
                    .into());
                }
                _ => {} // Allow other imports
            }
        }

        Ok(())
    }

    /// Validate module exports for security
    fn validate_module_exports(
        &self,
        module: &Module,
        plugin_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check that required plugin interface functions are exported
        let required_exports = [
            "get_task_definitions",
            "get_facts_collectors",
            "get_template_extensions",
            "get_log_sources",
            "get_log_parsers",
            "get_log_filters",
            "get_log_outputs",
        ];

        let mut has_required_export = false;
        for export in module.exports() {
            let export_name = export.name();

            // Check if this is a required plugin interface function
            if required_exports.contains(&export_name) {
                has_required_export = true;
            }

            // Block potentially dangerous exports
            if export_name.starts_with("unsafe_") || export_name.contains("syscall") {
                return Err(format!(
                    "Plugin '{}' exports forbidden function: {}",
                    plugin_name, export_name
                )
                .into());
            }
        }

        if !has_required_export {
            warn!(
                "Plugin '{}' does not export any standard plugin interface functions",
                plugin_name
            );
        }

        Ok(())
    }

    /// Load a specific plugin by name
    pub fn load_plugin(&mut self, plugin_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Validate the plugin first with comprehensive security checks
        self.validate_plugin(plugin_name)?;

        let plugin_info = self
            .registry
            .get_plugin_info(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found in registry", plugin_name))?;

        let module = Module::from_file(&self.engine, &plugin_info.path)?;

        // Extract metadata from the plugin if not already extracted during scanning
        let (version, description) =
            if plugin_info.version.is_some() || plugin_info.description.is_some() {
                // Metadata was already extracted during scanning
                (plugin_info.version.clone(), plugin_info.description.clone())
            } else {
                // Extract metadata with full security validation
                {
                    // Set up secure store for metadata extraction
                    let mut store = Store::new(&self.engine, ());
                    store.set_fuel(self.security_config.fuel_limit)?;

                    // Set up epoch-based timeout
                    let epoch_counter = Arc::clone(&self.epoch_counter);
                    store.epoch_deadline_callback(move |_| {
                        epoch_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        Ok(UpdateDeadline::Continue(1))
                    });

                    let linker = Linker::new(&self.engine);
                    // No WASI added for security - plugins have no host access
                    // unless explicitly allowed (which it shouldn't be)

                    // Instantiate the module temporarily for metadata extraction
                    let instance = linker.instantiate(&mut store, &module)?;

                    // Extract metadata with timeout protection
                    self.extract_plugin_metadata_with_timeout(&instance, &mut store, plugin_name)?
                }
            };

        // Store the module (not the instance)
        self.loaded_plugins.insert(plugin_name.to_string(), module);

        // Mark as loaded in registry and update metadata
        self.registry
            .update_plugin_info(plugin_name, version, description);

        info!("Successfully loaded plugin: {}", plugin_name);
        Ok(())
    }

    /// Load all discovered plugins
    pub fn load_all_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.registry.is_scanned() {
            self.scan_plugins()?;
        }

        let plugin_names: Vec<String> = self
            .registry
            .get_discovered_plugins()
            .keys()
            .cloned()
            .collect();

        for plugin_name in plugin_names {
            if let Err(e) = self.load_plugin(&plugin_name) {
                error!("Failed to load plugin '{}': {}", plugin_name, e);
                // Mark as failed in registry
                self.registry
                    .update_plugin_load_error(&plugin_name, e.to_string());
            }
        }

        Ok(())
    }

    /// Get plugin registry information
    #[allow(dead_code)]
    pub fn get_registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Get loaded plugin names
    #[allow(dead_code)]
    pub fn get_loaded_plugins(&self) -> Vec<String> {
        self.loaded_plugins.keys().cloned().collect()
    }

    /// Test-only method to access engine
    #[allow(dead_code)]
    pub fn test_get_engine(&self) -> &Engine {
        &self.engine
    }

    /// Test-only method to access security config
    #[allow(dead_code)]
    pub fn test_get_security_config(&mut self) -> &mut PluginSecurityConfig {
        &mut self.security_config
    }

    /// Test-only method to validate WASM module
    #[allow(dead_code)]
    pub fn test_validate_wasm_module(
        &self,
        module: &Module,
        plugin_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.validate_wasm_module(module, plugin_name)
    }

    /// Check if a plugin instance has a specific capability
    fn has_capability(
        &self,
        instance: &Instance,
        store: &mut Store<()>,
        function_name: &str,
    ) -> bool {
        instance
            .get_typed_func::<(), (i32, i32)>(store, function_name)
            .is_ok()
    }

    /// Helper method to call a plugin function that returns JSON definitions
    fn call_plugin_json_function(
        &self,
        plugin_name: &str,
        module: &Module,
        function_name: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(self.security_config.fuel_limit)?;

        // Set up timeout protection
        let _epoch_counter = Arc::clone(&self.epoch_counter);
        let engine = self.engine.clone();
        let timeout_secs = self.security_config.execution_timeout_secs;
        let timeout_handle = std::thread::spawn(move || {
            for _ in 0..timeout_secs {
                std::thread::sleep(Duration::from_secs(1));
                engine.increment_epoch();
            }
        });

        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Call the function with timeout protection
        let result = self.call_plugin_function_with_timeout(
            &instance,
            &mut store,
            function_name,
            plugin_name,
        );

        // Clean up timeout thread
        drop(timeout_handle);

        result
    }

    /// Call a plugin function with timeout protection
    fn call_plugin_function_with_timeout(
        &self,
        instance: &Instance,
        store: &mut Store<()>,
        function_name: &str,
        _plugin_name: &str,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        // Check capability
        if !self.has_capability(instance, store, function_name) {
            return Ok(Vec::new()); // Return empty vec if capability not available
        }

        // Get the function and call it
        let func = instance
            .get_func(&mut *store, function_name)
            .ok_or_else(|| format!("Failed to get {} function", function_name))?;
        let func_typed = func.typed::<(), i32>(&mut *store)?;

        // Call the WASM function
        let result_ptr = func_typed.call(&mut *store, ())?;

        // Read the result string from WASM memory
        let json_str = self.read_string(&mut *store, instance, result_ptr)?;

        // Parse the JSON result
        let definitions: Vec<serde_json::Value> = serde_json::from_str(&json_str)?;
        Ok(definitions)
    }

    /// Registers tasks provided by loaded plugins
    pub fn register_plugin_tasks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (plugin_name, module) in &self.loaded_plugins {
            // Call the helper to get task definitions
            let task_defs = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_TASK_DEFINITIONS,
            )?;

            // Register each task
            for task_def in task_defs {
                if let (Some(name), Some(task_type)) = (
                    task_def.get("name").and_then(|v| v.as_str()),
                    task_def.get("type").and_then(|v| v.as_str()),
                ) {
                    match task_type {
                        "apply" => {
                            // Register the task in the task registry
                            // For now, just print that we found it
                            info!("Plugin {} provides apply task: {}", plugin_name, name);
                        }
                        "facts" => {
                            info!("Plugin {} provides facts task: {}", plugin_name, name);
                        }
                        "logs" => {
                            info!("Plugin {} provides logs task: {}", plugin_name, name);
                        }
                        _ => {
                            error!(
                                "Unknown task type '{}' from plugin {}",
                                task_type, plugin_name
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Registers facts collectors provided by loaded plugins
    pub fn register_plugin_facts_collectors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (plugin_name, module) in &self.loaded_plugins {
            // Call the helper to get facts collector definitions
            let collector_defs = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_FACTS_COLLECTORS,
            )?;

            // Register each facts collector
            for collector_def in collector_defs {
                if let Some(name) = collector_def.get("name").and_then(|v| v.as_str()) {
                    let description = collector_def
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Plugin-provided facts collector");
                    let category = collector_def
                        .get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Plugin");

                    info!("Plugin {} provides facts collector: {}", plugin_name, name);

                    // Create a closure that will execute the plugin collector
                    let plugin_name_clone = plugin_name.clone();
                    let name_clone = name.to_string();

                    let collector_fn = Arc::new(move |collector: &crate::facts::Collector| {
                        if let crate::facts::Collector::Plugin(plugin_collector) = collector {
                            // Get the global plugin manager
                            if let Some(pm_arc) = &*GLOBAL_PLUGIN_MANAGER.read().unwrap() {
                                let pm = pm_arc.read().unwrap();
                                // Convert YAML config to JSON
                                let config_json = serde_json::to_value(&plugin_collector.config)
                                    .unwrap_or(serde_json::Value::Null);
                                match pm.execute_facts_collector(
                                    &plugin_name_clone,
                                    &name_clone,
                                    &config_json,
                                ) {
                                    Ok(result) => {
                                        // Convert JSON to YAML
                                        let yaml_str = serde_yaml::to_string(&result)?;
                                        serde_yaml::from_str(&yaml_str).map_err(|e| {
                                            anyhow::anyhow!("YAML conversion failed: {}", e)
                                        })
                                    }
                                    Err(e) => Err(anyhow::anyhow!(
                                        "Plugin collector execution failed: {}",
                                        e
                                    )),
                                }
                            } else {
                                Err(anyhow::anyhow!("Global plugin manager not initialized"))
                            }
                        } else {
                            Err(anyhow::anyhow!("Invalid collector type for plugin facts"))
                        }
                    });

                    // Register the collector in the facts registry
                    crate::facts::FactsRegistry::register_collector(
                        &format!("plugin_{}_{}", plugin_name, name),
                        category,
                        description,
                        &format!("plugin_{}", plugin_name),
                        collector_fn,
                    );

                    info!(
                        "Registered facts collector '{}' from plugin {}",
                        name, plugin_name
                    );
                }
            }
        }
        Ok(())
    }

    /// Registers logs components provided by loaded plugins
    pub fn register_plugin_logs_components(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for (plugin_name, module) in &self.loaded_plugins {
            // Register log sources
            let source_defs = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_LOG_SOURCES,
            )?;
            for source_def in source_defs {
                if let Some(name) = source_def.get("name").and_then(|v| v.as_str()) {
                    info!("Plugin {} provides log source: {}", plugin_name, name);
                    // Log sources are used by specifying source_type: "plugin" with plugin_name and plugin_source_name
                    info!("Log source '{}' from plugin {} can be used by setting source_type: 'plugin', plugin_name: '{}', plugin_source_name: '{}' in config", name, plugin_name, plugin_name, name);
                }
            }

            // Register log parsers
            let parser_defs = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_LOG_PARSERS,
            )?;
            for parser_def in parser_defs {
                if let Some(name) = parser_def.get("name").and_then(|v| v.as_str()) {
                    info!("Plugin {} provides log parser: {}", plugin_name, name);
                    // Parsers are used by specifying parser_type: { type: "plugin", name: "..." } in config
                    info!("Log parser '{}' from plugin {} can be used by setting parser_type: {{ type: 'plugin', name: '{}' }} in config", name, plugin_name, name);
                }
            }

            // Register log filters
            let filter_defs = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_LOG_FILTERS,
            )?;
            for filter_def in filter_defs {
                if let Some(name) = filter_def.get("name").and_then(|v| v.as_str()) {
                    info!("Plugin {} provides log filter: {}", plugin_name, name);
                    // Filters are used in the filters array in config
                    info!(
                        "Log filter '{}' from plugin {} can be used in the filters array in config",
                        name, plugin_name
                    );
                }
            }

            // Register log outputs
            let output_defs = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_LOG_OUTPUTS,
            )?;
            for output_def in output_defs {
                if let Some(name) = output_def.get("name").and_then(|v| v.as_str()) {
                    info!("Plugin {} provides log output: {}", plugin_name, name);
                    // Outputs are used in the outputs array in config
                    info!(
                        "Log output '{}' from plugin {} can be used in the outputs array in config",
                        name, plugin_name
                    );
                }
            }
        }
        Ok(())
    }

    /// Registers template extensions provided by loaded plugins
    pub fn register_plugin_template_extensions(
        &mut self,
        _plugin_manager: Arc<RwLock<Self>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (plugin_name, module) in &self.loaded_plugins {
            // Check if plugin provides template extensions
            if let Ok(extension_defs) = self.call_plugin_json_function(
                plugin_name,
                module,
                crate::plugin_interface::plugin_exports::GET_TEMPLATE_EXTENSIONS,
            ) {
                for extension_def in extension_defs {
                    if let (Some(name), Some(ext_type)) = (
                        extension_def.get("name").and_then(|v| v.as_str()),
                        extension_def.get("type").and_then(|v| v.as_str()),
                    ) {
                        let description = extension_def
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Plugin-provided template extension");
                        let category = extension_def
                            .get("category")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Plugin");

                        let arguments = extension_def
                            .get("arguments")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|arg| {
                                        if let Some(name) = arg.get("name").and_then(|v| v.as_str())
                                        {
                                            let desc = arg
                                                .get("description")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            Some((name.to_string(), desc.to_string()))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();

                        info!(
                            "Plugin {} provides template extension: {} ({})",
                            plugin_name, name, ext_type
                        );

                        let plugin_name_clone = plugin_name.clone();
                        let name_clone = name.to_string();

                        match ext_type {
                            "filter" => {
                                let filter_fn = Arc::new(
                                    move |value: minijinja::Value, args: &[minijinja::Value]| {
                                        // Get the global plugin manager
                                        if let Some(pm_arc) =
                                            &*GLOBAL_PLUGIN_MANAGER.read().unwrap()
                                        {
                                            let pm = pm_arc.read().unwrap();
                                            match pm.execute_template_filter(
                                                &plugin_name_clone,
                                                &name_clone,
                                                &serde_json::Value::Null,
                                                &value,
                                                args,
                                            ) {
                                                Ok(result) => result,
                                                Err(e) => {
                                                    eprintln!("Plugin template filter execution failed: {}", e);
                                                    minijinja::Value::from("ERROR")
                                                }
                                            }
                                        } else {
                                            eprintln!("Global plugin manager not initialized for template filter");
                                            minijinja::Value::from("ERROR")
                                        }
                                    },
                                );

                                crate::apply::templating::TemplateRegistry::register_custom_filter(
                                    name,
                                    description,
                                    category,
                                    arguments,
                                    filter_fn,
                                );
                            }
                            "function" => {
                                let function_fn = Arc::new(move |args: &[minijinja::Value]| {
                                    // Get the global plugin manager
                                    if let Some(pm_arc) = &*GLOBAL_PLUGIN_MANAGER.read().unwrap() {
                                        let pm = pm_arc.read().unwrap();
                                        match pm.execute_template_function(
                                            &plugin_name_clone,
                                            &name_clone,
                                            &serde_json::Value::Null,
                                            args,
                                        ) {
                                            Ok(result) => result,
                                            Err(e) => {
                                                eprintln!(
                                                    "Plugin template function execution failed: {}",
                                                    e
                                                );
                                                minijinja::Value::from("ERROR")
                                            }
                                        }
                                    } else {
                                        eprintln!("Global plugin manager not initialized for template function");
                                        minijinja::Value::from("ERROR")
                                    }
                                });

                                crate::apply::templating::TemplateRegistry::register_custom_function(
                                    name,
                                    description,
                                    category,
                                    arguments,
                                    function_fn,
                                );
                            }
                            _ => {
                                warn!("Unknown template extension type: {}", ext_type);
                            }
                        }

                        info!(
                            "Registered template extension '{}' ({}) from plugin {}",
                            name, ext_type, plugin_name
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Execute a plugin-provided log source
    #[allow(dead_code)]
    pub fn execute_log_source(
        &self,
        plugin_name: &str,
        source_name: &str,
        config: &serde_json::Value,
    ) -> Result<Vec<crate::logs::LogEntry>, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_log_source function from the plugin
        let execute_source = instance
            .get_func(&mut store, "execute_log_source")
            .ok_or("Plugin does not export execute_log_source function")?;
        let execute_source_typed = execute_source.typed::<(i32, i32), i32>(&store)?;

        // Serialize config to JSON string
        let config_json = serde_json::to_string(config)?;
        let source_name_str = source_name.to_string();

        // Allocate memory in the WASM instance for the strings
        let config_ptr = self.allocate_string(&mut store, &instance, &config_json)?;
        let source_name_ptr = self.allocate_string(&mut store, &instance, &source_name_str)?;

        // Call the WASM function
        let result_ptr = execute_source_typed.call(&mut store, (source_name_ptr, config_ptr))?;

        // Read the result string from WASM memory
        let result_json = self.read_string(&mut store, &instance, result_ptr)?;

        // Parse the JSON result into a vector of LogEntry
        let log_entries: Vec<crate::logs::LogEntry> = serde_json::from_str(&result_json)?;

        Ok(log_entries)
    }

    /// Execute a plugin-provided log parser
    #[allow(dead_code)]
    pub fn execute_log_parser(
        &self,
        plugin_name: &str,
        parser_name: &str,
        config: &serde_json::Value,
        input: &str,
    ) -> Result<crate::logs::LogEntry, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_log_parser function from the plugin
        let execute_log_parser = instance
            .get_func(
                &mut store,
                crate::plugin_interface::plugin_exports::EXECUTE_LOG_PARSER,
            )
            .ok_or("Plugin does not export execute_log_parser function")?;
        let execute_log_parser_typed = execute_log_parser.typed::<(i32, i32, i32), i32>(&store)?;

        // Serialize config to JSON string
        let config_json = serde_json::to_string(config)?;
        let parser_name_str = parser_name.to_string();

        // Allocate memory in the WASM instance for the strings
        let config_ptr = self.allocate_string(&mut store, &instance, &config_json)?;
        let parser_name_ptr = self.allocate_string(&mut store, &instance, &parser_name_str)?;
        let input_ptr = self.allocate_string(&mut store, &instance, input)?;

        // Call the WASM function
        let result_ptr =
            execute_log_parser_typed.call(&mut store, (parser_name_ptr, config_ptr, input_ptr))?;

        // Read the result string from WASM memory
        let result_json = self.read_string(&mut store, &instance, result_ptr)?;

        // Parse the JSON result into a LogEntry
        let log_entry: crate::logs::LogEntry = serde_json::from_str(&result_json)?;

        Ok(log_entry)
    }

    /// Execute a plugin-provided log output
    #[allow(dead_code)]
    pub fn execute_log_output(
        &self,
        plugin_name: &str,
        output_name: &str,
        config: &serde_json::Value,
        entry: &crate::logs::LogEntry,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_log_output function from the plugin
        let execute_output = instance
            .get_func(&mut store, "execute_log_output")
            .ok_or("Plugin does not export execute_log_output function")?;
        let execute_output_typed = execute_output.typed::<(i32, i32, i32), i32>(&store)?;

        // Serialize config and entry to JSON strings
        let config_json = serde_json::to_string(config)?;
        let output_name_str = output_name.to_string();
        let entry_json = serde_json::to_string(entry)?;

        // Allocate memory in the WASM instance for the strings
        let config_ptr = self.allocate_string(&mut store, &instance, &config_json)?;
        let output_name_ptr = self.allocate_string(&mut store, &instance, &output_name_str)?;
        let entry_ptr = self.allocate_string(&mut store, &instance, &entry_json)?;

        // Call the WASM function
        let result =
            execute_output_typed.call(&mut store, (output_name_ptr, config_ptr, entry_ptr))?;

        // Check result (0 = success, non-zero = error)
        if result != 0 {
            return Err(format!(
                "Log output '{}' from plugin '{}' execution failed (returned {})",
                output_name, plugin_name, result
            )
            .into());
        }

        Ok(())
    }

    /// Execute a plugin-provided log filter
    #[allow(dead_code)]
    pub fn execute_log_filter(
        &self,
        plugin_name: &str,
        filter_name: &str,
        config: &serde_json::Value,
        entry: &crate::logs::LogEntry,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_log_filter function from the plugin
        let execute_log_filter = instance
            .get_func(
                &mut store,
                crate::plugin_interface::plugin_exports::EXECUTE_LOG_FILTER,
            )
            .ok_or("Plugin does not export execute_log_filter function")?;
        let execute_log_filter_typed = execute_log_filter.typed::<(i32, i32, i32), i32>(&store)?;

        // Serialize config and entry to JSON strings
        let config_json = serde_json::to_string(config)?;
        let filter_name_str = filter_name.to_string();
        let entry_json = serde_json::to_string(entry)?;

        // Allocate memory in the WASM instance for the strings
        let config_ptr = self.allocate_string(&mut store, &instance, &config_json)?;
        let filter_name_ptr = self.allocate_string(&mut store, &instance, &filter_name_str)?;
        let entry_ptr = self.allocate_string(&mut store, &instance, &entry_json)?;

        // Call the WASM function
        let result =
            execute_log_filter_typed.call(&mut store, (filter_name_ptr, config_ptr, entry_ptr))?;

        // Convert result to boolean (0 = false, non-zero = true)
        Ok(result != 0)
    }

    /// Execute a plugin-provided apply task
    #[allow(dead_code)]
    pub fn execute_apply_task(
        &self,
        plugin_name: &str,
        task_name: &str,
        config: &serde_json::Value,
    ) -> Result<serde_yaml::Value, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_task function from the plugin
        let execute_task = instance
            .get_func(&mut store, "execute_task")
            .ok_or("Plugin does not export execute_task function")?;
        let execute_task_typed = execute_task.typed::<(i32, i32), i32>(&store)?;

        // Serialize config to JSON string
        let config_json = serde_json::to_string(config)?;
        let task_name_str = task_name.to_string();

        // Allocate memory in the WASM instance for the strings
        let config_ptr = self.allocate_string(&mut store, &instance, &config_json)?;
        let task_name_ptr = self.allocate_string(&mut store, &instance, &task_name_str)?;

        // Call the WASM function
        let result_ptr = execute_task_typed.call(&mut store, (task_name_ptr, config_ptr))?;

        // Read the result string from WASM memory
        let result_json = self.read_string(&mut store, &instance, result_ptr)?;

        // Parse the JSON result
        let json_value: serde_json::Value = serde_json::from_str(&result_json)?;

        // Convert to YAML value
        let yaml_str = serde_json::to_string(&json_value)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_str)?;

        Ok(yaml_value)
    }

    /// Execute a plugin-provided facts collector
    #[allow(dead_code)]
    pub fn execute_facts_collector(
        &self,
        plugin_name: &str,
        collector_name: &str,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security (aligned with plugin loading)
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_facts_collector function from the plugin
        let execute_facts_collector = instance
            .get_func(
                &mut store,
                crate::plugin_interface::plugin_exports::EXECUTE_FACTS_COLLECTOR,
            )
            .ok_or("Plugin does not export execute_facts_collector function")?;
        let execute_facts_collector_typed =
            execute_facts_collector.typed::<(i32, i32), i32>(&store)?;

        // Serialize config to JSON string
        let config_json = serde_json::to_string(config)?;
        let collector_name_str = collector_name.to_string();

        // Allocate memory in the WASM instance for the strings
        let config_ptr = self.allocate_string(&mut store, &instance, &config_json)?;
        let collector_name_ptr =
            self.allocate_string(&mut store, &instance, &collector_name_str)?;

        // Call the WASM function
        let result_ptr =
            execute_facts_collector_typed.call(&mut store, (collector_name_ptr, config_ptr))?;

        // Read the result string from WASM memory
        let result_json = self.read_string(&mut store, &instance, result_ptr)?;

        // Parse the JSON result
        let json_value: serde_json::Value = serde_json::from_str(&result_json)?;

        Ok(json_value)
    }

    /// Execute a plugin-provided template filter
    #[allow(dead_code)]
    pub fn execute_template_filter(
        &self,
        plugin_name: &str,
        filter_name: &str,
        _config: &serde_json::Value,
        value: &minijinja::Value,
        args: &[minijinja::Value],
    ) -> Result<minijinja::Value, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security (consistent with other plugin execution)
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_template_filter function
        let execute_template_filter = instance
            .get_func(
                &mut store,
                crate::plugin_interface::plugin_exports::EXECUTE_TEMPLATE_FILTER,
            )
            .ok_or("Plugin does not export execute_template_filter function")?;
        let execute_template_filter_typed =
            execute_template_filter.typed::<(i32, i32, i32), i32>(&store)?;

        // Serialize input value to JSON
        let input_json = serde_json::to_string(value)?;

        // Serialize args to JSON array
        let args_json = serde_json::to_string(args)?;

        // Allocate memory in the WASM instance for the strings
        let name_ptr = self.allocate_string(&mut store, &instance, filter_name)?;
        let input_ptr = self.allocate_string(&mut store, &instance, &input_json)?;
        let args_ptr = self.allocate_string(&mut store, &instance, &args_json)?;

        // Call the WASM function
        let result_ptr =
            execute_template_filter_typed.call(&mut store, (name_ptr, input_ptr, args_ptr))?;

        // Read the result string from WASM memory
        let result_json = self.read_string(&mut store, &instance, result_ptr)?;

        // Parse the JSON result into a minijinja Value
        let json_value: serde_json::Value = serde_json::from_str(&result_json)?;
        let minijinja_value = minijinja::Value::from_serialize(&json_value);

        Ok(minijinja_value)
    }

    /// Execute a plugin-provided template function
    #[allow(dead_code)]
    pub fn execute_template_function(
        &self,
        plugin_name: &str,
        function_name: &str,
        _config: &serde_json::Value,
        args: &[minijinja::Value],
    ) -> Result<minijinja::Value, Box<dyn std::error::Error>> {
        let module = self
            .loaded_plugins
            .get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // Create a new store and instantiate the module
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security (consistent with other plugin executions)
        let linker = Linker::new(&self.engine);
        let instance = linker.instantiate(&mut store, module)?;

        // Get the execute_template_function function
        let execute_template_function = instance
            .get_func(
                &mut store,
                crate::plugin_interface::plugin_exports::EXECUTE_TEMPLATE_FUNCTION,
            )
            .ok_or("Plugin does not export execute_template_function function")?;
        let execute_template_function_typed =
            execute_template_function.typed::<(i32, i32), i32>(&store)?;

        // Serialize args to JSON array
        let args_json = serde_json::to_string(args)?;

        // Allocate memory in the WASM instance for the strings
        let name_ptr = self.allocate_string(&mut store, &instance, function_name)?;
        let args_ptr = self.allocate_string(&mut store, &instance, &args_json)?;

        // Call the WASM function
        let result_ptr = execute_template_function_typed.call(&mut store, (name_ptr, args_ptr))?;

        // Read the result string from WASM memory
        let result_json = self.read_string(&mut store, &instance, result_ptr)?;

        // Parse the JSON result into a minijinja Value
        let json_value: serde_json::Value = serde_json::from_str(&result_json)?;
        let minijinja_value = minijinja::Value::from_serialize(&json_value);

        Ok(minijinja_value)
    }

    /// Create a template filter function that calls a plugin
    #[allow(dead_code)]
    pub fn create_plugin_filter(
        plugin_manager: Arc<RwLock<Self>>,
        plugin_name: String,
        filter_name: String,
        config: serde_json::Value,
    ) -> crate::apply::templating::TemplateFilterFn {
        Arc::new(move |value: minijinja::Value, args: &[minijinja::Value]| {
            let manager = match plugin_manager.read() {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to acquire plugin manager lock: {}", e);
                    return value; // Return original value on lock error
                }
            };
            match manager.execute_template_filter(&plugin_name, &filter_name, &config, &value, args)
            {
                Ok(result) => result,
                Err(e) => {
                    error!("Plugin filter execution error: {}", e);
                    value // Return original value on error
                }
            }
        })
    }

    /// Create a template function that calls a plugin
    #[allow(dead_code)]
    pub fn create_plugin_function(
        plugin_manager: Arc<RwLock<Self>>,
        plugin_name: String,
        function_name: String,
        config: serde_json::Value,
    ) -> crate::apply::templating::TemplateFunctionFn {
        Arc::new(move |args: &[minijinja::Value]| {
            let manager = match plugin_manager.read() {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to acquire plugin manager lock: {}", e);
                    return minijinja::Value::from(format!("error: {}", e));
                }
            };
            match manager.execute_template_function(&plugin_name, &function_name, &config, args) {
                Ok(result) => result,
                Err(e) => {
                    error!("Plugin function execution error: {}", e);
                    minijinja::Value::from(format!("error: {}", e))
                }
            }
        })
    }

    /// Get available plugin capabilities for configuration
    pub fn get_available_capabilities(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut capabilities = serde_json::json!({
            "plugins": {},
            "registry": {
                "plugin_dir": self.registry.plugin_dir().to_string_lossy(),
                "scanned": self.registry.is_scanned(),
                "discovered_count": self.registry.get_discovered_plugins().len(),
                "loaded_count": self.loaded_plugins.len()
            }
        });

        // Get capabilities for each loaded plugin
        for (plugin_name, module) in &self.loaded_plugins {
            let mut plugin_caps = serde_json::json!({
                "name": plugin_name,
                "loaded": true,
                "capabilities": {
                    "tasks": false,
                    "facts_collectors": false,
                    "log_sources": false,
                    "log_parsers": false,
                    "log_filters": false,
                    "log_outputs": false,
                    "template_extensions": false
                }
            });

            // Create a new store and instantiate the module for checking functions
            let mut store = Store::new(&self.engine, ());
            store.set_fuel(PLUGIN_FUEL_LIMIT)?; // Set fuel limit for security
            let linker = Linker::new(&self.engine);
            let instance = linker.instantiate(&mut store, module)?;

            // Check each capability
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_TASK_DEFINITIONS,
            ) {
                plugin_caps["capabilities"]["tasks"] = serde_json::json!(true);
            }
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_FACTS_COLLECTORS,
            ) {
                plugin_caps["capabilities"]["facts_collectors"] = serde_json::json!(true);
            }
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_LOG_SOURCES,
            ) {
                plugin_caps["capabilities"]["log_sources"] = serde_json::json!(true);
            }
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_LOG_PARSERS,
            ) {
                plugin_caps["capabilities"]["log_parsers"] = serde_json::json!(true);
            }
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_LOG_FILTERS,
            ) {
                plugin_caps["capabilities"]["log_filters"] = serde_json::json!(true);
            }
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_LOG_OUTPUTS,
            ) {
                plugin_caps["capabilities"]["log_outputs"] = serde_json::json!(true);
            }
            if self.has_capability(
                &instance,
                &mut store,
                crate::plugin_interface::plugin_exports::GET_TEMPLATE_EXTENSIONS,
            ) {
                plugin_caps["capabilities"]["template_extensions"] = serde_json::json!(true);
            }

            capabilities["plugins"][plugin_name] = plugin_caps;
        }

        // Add discovered but not loaded plugins
        for (plugin_name, plugin_info) in self.registry.get_discovered_plugins() {
            if !self.loaded_plugins.contains_key(plugin_name) {
                let plugin_caps = serde_json::json!({
                    "name": plugin_name,
                    "loaded": false,
                    "path": plugin_info.path.to_string_lossy(),
                    "load_error": plugin_info.load_error
                });
                capabilities["plugins"][plugin_name] = plugin_caps;
            }
        }

        Ok(capabilities)
    }

    /// Reload plugins from the registry
    #[allow(dead_code)]
    pub fn reload_plugins(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Reloading plugins...");
        self.loaded_plugins.clear();
        self.load_all_plugins()
    }

    /// Extract plugin metadata with timeout protection
    fn extract_plugin_metadata_with_timeout(
        &self,
        instance: &Instance,
        store: &mut Store<()>,
        _plugin_name: &str,
    ) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
        use std::sync::mpsc;

        // Set epoch deadline for timeout
        let timeout_epoch = self.security_config.execution_timeout_secs;
        store.set_epoch_deadline(timeout_epoch);

        // Create a channel for thread cancellation
        let (cancel_tx, cancel_rx) = mpsc::channel();

        // Start timeout monitoring in a separate thread
        let epoch_counter = Arc::clone(&self.epoch_counter);
        let engine = self.engine.clone();
        let timeout_secs = self.security_config.execution_timeout_secs;
        let timeout_handle = std::thread::spawn(move || {
            for _ in 0..timeout_secs {
                // Check for cancellation signal
                if cancel_rx.try_recv().is_ok() {
                    return;
                }
                std::thread::sleep(Duration::from_secs(1));
                let current_epoch = epoch_counter.load(std::sync::atomic::Ordering::SeqCst);
                if current_epoch >= timeout_epoch {
                    // Trigger epoch interruption
                    engine.increment_epoch();
                }
            }
        });

        // Extract metadata
        let result = self.extract_plugin_metadata(instance, store);

        // Signal the timeout thread to stop
        let _ = cancel_tx.send(());

        // Wait for the timeout thread to finish
        if let Err(e) = timeout_handle.join() {
            error!("Timeout monitoring thread panicked: {:?}", e);
        }

        Ok(result)
    }

    /// Extract metadata from a loaded plugin
    fn extract_plugin_metadata(
        &self,
        instance: &Instance,
        store: &mut Store<()>,
    ) -> (Option<String>, Option<String>) {
        // Try to get metadata from the plugin if it exports a get_metadata function
        match instance.get_typed_func::<(), i32>(&mut *store, "get_plugin_metadata") {
            Ok(metadata_func) => match metadata_func.call(&mut *store, ()) {
                Ok(metadata_ptr) => match self.read_string(&mut *store, instance, metadata_ptr) {
                    Ok(metadata_json) => {
                        match serde_json::from_str::<serde_json::Value>(&metadata_json) {
                            Ok(metadata) => {
                                let version = metadata
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                let description = metadata
                                    .get("description")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                                (version, description)
                            }
                            Err(e) => {
                                error!(
                                    "Failed to parse plugin metadata JSON: {}. Raw JSON: {}",
                                    e, metadata_json
                                );
                                (None, None)
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to read plugin metadata string from WASM memory: {}",
                            e
                        );
                        (None, None)
                    }
                },
                Err(e) => {
                    error!(
                        "Failed to call `get_plugin_metadata` function exported by plugin: {}",
                        e
                    );
                    (None, None)
                }
            },
            Err(e) => {
                error!(
                    "Plugin does not export a usable `get_plugin_metadata` function: {}",
                    e
                );
                (None, None)
            }
        }

        // Fallback: try to extract from WASM custom sections
        // For now, return None - this could be enhanced to parse custom sections
    }

    /// Helper method to allocate a string in WASM memory and return its pointer
    fn allocate_string(
        &self,
        store: &mut Store<()>,
        instance: &Instance,
        s: &str,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or("Plugin does not export memory")?;

        // Check memory bounds first
        let memory_size = memory.data_size(&store);

        // Get the allocate function from the plugin (assuming it exports one)
        let allocate = instance
            .get_func(&mut *store, "allocate")
            .ok_or("Plugin does not export allocate function")?;
        let allocate_typed = allocate.typed::<i32, i32>(&*store)?;

        // Allocate memory for the string (length + 1 for null terminator)
        let len = s.len() as i32 + 1;
        let ptr = allocate_typed.call(&mut *store, len)?;

        let offset = ptr as usize;
        if offset >= memory_size {
            return Err(format!(
                "Pointer {} is outside valid memory bounds (size: {})",
                ptr, memory_size
            )
            .into());
        }

        let mut buffer = s.as_bytes().to_vec();
        buffer.push(0); // null terminator

        // Use safe write method with bounds checking
        memory.write(&mut *store, ptr as usize, &buffer)?;

        Ok(ptr)
    }

    /// Helper method to read a string from WASM memory given a pointer
    fn read_string(
        &self,
        store: &mut Store<()>,
        instance: &Instance,
        ptr: i32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or("Plugin does not export memory")?;

        // Check that the pointer is within valid memory bounds
        let memory_size = memory.data_size(&store);
        if ptr < 0 || ptr as usize >= memory_size {
            return Err("Pointer is outside valid memory bounds".into());
        }

        let offset = ptr as usize;
        if offset >= memory_size {
            return Err("Pointer is outside valid memory bounds".into());
        }

        // Read up to 1MB to find the null terminator, but don't exceed memory bounds
        const MAX_LEN: usize = 1024 * 1024;
        let max_read = MAX_LEN.min(memory_size.saturating_sub(offset));

        // Read the memory data
        let mut data = vec![0u8; max_read];
        memory.read(&store, offset, &mut data)?;

        // Find the null terminator
        let null_pos = data.iter().position(|&b| b == 0).unwrap_or(max_read);

        // Convert to string
        let s = std::str::from_utf8(&data[..null_pos])?;
        Ok(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = std::env::temp_dir();
        let manager = PluginManager::new(temp_dir);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_load_invalid_plugin() {
        let temp_dir = std::env::temp_dir();
        let mut manager = PluginManager::new(temp_dir).unwrap();
        let result = manager.load_plugin("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_valid_plugin() {
        use tempfile::TempDir;

        // Create a minimal valid WASM module
        let wat = r#"
        (module
            (func (export "get_plugin_info") (result i32)
                i32.const 42
            )
        )
        "#;

        let wasm_bytes = wat::parse_str(wat).unwrap();

        // Create a temporary directory for plugins
        let temp_dir = TempDir::new().unwrap();
        let plugin_path = temp_dir.path().join("test_plugin.wasm");

        // Write the WASM module to the temp directory
        std::fs::write(&plugin_path, &wasm_bytes).unwrap();

        let mut manager = PluginManager::new(temp_dir.path().to_path_buf()).unwrap();

        // Scan for plugins first
        manager.scan_plugins().unwrap();

        // Test loading the valid plugin
        let result = manager.load_plugin("test_plugin");
        assert!(
            result.is_ok(),
            "Failed to load valid WASM plugin: {:?}",
            result.err()
        );
    }
}
