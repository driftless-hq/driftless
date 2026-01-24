use std::path::Path;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

/// PluginManager handles loading and instantiating WebAssembly modules securely.
pub struct PluginManager {
    engine: Engine,
}

impl PluginManager {
    /// Creates a new PluginManager with secure default configuration.
    pub fn new() -> Result<Self, wasmtime::Error> {
        let mut config = Config::new();
        // Set resource limits for security
        config.max_wasm_stack(2 * 1024 * 1024); // 2MB stack
                                                // config.memory_max(64 * 1024 * 1024); // 64MB max memory - TODO: find correct method
        config.memory_reservation_for_growth(0); // No growth
        config.consume_fuel(true); // Enable fuel consumption
        config.epoch_interruption(true); // Allow interruption

        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    /// Loads and instantiates a WebAssembly module from the given path.
    /// Returns the instance if successful.
    pub fn load_plugin(&self, path: &Path) -> Result<Instance, wasmtime::Error> {
        let module = Module::from_file(&self.engine, path)?;

        // Create a store with fuel limit (e.g., 1 billion instructions)
        let mut store = Store::new(&self.engine, ());
        store.set_fuel(1_000_000_000)?;

        let linker = Linker::new(&self.engine);
        // No WASI added for security - plugins have no host access

        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)?;
        Ok(instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_load_invalid_plugin() {
        let manager = PluginManager::new().unwrap();
        let result = manager.load_plugin(Path::new("nonexistent.wasm"));
        assert!(result.is_err());
    }

    // TODO: Add test with a valid minimal WASM module
}
