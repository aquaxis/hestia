//! fpga.toml 設定モデル（§5.4）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// fpga.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaToml {
    pub project: FpgaProject,
    #[serde(default)]
    pub targets: HashMap<String, FpgaTarget>,
    #[serde(default)]
    pub toolchain: FpgaToolchain,
    #[serde(default)]
    pub ip: HashMap<String, FpgaIp>,
    #[serde(default)]
    pub build: FpgaBuild,
    #[serde(default)]
    pub sim: FpgaSim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaProject {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub hdl_files: Vec<String>,
    #[serde(default)]
    pub include_dirs: Vec<String>,
    #[serde(default)]
    pub testbenches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaTarget {
    pub vendor: String,
    pub device: String,
    pub top: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub interface_script: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FpgaToolchain {
    #[serde(default)]
    pub vivado: Option<String>,
    #[serde(default)]
    pub quartus: Option<String>,
    #[serde(default)]
    pub efinity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaIp {
    pub vendor: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FpgaBuild {
    #[serde(default = "default_parallel_jobs")]
    pub parallel_jobs: u32,
    #[serde(default)]
    pub incremental_compile: bool,
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FpgaSim {
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub top_tb: Option<String>,
    #[serde(default)]
    pub plusargs: Vec<String>,
}

fn default_parallel_jobs() -> u32 { 4 }
fn default_cache_dir() -> String { ".fpga-cache".to_string() }