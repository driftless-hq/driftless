//! Plugin Registry Management
//!
//! This module handles downloading, caching, and managing plugins from remote registries.
//! Plugins are distributed as pre-compiled WASM binaries.

use crate::config::{PluginRegistryConfig, RegistryEntry};
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs as async_fs;
use tokio::time;
use tracing::{info, warn};

/// Plugin metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: Option<String>,
    /// Download URL for the WASM binary
    pub download_url: String,
    /// SHA256 checksum of the WASM binary
    pub checksum: String,
    /// Minimum driftless version required
    pub min_version: Option<String>,
    /// Plugin homepage
    pub homepage: Option<String>,
    /// Plugin author
    pub author: Option<String>,
    /// Plugin license
    pub license: Option<String>,
}

/// Plugin registry client
pub struct PluginRegistryClient {
    config: RegistryEntry,
    client: Client,
    cache_dir: PathBuf,
}

impl PluginRegistryClient {
    /// Create a new registry client
    pub fn new(config: RegistryEntry, cache_dir: PathBuf, timeout_seconds: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .connect_timeout(Duration::from_secs(10)) // DNS/connect timeout
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            cache_dir,
        }
    }

    /// List available plugins from the registry
    pub async fn list_plugins(&self) -> Result<Vec<PluginMetadata>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let url = format!("{}/api/v1/plugins", self.config.url);
        info!("Fetching plugin list from {}", url);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.config.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Registry request failed with status: {}", response.status());
        }

        let plugins: Vec<PluginMetadata> = response.json().await?;

        Ok(plugins)
    }

    /// Get metadata for a specific plugin
    pub async fn get_plugin_metadata(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<PluginMetadata> {
        if !self.config.enabled {
            anyhow::bail!("Registry {} is disabled", self.config.name);
        }

        let version_part = version.map(|v| format!("/{}", v)).unwrap_or_default();
        let url = format!(
            "{}/api/v1/plugins/{}{}",
            self.config.url, name, version_part
        );

        let mut request = self.client.get(&url);

        if let Some(token) = &self.config.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Registry request failed with status: {}", response.status());
        }

        let metadata: PluginMetadata = response.json().await?;

        Ok(metadata)
    }

    /// Download and cache a plugin
    pub async fn download_plugin(&self, metadata: &PluginMetadata) -> Result<PathBuf> {
        let cache_path = self.get_cache_path(metadata);

        // Check if already cached and valid
        if self.is_cached_and_valid(metadata).await? {
            info!(
                "Plugin {} v{} already cached",
                metadata.name, metadata.version
            );
            return Ok(cache_path);
        }

        info!(
            "Downloading plugin {} v{} from {}",
            metadata.name, metadata.version, metadata.download_url
        );

        let mut request = self.client.get(&metadata.download_url);

        if let Some(token) = &self.config.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Download request failed with status: {}", response.status());
        }

        let bytes = response.bytes().await?;

        // Verify checksum
        let actual_checksum = sha256::digest(&bytes[..]);
        if actual_checksum != metadata.checksum {
            anyhow::bail!(
                "Checksum mismatch for plugin {}: expected {}, got {}",
                metadata.name,
                metadata.checksum,
                actual_checksum
            );
        }

        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }

        // Write to cache
        async_fs::write(&cache_path, &bytes).await?;

        info!(
            "Successfully cached plugin {} v{} at {}",
            metadata.name,
            metadata.version,
            cache_path.display()
        );

        Ok(cache_path)
    }

    /// Get the cache path for a plugin
    fn get_cache_path(&self, metadata: &PluginMetadata) -> PathBuf {
        self.cache_dir
            .join(&self.config.name)
            .join(&metadata.name)
            .join(&metadata.version)
            .with_extension("wasm")
    }

    /// Check if a plugin is cached and has valid checksum
    async fn is_cached_and_valid(&self, metadata: &PluginMetadata) -> Result<bool> {
        let cache_path = self.get_cache_path(metadata);

        if !cache_path.exists() {
            return Ok(false);
        }

        // Read cached file
        let bytes = async_fs::read(&cache_path).await?;

        // Verify checksum
        let actual_checksum = sha256::digest(&bytes);
        Ok(actual_checksum == metadata.checksum)
    }
}

/// Plugin lifecycle manager
pub struct PluginLifecycleManager {
    registries: Vec<PluginRegistryClient>,
    local_plugin_dir: PathBuf,
    _cache_dir: PathBuf,
}

