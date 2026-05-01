//! HAL Tool Adapter trait — domain-specific extension of adapter_core::ToolAdapter

use async_trait::async_trait;
use adapter_core::{ToolAdapter, capability::CapabilitySet};
use crate::bus_protocol::BusProtocol;
use crate::fsm_states::HalBuildState;

/// HAL ドメイン ToolAdapter トレイト
#[async_trait]
pub trait HalToolAdapter: ToolAdapter {
    /// サポート入力フォーマット（例: "systemrdl", "ipxact", "toml"）
    fn supported_inputs(&self) -> Vec<String>;

    /// サポート出力言語（例: "c", "rust", "python", "svd"）
    fn supported_outputs(&self) -> Vec<String>;

    /// アダプター固有の capability フラグ
    fn capabilities(&self) -> &CapabilitySet;

    /// レジスタ定義をパース
    async fn parse(&self, ctx: &HalBuildContext) -> Result<HalStepResult, adapter_core::error::AdapterError>;

    /// パース結果をバリデーション
    async fn validate(&self, ctx: &HalBuildContext) -> Result<HalStepResult, adapter_core::error::AdapterError>;

    /// コード生成
    async fn generate(&self, ctx: &HalBuildContext) -> Result<HalStepResult, adapter_core::error::AdapterError>;
}

/// HAL ビルドコンテキスト
#[derive(Debug, Clone)]
pub struct HalBuildContext {
    pub project_dir: std::path::PathBuf,
    pub project_name: String,
    pub input_format: String,
    pub bus_protocol: BusProtocol,
    pub data_width: u32,
    pub addr_width: u32,
    pub job_id: String,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// HAL ステップ実行結果
#[derive(Debug, Clone)]
pub struct HalStepResult {
    pub success: bool,
    pub output_dir: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<adapter_core::Diagnostic>,
    pub state: HalBuildState,
}