//! ロックファイル管理（fpga.lock / asic.lock によるビルド再現性保証）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// fpga.lock — FPGA ビルドの完全再現を保証
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FpgaLock {
    pub version: String,
    pub project: LockProject,
    pub toolchain: HashMap<String, LockedTool>,
    pub container: Option<LockedContainer>,
    pub build_config: LockBuildConfig,
}

/// asic.lock — ASIC ビルドの完全再現を保証
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