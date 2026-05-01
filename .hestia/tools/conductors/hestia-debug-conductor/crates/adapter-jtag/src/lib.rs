//! debug-adapter-jtag -- JTAG debug adapter via OpenOCD

use async_trait::async_trait;
use debug_plugin_registry::{AdapterMeta, DebugAdapter, RegistryError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// JTAG-specific errors.
#[derive(Debug, Error)]
pub enum JtagError {
    #[error("OpenOCD process error: {0}")]
    OpenOcdProcess(String),
    #[error("JTAG chain error: {0}")]
    ChainError(String),
    #[error("TAP not found: {0}")]
    TapNotFound(String),
    #[error("connection refused: {0}")]
    ConnectionRefused(String),
}

/// Configuration for the JTAG/OpenOCD adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JtagConfig {
    /// Path to the OpenOCD configuration file.
    pub config_file: String,
    /// OpenOCD TCL port (default 6664).
    #[serde(default)]
    pub tcl_port: u16,
    /// OpenOCD telnet port (default 4444).
    #[serde(default)]
    pub telnet_port: u16,
    /// Additional command-line arguments for OpenOCD.
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for JtagConfig {
    fn default() -> Self {
        Self {
            config_file: String::new(),
            tcl_port: 6664,
            telnet_port: 4444,
            extra_args: Vec::new(),
        }
    }
}

/// JTAG adapter implementation using OpenOCD.
pub struct JtagAdapter {
    meta: AdapterMeta,
    config: JtagConfig,
}

impl JtagAdapter {
    /// Create a new JTAG adapter with default metadata and the given configuration.
    pub fn new(config: JtagConfig) -> Self {
        Self {
            meta: AdapterMeta {
                name: "jtag-openocd".to_string(),
                description: "JTAG debug adapter via OpenOCD".to_string(),
                interface_type: "jtag".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                available: true,
            },
            config,
        }
    }

    /// Return a reference to the current configuration.
    pub fn config(&self) -> &JtagConfig {
        &self.config
    }
}

#[async_trait]
impl DebugAdapter for JtagAdapter {
    fn meta(&self) -> &AdapterMeta {
        &self.meta
    }

    async fn connect(&self, _config: &serde_json::Value) -> Result<(), RegistryError> {
        // In a real implementation this would spawn the OpenOCD process and
        // establish a TCL connection.  For now we just validate.
        tracing::info!(
            "JTAG: connecting via OpenOCD config={}",
            self.config.config_file
        );
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), RegistryError> {
        tracing::info!("JTAG: disconnecting from OpenOCD");
        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.meta.available
    }
}