impl PluginLifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(registry_config: PluginRegistryConfig, local_plugin_dir: PathBuf) -> Self {
        let cache_dir = registry_config.cache_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".cache")
                .join("driftless")
                .join("plugins")
        });

        let registries = registry_config
            .registries
            .into_iter()
            .filter(|r| r.enabled)
            .map(|config| {
                PluginRegistryClient::new(
                    config,
                    cache_dir.clone(),
                    registry_config.timeout_seconds,
                )
            })
            .collect();

        Self {
            registries,
            local_plugin_dir,
            _cache_dir: cache_dir,
        }
    }

    /// Ensure all configured plugins are available locally
    #[allow(dead_code)]
    pub async fn ensure_plugins_available(&self, required_plugins: &[String]) -> Result<()> {
        info!("Ensuring {} plugins are available", required_plugins.len());

        for plugin_spec in required_plugins {
            self.ensure_plugin_available(plugin_spec).await?;
        }

        Ok(())
    }

    /// Ensure a specific plugin is available locally
    #[allow(dead_code)]
    async fn ensure_plugin_available(&self, plugin_spec: &str) -> Result<()> {
        // Parse plugin spec: "name" or "name@version" or "registry/name@version"
        let (registry_name, name, version) = self.parse_plugin_spec(plugin_spec)?;

        // Check if already installed locally
        if self.is_plugin_installed_locally(&name, version.as_deref())? {
            info!("Plugin {} already installed locally", plugin_spec);
            return Ok(());
        }

        // Find the plugin in registries
        let metadata = self
            .find_plugin_in_registries(&registry_name, &name, version.as_deref())
            .await?;

        // Download and install
        let cached_path = self.download_plugin_to_cache(&metadata).await?;
        self.install_plugin_locally(&metadata, &cached_path).await?;

        Ok(())
    }

    /// List all available plugins from all registries
    pub async fn list_available_plugins(&self) -> Result<HashMap<String, Vec<PluginMetadata>>> {
        let mut all_plugins = HashMap::new();
        let mut any_success = false;

        for registry in &self.registries {
            // Add a timeout to prevent hanging on network issues
            match time::timeout(Duration::from_secs(15), registry.list_plugins()).await {
                Ok(Ok(plugins)) => {
                    all_plugins.insert(registry.config.name.clone(), plugins);
                    any_success = true;
                }
                Ok(Err(e)) => {
                    warn!(
                        "Failed to list plugins from registry {}: {}",
                        registry.config.name, e
                    );
                }
                Err(_) => {
                    warn!(
                        "Timeout listing plugins from registry {}",
                        registry.config.name
                    );
                }
            }
        }

        if !any_success && !self.registries.is_empty() {
            anyhow::bail!("Failed to list plugins from any configured registry");
        }

        Ok(all_plugins)
    }

    /// Install a plugin locally
    pub async fn install_plugin(&self, plugin_spec: &str) -> Result<()> {
        let (registry_name, name, version) = self.parse_plugin_spec(plugin_spec)?;
        let metadata = self
            .find_plugin_in_registries(&registry_name, &name, version.as_deref())
            .await?;
        let cached_path = self.download_plugin_to_cache(&metadata).await?;
        self.install_plugin_locally(&metadata, &cached_path).await?;
        Ok(())
    }

    /// Remove a plugin locally
    pub fn remove_plugin(&self, name: &str, version: Option<&str>) -> Result<()> {
        let plugin_path = self.get_local_plugin_path(name, version);

        if plugin_path.exists() {
            fs::remove_file(&plugin_path)?;
            info!("Removed plugin {} from {}", name, plugin_path.display());
        } else {
            warn!("Plugin {} not found locally", name);
        }

        Ok(())
    }

    /// List locally installed plugins
    pub fn list_installed_plugins(&self) -> Result<Vec<(String, String, PathBuf)>> {
        let mut installed = Vec::new();

        if !self.local_plugin_dir.exists() {
            return Ok(installed);
        }

        for entry in fs::read_dir(&self.local_plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // For now, assume version is embedded or use a default
                    // In a real implementation, you'd extract version from metadata
                    let version = "latest".to_string();
                    installed.push((file_stem.to_string(), version, path));
                }
            }
        }

        Ok(installed)
    }

    /// Validate a locally installed plugin
    pub fn validate_plugin(&self, name: &str, version: Option<&str>) -> Result<()> {
        let plugin_path = self.get_local_plugin_path(name, version);

        if !plugin_path.exists() {
            anyhow::bail!("Plugin {} not found locally", name);
        }

        // Basic WASM validation
        let engine = wasmtime::Engine::new(&wasmtime::Config::new())?;
        wasmtime::Module::from_file(&engine, &plugin_path)?;

        info!("Plugin {} validated successfully", name);
        Ok(())
    }

    /// Parse plugin specification: "name", "name@version", or "registry/name@version"
    fn parse_plugin_spec(&self, spec: &str) -> Result<(Option<String>, String, Option<String>)> {
        let parts: Vec<&str> = spec.split('/').collect();

        match parts.len() {
            1 => {
                // "name" or "name@version"
                let name_version: Vec<&str> = parts[0].split('@').collect();
                match name_version.len() {
                    1 => Ok((None, name_version[0].to_string(), None)),
                    2 => Ok((
                        None,
                        name_version[0].to_string(),
                        Some(name_version[1].to_string()),
                    )),
                    _ => anyhow::bail!("Invalid plugin spec: {}", spec),
                }
            }
            2 => {
                // "registry/name" or "registry/name@version"
                let registry = parts[0].to_string();
                let name_version: Vec<&str> = parts[1].split('@').collect();
                match name_version.len() {
                    1 => Ok((Some(registry), name_version[0].to_string(), None)),
                    2 => Ok((
                        Some(registry),
                        name_version[0].to_string(),
                        Some(name_version[1].to_string()),
                    )),
                    _ => anyhow::bail!("Invalid plugin spec: {}", spec),
                }
            }
            _ => anyhow::bail!("Invalid plugin spec: {}", spec),
        }
    }

    /// Check if a plugin is installed locally
    #[allow(dead_code)]
    fn is_plugin_installed_locally(&self, name: &str, version: Option<&str>) -> Result<bool> {
        let plugin_path = self.get_local_plugin_path(name, version);
        Ok(plugin_path.exists())
    }

    /// Get the local path for a plugin
    fn get_local_plugin_path(&self, name: &str, version: Option<&str>) -> PathBuf {
        let version_suffix = version.map(|v| format!("@{}", v)).unwrap_or_default();
        self.local_plugin_dir
            .join(format!("{}{}.wasm", name, version_suffix))
    }

    /// Find a plugin in the configured registries
    async fn find_plugin_in_registries(
        &self,
        registry_name: &Option<String>,
        name: &str,
        version: Option<&str>,
    ) -> Result<PluginMetadata> {
        // If specific registry requested, search only there
        if let Some(registry_name) = registry_name {
            for registry in &self.registries {
                if registry.config.name == *registry_name {
                    return registry.get_plugin_metadata(name, version).await;
                }
            }
            anyhow::bail!("Registry '{}' not found", registry_name);
        }

        // Search all registries
        for registry in &self.registries {
            match registry.get_plugin_metadata(name, version).await {
                Ok(metadata) => return Ok(metadata),
                Err(_) => continue,
            }
        }

        anyhow::bail!("Plugin '{}' not found in any registry", name);
    }

    /// Download a plugin to cache
    async fn download_plugin_to_cache(&self, metadata: &PluginMetadata) -> Result<PathBuf> {
        // Find the appropriate registry client
        for registry in &self.registries {
            if registry.config.name == metadata.name.split('/').next().unwrap_or("") {
                return registry.download_plugin(metadata).await;
            }
        }

        // Try all registries
        for registry in &self.registries {
            match registry.download_plugin(metadata).await {
                Ok(path) => return Ok(path),
                Err(_) => continue,
            }
        }

        anyhow::bail!(
            "Failed to download plugin {} from any registry",
            metadata.name
        );
    }

    /// Install a plugin locally from cache
    async fn install_plugin_locally(
        &self,
        metadata: &PluginMetadata,
        cache_path: &Path,
    ) -> Result<()> {
        let local_path = self.get_local_plugin_path(&metadata.name, Some(&metadata.version));

        // Ensure local directory exists
        if let Some(parent) = local_path.parent() {
            async_fs::create_dir_all(parent).await?;
        }

        // Copy from cache to local
        async_fs::copy(cache_path, &local_path).await?;

        info!(
            "Installed plugin {} v{} to {}",
            metadata.name,
            metadata.version,
            local_path.display()
        );

        Ok(())
    }
}
