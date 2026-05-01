//! fpga-plugin-registry -- Adapter registration and capability routing

use adapter_core::VendorAdapter;
use std::collections::HashMap;

/// Registry that maps vendor names to their FPGA adapters.
pub struct FpgaPluginRegistry {
    adapters: HashMap<String, Box<dyn VendorAdapter>>,
}

impl FpgaPluginRegistry {
    /// Create a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a vendor adapter.
    pub fn register(&mut self, name: &str, adapter: Box<dyn VendorAdapter>) {
        tracing::info!(vendor = %name, "registering FPGA adapter");
        self.adapters.insert(name.to_string(), adapter);
    }

    /// Look up an adapter by vendor name.
    pub fn get(&self, name: &str) -> Option<&dyn VendorAdapter> {
        self.adapters.get(name).map(|b| b.as_ref())
    }

    /// List all registered vendor names.
    pub fn vendors(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FpgaPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}