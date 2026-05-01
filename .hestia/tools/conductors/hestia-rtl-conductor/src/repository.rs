//! RTL module repository — registry of design modules

use std::collections::HashMap;
use std::path::PathBuf;

/// A registered RTL module
#[derive(Debug, Clone)]
pub struct RtlModule {
    /// Module name (e.g., "uart_tx")
    pub name: String,
    /// HDL language of the module
    pub language: crate::language::HdlLanguage,
    /// Path to the primary source file
    pub path: PathBuf,
    /// List of dependent module names
    pub dependencies: Vec<String>,
}

/// RTL module repository
#[derive(Debug, Default)]
pub struct RtlRepository {
    modules: HashMap<String, RtlModule>,
}

impl RtlRepository {
    /// Create a new empty repository
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a module
    pub fn register(&mut self, module: RtlModule) {
        self.modules.insert(module.name.clone(), module);
    }

    /// Look up a module by name
    pub fn get(&self, name: &str) -> Option<&RtlModule> {
        self.modules.get(name)
    }

    /// List all registered module names
    pub fn module_names(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    /// Remove a module by name
    pub fn remove(&mut self, name: &str) -> Option<RtlModule> {
        self.modules.remove(name)
    }

    /// Return the number of registered modules
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Check if the repository is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}