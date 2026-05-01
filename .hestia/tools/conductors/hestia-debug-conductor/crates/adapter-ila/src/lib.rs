//! debug-adapter-ila -- On-chip debug: Xilinx ILA, Intel SignalTap, Lattice Reveal

use async_trait::async_trait;
use debug_plugin_registry::{AdapterMeta, DebugAdapter, RegistryError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ILA-specific errors.
#[derive(Debug, Error)]
pub enum IlaError {
    #[error("ILA capture failed: {0}")]
    CaptureFailed(String),
    #[error("trigger configuration error: {0}")]
    TriggerConfigError(String),
    #[error("device not supported: {0}")]
    DeviceNotSupported(String),
}

/// Vendor-specific on-chip debug tool variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IlaVendor {
    /// Xilinx Integrated Logic Analyzer
    Xilinx,
    /// Intel (Altera) SignalTap II
    Intel,
    /// Lattice Reveal
    Lattice,
}

impl std::fmt::Display for IlaVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xilinx => "xilinx",
            Self::Intel => "intel",
            Self::Lattice => "lattice",
        }
        .fmt(f)
    }
}

/// Configuration for the ILA adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlaConfig {
    /// Vendor tool to use.
    pub vendor: IlaVendor,
    /// Device part number (e.g. "xc7z010", "10CL025", "LFE3-150A").
    pub device: String,
    /// Trigger condition (vendor-specific expression).
    #[serde(default)]
    pub trigger: String,
    /// Sample depth.
    #[serde(default)]
    pub sample_depth: u32,
}

impl Default for IlaConfig {
    fn default() -> Self {
        Self {
            vendor: IlaVendor::Xilinx,
            device: String::new(),
            trigger: String::new(),
            sample_depth: 1024,
        }
    }
}

/// ILA adapter implementation.
pub struct IlaAdapter {
    meta: AdapterMeta,
    config: IlaConfig,
}

impl IlaAdapter {
    /// Create a new ILA adapter.
    pub fn new(config: IlaConfig) -> Self {
        let interface_type = match config.vendor {
            IlaVendor::Xilinx => "ila-xilinx",
            IlaVendor::Intel => "ila-intel",
            IlaVendor::Lattice => "ila-lattice",
        };
        Self {
            meta: AdapterMeta {
                name: format!("ila-{}", config.vendor),
                description: format!(
                    "On-chip debug adapter via {}",
                    match config.vendor {
                        IlaVendor::Xilinx => "Xilinx ILA",
                        IlaVendor::Intel => "Intel SignalTap II",
                        IlaVendor::Lattice => "Lattice Reveal",
                    }
                ),
                interface_type: interface_type.to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                available: true,
            },
            config,
        }
    }

    /// Return a reference to the current configuration.
    pub fn config(&self) -> &IlaConfig {
        &self.config
    }

    /// Arm the trigger and begin capturing samples.
    pub async fn arm(&self) -> Result<(), IlaError> {
        tracing::info!("ILA: arming trigger vendor={}", self.config.vendor);
        Ok(())
    }

    /// Upload captured sample data.
    pub async fn upload_capture(&self) -> Result<Vec<u8>, IlaError> {
        tracing::info!("ILA: uploading capture depth={}", self.config.sample_depth);
        Ok(Vec::new())
    }
}

#[async_trait]
impl DebugAdapter for IlaAdapter {
    fn meta(&self) -> &AdapterMeta {
        &self.meta
    }

    async fn connect(&self, _config: &serde_json::Value) -> Result<(), RegistryError> {
        tracing::info!(
            "ILA: connecting vendor={} device={}",
            self.config.vendor,
            self.config.device
        );
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), RegistryError> {
        tracing::info!("ILA: disconnecting");
        Ok(())
    }

    async fn health_check(&self) -> bool {
        self.meta.available
    }
}