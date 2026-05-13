//! Lock file management (ensuring build reproducibility via fpga.lock / asic.lock)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// fpga.lock — Ensures full reproducibility of FPGA builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaLock {
    pub version: String,
    pub project: LockProject,
    pub toolchain: HashMap<String, LockedTool>,
    pub container: Option<LockedContainer>,
    pub build_config: LockBuildConfig,
}

/// asic.lock — Ensures full reproducibility of ASIC builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsicLock {
    pub version: String,
    pub project: LockProject,
    pub toolchain: HashMap<String, LockedTool>,
    pub container: Option<LockedContainer>,
    pub pdk: LockedPdk,
    pub build_config: LockBuildConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockProject {
    pub name: String,
    pub top: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedTool {
    pub version: String,
    pub path: String,
    #[serde(default)]
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedContainer {
    pub image: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPdk {
    pub name: String,
    pub version: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockBuildConfig {
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    pub created_at: String,
}