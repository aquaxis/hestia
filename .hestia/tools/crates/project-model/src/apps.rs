//! apps.toml 設定モデル（§9.4）

use serde::{Deserialize, Serialize};

/// apps.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsToml {
    pub project: AppsProject,
    pub toolchain: AppsToolchain,
    #[serde(default)]
    pub rtos: AppsRtos,
    pub memory: AppsMemory,
    #[serde(default)]
    pub hal: AppsHal,
    #[serde(default)]
    pub test: AppsTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsProject {
    pub name: String,
    pub language: String, // "c" | "cpp" | "rust"
    pub target: String,   // triple e.g. "thumbv7em-none-eabihf"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsToolchain {
    pub compiler: String, // "arm-none-eabi-gcc" | "cargo"
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppsRtos {
    #[serde(default)]
    pub kernel: Option<String>, // "freertos" | "zephyr" | "embassy-rs" | "bare-metal"
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsMemory {
    pub flash_origin: u64,
    pub flash_length: String,
    pub ram_origin: u64,
    pub ram_length: String,
    pub linker_script: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppsHal {
    #[serde(default)]
    pub import: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppsTest {
    #[serde(default)]
    pub mode: Option<String>, // "sil" | "hil" | "qemu"
    #[serde(default)]
    pub probe: Option<String>,
}