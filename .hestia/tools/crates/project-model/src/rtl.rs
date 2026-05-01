//! rtl.toml 設定モデル（§4.4）

use serde::{Deserialize, Serialize};

/// rtl.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtlToml {
    pub project: RtlProject,
    pub sources: RtlSources,
    #[serde(default)]
    pub adapters: RtlAdapters,
    #[serde(default)]
    pub handoff: RtlHandoff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtlProject {
    pub name: String,
    pub top: String,
    pub language: String, // "systemverilog" | "vhdl" | "chisel" | "spinalhdl" | "amaranth"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtlSources {
    #[serde(default)]
    pub rtl: Vec<String>,
    #[serde(default)]
    pub testbench: Vec<String>,
    #[serde(default)]
    pub constraints_shared: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RtlAdapters {
    #[serde(default)]
    pub lint: Option<String>,
    #[serde(default)]
    pub simulation: Option<String>,
    #[serde(default)]
    pub formal: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RtlHandoff {
    #[serde(default)]
    pub fpga: Vec<String>,
    #[serde(default)]
    pub asic: Vec<String>,
    #[serde(default)]
    pub hal_bus_decl: Option<String>,
}