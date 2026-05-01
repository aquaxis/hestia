//! pcb-plugin-registry -- Adapter registration for PCB EDA tools

use std::collections::HashMap;

/// Trait for PCB EDA tool adapters.
pub trait PcbToolAdapter: Send + Sync {
    /// Unique identifier for this adapter.
    fn id(&self) -> &str;

    /// Display name.
    fn name(&self) -> &str;

    /// Generate schematic from requirements.
    fn generate_schematic(&self, project_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Run DRC check.
    fn run_drc(&self, project_dir: &std::path::Path) -> Result<DrcResult, Box<dyn std::error::Error>>;

    /// Export fabrication files.
    fn export_fab(&self, project_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>>;
}

/// Result of a DRC check.
#[derive(Debug, Clone)]
pub struct DrcResult {
    pub violations: u32,
    pub warnings: u32,
    pub passed: bool,
}

/// Registry that maps tool names to their PCB adapters.
pub struct PcbPluginRegistry {
    adapters: HashMap<String, Box<dyn PcbToolAdapter>>,
}

impl PcbPluginRegistry {
    /// Create a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a PCB tool adapter.
    pub fn register(&mut self, name: &str, adapter: Box<dyn PcbToolAdapter>) {
        tracing::info!(tool = %name, "registering PCB adapter");
        self.adapters.insert(name.to_string(), adapter);
    }

    /// Look up an adapter by tool name.
    pub fn get(&self, name: &str) -> Option<&dyn PcbToolAdapter> {
        self.adapters.get(name).map(|b| b.as_ref())
    }

    /// List all registered tool names.
    pub fn tools(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PcbPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}