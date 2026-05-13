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

/// Generic ToolAdapter trait (implemented in RTL / HAL / Apps domains)
#[async_trait]
pub trait ToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> &CapabilitySet;
}

/// FPGA VendorAdapter trait (section 5.2 unified interface)
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

/// Build context
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub project_dir: PathBuf,
    pub target: String,
    pub job_id: String,
    pub constraints: Vec<PathBuf>,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// Program context
#[derive(Debug, Clone)]
pub struct ProgramContext {
    pub bitstream: PathBuf,
    pub device: String,
    pub probe: Option<String>,
}

/// Simulation context
#[derive(Debug, Clone)]
pub struct SimContext {
    pub testbench: String,
    pub simulator: String,
    pub work_dir: PathBuf,
}

/// Step execution result
#[derive(Debug, Clone)]
pub struct StepResult {
    pub success: bool,
    pub output_dir: PathBuf,
    pub log_path: PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<Diagnostic>,
}

/// Timing report
#[derive(Debug, Clone)]
pub struct TimingReport {
    pub wns: f64,
    pub tns: f64,
    pub whs: f64,
    pub ths: f64,
    pub met: bool,
    pub paths: Vec<TimingPath>,
}

/// Timing path
#[derive(Debug, Clone)]
pub struct TimingPath {
    pub slack: f64,
    pub source: String,
    pub destination: String,
    pub delay_ns: f64,
}

/// Debug session
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub session_id: String,
    pub device: String,
    pub interface: String,
}

/// Simulation result
#[derive(Debug, Clone)]
pub struct SimResult {
    pub passed: bool,
    pub vcd_path: Option<PathBuf>,
    pub log_path: PathBuf,
    pub duration_secs: f64,
}

/// Diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

/// HDL language identifier
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

/// RTL adapter capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RtlCapability {
    Lint,
    Sim,
    Formal,
    Transpile,
}

/// Register format (HAL conductor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegisterFormat {
    SystemRdl,
    IpXact,
    Toml,
}

/// Output language (HAL conductor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputLang {
    C,
    Rust,
    Python,
    Markdown,
    Svd,
}

/// Target architecture (Apps conductor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetArch {
    ArmCortexM,
    Riscv32Imac,
    XtensaEsp32,
}

/// Application language (Apps conductor)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppLanguage {
    C,
    Cpp,
    Rust,
}