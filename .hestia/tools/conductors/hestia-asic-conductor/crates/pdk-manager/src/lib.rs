//! pdk-manager -- PDK management: Sky130, GF180MCU, IHP

pub mod pdk_config;

pub use pdk_config::PdkConfig;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported PDK identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PdkId {
    Sky130A,
    Sky130B,
    Gf180mcuC,
    Gf180mcuD,
    IhpSg13g2,
}

impl std::fmt::Display for PdkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdkId::Sky130A => write!(f, "sky130A"),
            PdkId::Sky130B => write!(f, "sky130B"),
            PdkId::Gf180mcuC => write!(f, "gf180mcuC"),
            PdkId::Gf180mcuD => write!(f, "gf180mcuD"),
            PdkId::IhpSg13g2 => write!(f, "ihp-sg13g2"),
        }
    }
}

/// PDK installation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdkStatus {
    NotInstalled,
    Installed(String),
    Error(String),
}

/// PDK manager handles installation and path resolution for process design kits.
pub struct PdkManager {
    pdk_root: std::path::PathBuf,
    installed: HashMap<PdkId, PdkStatus>,
}

impl PdkManager {
    /// Create a new PDK manager rooted at the given path.
    pub fn new(pdk_root: &std::path::Path) -> Self {
        Self {
            pdk_root: pdk_root.to_path_buf(),
            installed: HashMap::new(),
        }
    }

    /// Get the root path for PDK installations.
    pub fn pdk_root(&self) -> &std::path::Path {
        &self.pdk_root
    }

    /// Resolve the path for a given PDK.
    pub fn pdk_path(&self, pdk: PdkId) -> std::path::PathBuf {
        self.pdk_root.join(pdk.to_string())
    }

    /// Check if a PDK is installed.
    pub fn is_installed(&self, pdk: PdkId) -> bool {
        matches!(self.installed.get(&pdk), Some(PdkStatus::Installed(_)))
    }

    /// Mark a PDK as installed.
    pub fn mark_installed(&mut self, pdk: PdkId, version: String) {
        self.installed.insert(pdk, PdkStatus::Installed(version));
    }

    /// Mark a PDK as having an error.
    pub fn mark_error(&mut self, pdk: PdkId, error: String) {
        self.installed.insert(pdk, PdkStatus::Error(error));
    }

    /// Get the status of a PDK.
    pub fn status(&self, pdk: PdkId) -> &PdkStatus {
        self.installed.get(&pdk).unwrap_or(&PdkStatus::NotInstalled)
    }
}