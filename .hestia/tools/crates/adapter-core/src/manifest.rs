//! AdapterManifest — Adapter self-description

use serde::{Deserialize, Serialize};

/// Adapter configuration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterManifest {
    /// Identifier (e.g. "com.xilinx.vivado")
    pub id: String,
    /// Display name (e.g. "AMD Vivado")
    pub name: String,
    /// Adapter version
    pub version: String,
    /// Vendor name
    pub vendor: String,
    /// Version for ABI compatibility check
    pub api_version: u32,
    /// Supported devices (glob patterns)
    #[serde(default)]
    pub supported_devices: Vec<String>,
    /// Release notes URL (used by WatcherAgent)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_notes_url: Option<String>,
}