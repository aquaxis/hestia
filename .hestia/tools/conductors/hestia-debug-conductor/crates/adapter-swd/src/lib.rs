//! debug-adapter-swd -- SWD debug adapter via pyOCD

use async_trait::async_trait;
use debug_plugin_registry::{AdapterMeta, DebugAdapter, RegistryError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// SWD-specific errors.
#[derive(Debug, Error)]
pub enum SwdError {
    #[error("pyOCD process error: {0}")]
    PyOcdProcess(String),
    #[error("SWD protocol error: {0}")]
    ProtocolError(String),
    #[error("target not responding: {0}")]
    TargetNotResponding(String),
    #[error("flash algorithm error: {0}")]
    FlashAlgoError(String),
}

/// Configuration for the SWD/pyOCD adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwdConfig {
    /// Target MCU identifier (e.g. "stm32f407vg").
    pub target: String,
    /// Frequency in Hz (default 1 MHz).
    #[serde(default)]
    pub frequency_hz: u64,
    /// pyOCD probe serial number (empty = auto-detect).
    #[serde(default)]
    pub probe_serial: String,
}

impl Default for SwdConfig {
    fn default() -> Self {
        Self {
            target: String::new(),
            frequency_hz: 1_000_000,
            probe_serial: String::new(),
        }
    }
}

/// SWD adapter implementation using pyOCD.
pub struct SwdAdapter {
    meta: AdapterMeta,
    config: SwdConfig,
}

impl SwdAdapter {
    /// Create a new SWD adapter with default metadata and the given configuration.
    pub fn new(config: SwdConfig) -> Self {
        Self {
            meta: AdapterMeta {
                name: "swd-pyocd".to_string(),
                description: "SWD debug adapter via pyOCD".to_string(),
                interface_type: "swd".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                available: true,
            },
            config,
        }
    }

    /// Return a reference to the current configuration.
    pub fn config(&self) -> &SwdConfig {
        &self.config
    }
}

#[async_trait]
impl DebugAdapter for SwdAdapter {
    fn meta(&self) -> &AdapterMeta {
        &self.meta
    }

    async fn connect(&self, _config: &serde_json::Value) -> Result<(), RegistryError> {
        tracing::info!("SWD: connecting via pyOCD target={}", self.config.target);
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), RegistryError> {
        tracing::info!("SWD: disconnecting from pyOCD");
        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.meta.available
    }
}