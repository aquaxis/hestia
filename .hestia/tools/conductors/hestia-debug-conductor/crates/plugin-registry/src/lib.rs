//! debug-plugin-registry -- Debug adapter registration and discovery

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors from the plugin registry.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("adapter already registered: {0}")]
    AlreadyRegistered(String),
    #[error("adapter not found: {0}")]
    NotFound(String),
    #[error("adapter initialization failed: {0}")]
    InitFailed(String),
}

/// Metadata for a registered debug adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterMeta {
    /// Unique adapter name (e.g. "jtag-openocd", "swd-pyocd").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Interface type: "jtag", "swd", "ila", etc.
    pub interface_type: String,
    /// Version string.
    pub version: String,
    /// Whether the adapter is currently available.
    pub available: bool,
}

/// Trait that every debug adapter must implement.
#[async_trait]
pub trait DebugAdapter: Send + Sync {
    /// Return the adapter metadata.
    fn meta(&self) -> &AdapterMeta;

    /// Connect to the debug target.
    async fn connect(&self, config: &serde_json::Value) -> Result<(), RegistryError>;

    /// Disconnect from the debug target.
    async fn disconnect(&self) -> Result<(), RegistryError>;

    /// Check whether the adapter is healthy and available.
    async fn health_check(&self) -> bool;
}

/// Central registry for debug adapter plugins.
pub struct PluginRegistry {
    adapters: HashMap<String, Box<dyn DebugAdapter>>,
}

impl PluginRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a debug adapter.
    pub fn register(&mut self, adapter: Box<dyn DebugAdapter>) -> Result<(), RegistryError> {
        let name = adapter.meta().name.clone();
        if self.adapters.contains_key(&name) {
            return Err(RegistryError::AlreadyRegistered(name));
        }
        self.adapters.insert(name, adapter);
        Ok(())
    }

    /// Look up a registered adapter by name.
    pub fn get(&self, name: &str) -> Result<&dyn DebugAdapter, RegistryError> {
        self.adapters
            .get(name)
            .map(|a| a.as_ref())
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))
    }

    /// List metadata for all registered adapters.
    pub fn list(&self) -> Vec<&AdapterMeta> {
        self.adapters.values().map(|a| a.meta()).collect()
    }

    /// Remove an adapter by name.
    pub fn unregister(&mut self, name: &str) -> Result<(), RegistryError> {
        self.adapters
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}