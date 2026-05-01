//! RTL Tool Adapter trait — domain-specific extension of adapter_core::ToolAdapter

use async_trait::async_trait;
use adapter_core::{ToolAdapter, capability::CapabilitySet};
use crate::language::HdlLanguage;
use crate::fsm_states::RtlBuildState;

/// RTL ドメイン ToolAdapter トレイト
#[async_trait]
pub trait RtlToolAdapter: ToolAdapter {
    /// サポート対象 HDL 言語一覧
    fn supported_languages(&self) -> Vec<HdlLanguage>;

    /// アダプター固有の capability フラグ
    fn capabilities(&self) -> &CapabilitySet;

    /// Lint 実行
    async fn lint(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;

    /// シミュレーション実行
    async fn simulate(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;

    /// 形式検証実行
    async fn formal_verify(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;

    /// トランスパイル（HDL→HDL 変換）
    async fn transpile(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;
}

/// RTL ビルドコンテキスト
#[derive(Debug, Clone)]
pub struct RtlBuildContext {
    pub project_dir: std::path::PathBuf,
    pub top_module: String,
    pub language: HdlLanguage,
    pub sources: Vec<std::path::PathBuf>,
    pub testbenches: Vec<std::path::PathBuf>,
    pub job_id: String,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// RTL ステップ実行結果
#[derive(Debug, Clone)]
pub struct RtlStepResult {
    pub success: bool,
    pub output_dir: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<adapter_core::Diagnostic>,
    pub state: RtlBuildState,
}