//! hal.toml 設定モデル（§8.4）

use serde::{Deserialize, Serialize};

/// hal.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalToml {
    pub project: HalProject,
    pub sources: HalSources,
    pub bus: HalBus,
    #[serde(default)]
    pub outputs: HalOutputs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalProject {
    pub name: String,
    pub input_format: String, // "systemrdl" | "ipxact" | "toml"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalSources {
    #[serde(default)]
    pub register_definitions: Vec<String>,
    #[serde(default)]
    pub memory_map: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalBus {
    pub protocol: String, // "axi4-lite" | "axi4" | "wishbone-b4" | "ahb-lite"
    pub data_width: u32,
    pub addr_width: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HalOutputs {
    #[serde(default)]
    pub c_header: Option<String>,
    #[serde(default)]
    pub rust_crate: Option<String>,
    #[serde(default)]
    pub python_module: Option<String>,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub svd: Option<String>,
}