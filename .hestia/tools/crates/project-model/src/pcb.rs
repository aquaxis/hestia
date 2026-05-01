//! pcb.toml 設定モデル（§7.6）

use serde::{Deserialize, Serialize};

/// pcb.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcbToml {
    pub project: PcbProject,
    pub board: PcbBoard,
    #[serde(default)]
    pub layers: Vec<PcbLayer>,
    #[serde(default)]
    pub design: PcbDesign,
    #[serde(default)]
    pub output: PcbOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcbProject {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub board_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcbBoard {
    pub layer_count: u32,
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcbLayer {
    pub name: String,
    #[serde(rename = "type")]
    pub layer_type: String, // "signal" | "power" | "ground"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PcbDesign {
    #[serde(default)]
    pub input_format: Option<String>,
    #[serde(default)]
    pub ai_enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PcbOutput {
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub output_dir: Option<String>,
}