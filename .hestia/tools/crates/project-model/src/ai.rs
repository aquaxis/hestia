//! ai-conductor 設定モデル（container.toml / upgrade.toml）

use serde::{Deserialize, Serialize};

/// container.toml（§3.8）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerToml {
    pub container: ContainerSection,
    #[serde(default)]
    pub tools: std::collections::HashMap<String, ToolEntry>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub volumes: Vec<VolumeEntry>,
    #[serde(default)]
    pub health: HealthCheck,
    #[serde(default)]
    pub update: ContainerUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSection {
    pub name: String,
    pub base_image: String,
    pub conductor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub install_script: Option<String>,
    #[serde(default)]
    pub version_cmd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeEntry {
    pub host: String,
    pub container: String,
    #[serde(default)]
    pub options: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    #[serde(default)]
    pub cmd: Option<String>,
    #[serde(default = "default_health_interval")]
    pub interval_secs: u64,
    #[serde(default = "default_health_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_health_retries")]
    pub max_retries: u32,
    #[serde(default = "default_true")]
    pub escalate_on_fail: bool,
    #[serde(default = "default_true")]
    pub restart_on_fail: bool,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            cmd: None,
            interval_secs: default_health_interval(),
            timeout_secs: default_health_timeout(),
            max_retries: default_health_retries(),
            escalate_on_fail: true,
            restart_on_fail: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerUpdate {
    #[serde(default)]
    pub auto: Option<bool>,
    #[serde(default)]
    pub schedule: Option<String>,
    #[serde(default)]
    pub rollback_on_failure: Option<bool>,
}

/// upgrade.toml（§3.9）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeToml {
    pub upgrade: UpgradeSection,
    #[serde(default)]
    pub strategy: UpgradeStrategy,
    #[serde(default)]
    pub rollback: RollbackConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeSection {
    #[serde(default)]
    pub check_interval_hours: Option<u64>,
    #[serde(default)]
    pub auto_upgrade: Option<bool>,
    #[serde(default)]
    pub notification_email: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpgradeStrategy {
    #[serde(default)]
    pub major: Option<StrategyEntry>,
    #[serde(default)]
    pub minor: Option<StrategyEntry>,
    #[serde(default)]
    pub patch: Option<StrategyEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEntry {
    #[serde(rename = "type")]
    pub strategy_type: String, // "canary" | "staging" | "production"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackConfig {
    #[serde(default = "default_true")]
    pub auto: bool,
    #[serde(default = "default_rollback_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_rollback_retries")]
    pub max_retries: u32,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            auto: true,
            timeout_secs: default_rollback_timeout(),
            max_retries: default_rollback_retries(),
        }
    }
}

fn default_health_interval() -> u64 { 60 }
fn default_health_timeout() -> u64 { 3 }
fn default_health_retries() -> u32 { 3 }
fn default_true() -> bool { true }
fn default_rollback_timeout() -> u64 { 300 }
fn default_rollback_retries() -> u32 { 3 }