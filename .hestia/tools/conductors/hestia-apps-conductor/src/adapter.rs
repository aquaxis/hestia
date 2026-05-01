//! Apps Tool Adapter trait — domain-specific extension of adapter_core::ToolAdapter

use async_trait::async_trait;
use adapter_core::{ToolAdapter, capability::CapabilitySet};
use crate::target::TargetConfig;
use crate::fsm_states::AppsBuildState;

/// Apps ドメイン ToolAdapter トレイト
#[async_trait]
pub trait AppsToolAdapter: ToolAdapter {
    /// Target architecture (e.g., ARM Cortex-M, RISC-V)
    fn target_arch(&self) -> &str;

    /// Supported application languages
    fn supported_languages(&self) -> Vec<String>;

    /// Adapter-specific capability flags
    fn capabilities(&self) -> &CapabilitySet;

    /// Build the firmware/application
    async fn build(&self, ctx: &AppsBuildContext) -> Result<AppsStepResult, adapter_core::error::AdapterError>;

    /// Flash firmware to target device
    async fn flash(&self, ctx: &AppsBuildContext) -> Result<AppsStepResult, adapter_core::error::AdapterError>;

    /// Run on-target tests
    async fn test(&self, ctx: &AppsBuildContext) -> Result<AppsStepResult, adapter_core::error::AdapterError>;

    /// Generate size report (flash, RAM, static usage)
    async fn size_report(&self, ctx: &AppsBuildContext) -> Result<AppsSizeReport, adapter_core::error::AdapterError>;
}

/// Apps ビルドコンテキスト
#[derive(Debug, Clone)]
pub struct AppsBuildContext {
    pub project_dir: std::path::PathBuf,
    pub project_name: String,
    pub target: TargetConfig,
    pub language: String,
    pub job_id: String,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// Apps ステップ実行結果
#[derive(Debug, Clone)]
pub struct AppsStepResult {
    pub success: bool,
    pub output_dir: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<adapter_core::Diagnostic>,
    pub state: AppsBuildState,
}

/// Firmware size report
#[derive(Debug, Clone)]
pub struct AppsSizeReport {
    pub text_bytes: u64,
    pub data_bytes: u64,
    pub bss_bytes: u64,
    pub flash_used: u64,
    pub flash_total: u64,
    pub ram_used: u64,
    pub ram_total: u64,
}