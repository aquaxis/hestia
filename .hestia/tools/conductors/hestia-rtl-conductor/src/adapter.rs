//! RTL Tool Adapter trait — domain-specific extension of adapter_core::ToolAdapter

use async_trait::async_trait;
use adapter_core::{ToolAdapter, capability::CapabilitySet};
use crate::language::HdlLanguage;
use crate::fsm_states::RtlBuildState;

/// RTL domain ToolAdapter trait
#[async_trait]
pub trait RtlToolAdapter: ToolAdapter {
    /// Supported HDL languages
    fn supported_languages(&self) -> Vec<HdlLanguage>;

    /// Adapter-specific capability flags
    fn capabilities(&self) -> &CapabilitySet;

    /// Run lint
    async fn lint(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;

    /// Run simulation
    async fn simulate(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;

    /// Run formal verification
    async fn formal_verify(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;

    /// Transpile (HDL-to-HDL conversion)
    async fn transpile(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError>;
}

/// RTL build context
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

/// RTL step execution result
#[derive(Debug, Clone)]
pub struct RtlStepResult {
    pub success: bool,
    pub output_dir: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<adapter_core::Diagnostic>,
    pub state: RtlBuildState,
}