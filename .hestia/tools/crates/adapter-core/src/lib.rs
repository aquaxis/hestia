//! Hestia Adapter Core — ToolAdapter / VendorAdapter traits

pub mod capability;
pub mod error;
pub mod manifest;
pub mod script;

use async_trait::async_trait;
use capability::CapabilitySet;
use error::AdapterError;
use manifest::AdapterManifest;
use std::path::PathBuf;

/// 汎用 ToolAdapter トレイト（RTL / HAL / Apps 等のドメインで実装）
#[async_trait]
pub trait ToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> &CapabilitySet;
}

/// FPGA VendorAdapter トレイト（§5.2 統一インターフェース）
#[async_trait]
pub trait VendorAdapter: Send + Sync + 'static {
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> CapabilitySet;

    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;

    async fn timing_analysis(&self, _ctx: &BuildContext) -> Result<TimingReport, AdapterError> {
        Err(AdapterError::Unsupported("timing_analysis".to_string()))
    }
    async fn start_debug_session(&self, _ctx: &BuildContext) -> Result<DebugSession, AdapterError> {
        Err(AdapterError::Unsupported("start_debug_session".to_string()))
    }
    async fn hls_compile(&self, _ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        Err(AdapterError::Unsupported("hls_compile".to_string()))
    }
    async fn program_device(&self, _ctx: &ProgramContext) -> Result<(), AdapterError> {
        Err(AdapterError::Unsupported("program_device".to_string()))
    }
    async fn simulate(&self, _ctx: &SimContext) -> Result<SimResult, AdapterError> {
        Err(AdapterError::Unsupported("simulate".to_string()))
    }

    fn parse_log_line(&self, _line: &str) -> Option<Diagnostic> {
        None
    }
}

/// ビルドコンテキスト
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub project_dir: PathBuf,
    pub target: String,
    pub job_id: String,
    pub constraints: Vec<PathBuf>,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// プログラムコンテキスト
#[derive(Debug, Clone)]
pub struct ProgramContext {
    pub bitstream: PathBuf,
    pub device: String,
    pub probe: Option<String>,
}

/// シミュレーションコンテキスト
#[derive(Debug, Clone)]
pub struct SimContext {
    pub testbench: String,
    pub simulator: String,
    pub work_dir: PathBuf,
}

/// ステップ実行結果
#[derive(Debug, Clone)]
pub struct StepResult {
    pub success: bool,
    pub output_dir: PathBuf,
    pub log_path: PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<Diagnostic>,
}

/// タイミングレポート
#[derive(Debug, Clone)]
pub struct TimingReport {
    pub wns: f64,
    pub tns: f64,
    pub whs: f64,
    pub ths: f64,
    pub met: bool,
    pub paths: Vec<TimingPath>,
}

/// タイミングパス
#[derive(Debug, Clone)]
pub struct TimingPath {
    pub slack: f64,
    pub source: String,
    pub destination: String,
    pub delay_ns: f64,
}

/// デバッグセッション
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub session_id: String,
    pub device: String,
    pub interface: String,
}

/// シミュレーション結果
#[derive(Debug, Clone)]
pub struct SimResult {
    pub passed: bool,
    pub vcd_path: Option<PathBuf>,
    pub log_path: PathBuf,
    pub duration_secs: f64,
}

/// 診断メッセージ
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// 診断重大度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// HDL 言語識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HdlLanguage {
    SystemVerilog,
    Verilog,
    Vhdl,
    Chisel,
    SpinalHdl,
    Amaranth,
    MyHdl,
}

/// RTL アダプター機能フラグ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RtlCapability {
    Lint,
    Sim,
    Formal,
    Transpile,
}

/// レジスタフォーマット（HAL conductor）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegisterFormat {
    SystemRdl,
    IpXact,
    Toml,
}

/// 出力言語（HAL conductor）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputLang {
    C,
    Rust,
    Python,
    Markdown,
    Svd,
}

/// ターゲットアーキテクチャ（Apps conductor）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetArch {
    ArmCortexM,
    Riscv32Imac,
    XtensaEsp32,
}

/// アプリケーション言語（Apps conductor）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguage {
    C,
    Cpp,
    Rust,
}