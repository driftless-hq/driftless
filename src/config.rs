//! Configuration management for Driftless
//!
//! This module provides configuration structures and loading functionality
//! for various Driftless components including plugin registries.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSecurityConfig {
    /// Maximum WASM stack size in bytes
    pub max_stack_size: usize,
    /// Maximum memory per plugin instance in bytes
    pub max_memory: usize,
    /// Fuel limit for execution (instruction count)
    pub fuel_limit: u64,
    /// Execution timeout in seconds
    pub execution_timeout_secs: u64,
    /// Whether to allow WASI access (should be false for security)
    pub allow_wasi: bool,
    /// Whether to enable debug features (should be false in production)
    pub debug_enabled: bool,
    /// Maximum number of tables per module
    pub max_tables: u32,
    /// Maximum number of memories per module
    pub max_memories: u32,
    /// Maximum number of globals per module
    pub max_globals: u32,
}

impl Default for PluginSecurityConfig {
    fn default() -> Self {
        Self {
            max_stack_size: 2 * 1024 * 1024, // 2MB
            max_memory: 64 * 1024 * 1024,    // 64MB
            fuel_limit: 1_000_000_000,       // 1 billion instructions
            execution_timeout_secs: 30,      // 30 seconds
            allow_wasi: false,               // No host access by default
            debug_enabled: false,            // No debug features
            max_tables: 1,
            max_memories: 1,
            max_globals: 100,
        }
    }
}

/// Plugin registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryConfig {
    /// List of plugin registries to use
    pub registries: Vec<RegistryEntry>,
    /// Local cache directory for downloaded plugins
    pub cache_dir: Option<PathBuf>,
    /// Whether to enable automatic plugin updates
    pub auto_update: bool,
    /// Timeout for registry requests (in seconds)
    pub timeout_seconds: u64,
    /// Security configuration for plugin execution
    pub security: PluginSecurityConfig,
}

impl Default for PluginRegistryConfig {
    fn default() -> Self {
        Self {
            registries: vec![RegistryEntry {
                name: "driftless-official".to_string(),
                url: "https://registry.driftless.dev".to_string(),
                enabled: true,
                priority: 0,
                token: None,
            }],
            cache_dir: None,
            auto_update: false,
            timeout_seconds: 30,
            security: PluginSecurityConfig::default(),
        }
    }
}

/// A single plugin registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Human-readable name for the registry
    pub name: String,
    /// Base URL of the registry
    pub url: String,
    /// Whether this registry is enabled
    pub enabled: bool,
    /// Priority for plugin resolution (lower numbers = higher priority)
    pub priority: u32,
    /// Optional authentication token
    pub token: Option<String>,
}

/// Load plugin registry configuration from the config directory
pub fn load_plugin_registry_config(
    config_dir: &std::path::Path,
) -> Result<PluginRegistryConfig, Box<dyn std::error::Error>> {
    let yaml_path = config_dir.join("plugins.yml");
    let json_path = config_dir.join("plugins.json");
    let toml_path = config_dir.join("plugins.toml");

    let contents = if yaml_path.exists() {
        std::fs::read_to_string(&yaml_path)?
    } else if json_path.exists() {
        std::fs::read_to_string(&json_path)?
    } else if toml_path.exists() {
        std::fs::read_to_string(&toml_path)?
    } else {
        // Return default configuration if no config file exists
        return Ok(PluginRegistryConfig::default());
    };

    let config: PluginRegistryConfig = if yaml_path.exists() {
        serde_yaml::from_str(&contents)?
    } else if json_path.exists() {
        serde_json::from_str(&contents)?
    } else {
        toml::from_str(&contents)?
    };

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = PluginRegistryConfig::default();
        assert_eq!(config.registries.len(), 1);
        assert_eq!(config.registries[0].name, "driftless-official");
        assert_eq!(config.registries[0].url, "https://registry.driftless.dev");
        assert!(config.registries[0].enabled);
        assert_eq!(config.registries[0].priority, 0);
        assert!(!config.auto_update);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_load_yaml_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("plugins.yml");

        let yaml_content = r#"
registries:
  - name: "test-registry"
    url: "https://test.example.com"
    enabled: true
    priority: 1
    token: "test-token"
  - name: "disabled-registry"
    url: "https://disabled.example.com"
    enabled: false
    priority: 2
cache_dir: "/tmp/plugin-cache"
auto_update: true
timeout_seconds: 60
security:
  max_stack_size: 2097152
  max_memory: 67108864
  fuel_limit: 1000000000
  execution_timeout_secs: 30
  allow_wasi: false
  debug_enabled: false
  max_tables: 1
  max_memories: 1
  max_globals: 100
"#;

        fs::write(&config_path, yaml_content).unwrap();

        let config = load_plugin_registry_config(temp_dir.path()).unwrap();
        assert_eq!(config.registries.len(), 2);
        assert_eq!(config.registries[0].name, "test-registry");
        assert_eq!(config.registries[0].token, Some("test-token".to_string()));
        assert_eq!(config.registries[1].name, "disabled-registry");
        assert!(!config.registries[1].enabled);
        assert_eq!(config.cache_dir, Some(PathBuf::from("/tmp/plugin-cache")));
        assert!(config.auto_update);
        assert_eq!(config.timeout_seconds, 60);
    }

    #[test]
    fn test_load_json_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("plugins.json");

        let json_content = r#"{
  "registries": [
    {
      "name": "json-registry",
      "url": "https://json.example.com",
      "enabled": true,
      "priority": 0
    }
  ],
  "auto_update": false,
  "timeout_seconds": 45,
  "security": {
    "max_stack_size": 2097152,
    "max_memory": 67108864,
    "fuel_limit": 1000000000,
    "execution_timeout_secs": 30,
    "allow_wasi": false,
    "debug_enabled": false,
    "max_tables": 1,
    "max_memories": 1,
    "max_globals": 100
  }
}"#;

        fs::write(&config_path, json_content).unwrap();

        let config = load_plugin_registry_config(temp_dir.path()).unwrap();
        assert_eq!(config.registries.len(), 1);
        assert_eq!(config.registries[0].name, "json-registry");
        assert_eq!(config.timeout_seconds, 45);
    }

    #[test]
    fn test_load_missing_config_returns_default() {
        let temp_dir = tempdir().unwrap();
        let config = load_plugin_registry_config(temp_dir.path()).unwrap();
        // Should return default config when no file exists
        assert_eq!(config.registries.len(), 1);
        assert_eq!(config.registries[0].name, "driftless-official");
    }
}
