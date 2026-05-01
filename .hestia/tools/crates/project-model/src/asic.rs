//! asic.toml 設定モデル（§6.9）

use serde::{Deserialize, Serialize};

/// asic.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsicToml {
    pub project: AsicProject,
    pub target: AsicTarget,
    #[serde(default)]
    pub synthesis: AsicSynthesis,
    #[serde(default)]
    pub placement: AsicPlacement,
    #[serde(default)]
    pub cts: AsicCts,
    #[serde(default)]
    pub routing: AsicRouting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsicProject {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub rtl_files: Vec<String>,
    pub top: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsicTarget {
    pub pdk: String,
    pub clock_period_ns: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsicSynthesis {
    #[serde(default)]
    pub flatten: bool,
    #[serde(default)]
    pub abc_script: Option<String>,
    #[serde(default = "default_strategy")]
    pub strategy: String, // "area" | "speed" | "balanced"
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsicPlacement {
    #[serde(default = "default_density")]
    pub target_density: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsicCts {
    #[serde(default)]
    pub max_skew_ns: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsicRouting {
    #[serde(default)]
    pub min_layer: Option<u32>,
    #[serde(default)]
    pub max_layer: Option<u32>,
}

fn default_strategy() -> String { "balanced".to_string() }
fn default_density() -> f64 { 0.55 }