//! asic-plugin-registry -- AsicToolAdapter registration and capability routing

use std::collections::HashMap;

/// Trait for ASIC tool adapters.
pub trait AsicToolAdapter: Send + Sync {
    /// Unique identifier for this adapter.
    fn id(&self) -> &str;

    /// Display name.
    fn name(&self) -> &str;

    /// Run the synthesis step.
    fn synthesize(&self, project_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Run the placement step.
    fn place(&self, project_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Run the routing step.
    fn route(&self, project_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>;
}

/// Registry that maps tool names to their ASIC adapters.
pub struct AsicPluginRegistry {
    adapters: HashMap<String, Box<dyn AsicToolAdapter>>,
}

impl AsicPluginRegistry {
    /// Create a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register an ASIC tool adapter.
    pub fn register(&mut self, name: &str, adapter: Box<dyn AsicToolAdapter>) {
        tracing::info!(tool = %name, "registering ASIC adapter");
        self.adapters.insert(name.to_string(), adapter);
    }

    /// Look up an adapter by tool name.
    pub fn get(&self, name: &str) -> Option<&dyn AsicToolAdapter> {
        self.adapters.get(name).map(|b| b.as_ref())
    }

    /// List all registered tool names.
    pub fn tools(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for AsicPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}