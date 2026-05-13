//! HAL Tool Adapter trait — domain-specific extension of adapter_core::ToolAdapter

use async_trait::async_trait;
use adapter_core::{ToolAdapter, capability::CapabilitySet};
use crate::bus_protocol::BusProtocol;
use crate::fsm_states::HalBuildState;

/// HAL domain ToolAdapter trait
#[async_trait]
pub trait HalToolAdapter: ToolAdapter {
    /// Supported input formats (e.g., "systemrdl", "ipxact", "toml")
    fn supported_inputs(&self) -> Vec<String>;

    /// Supported output languages (e.g., "c", "rust", "python", "svd")
    fn supported_outputs(&self) -> Vec<String>;

    /// Adapter-specific capability flags
    fn capabilities(&self) -> &CapabilitySet;

    /// Parse register definitions
    async fn parse(&self, ctx: &HalBuildContext) -> Result<HalStepResult, adapter_core::error::AdapterError>;

    /// Validate parsed results
    async fn validate(&self, ctx: &HalBuildContext) -> Result<HalStepResult, adapter_core::error::AdapterError>;

    /// Generate code
    async fn generate(&self, ctx: &HalBuildContext) -> Result<HalStepResult, adapter_core::error::AdapterError>;
}

/// HAL build context
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

/// HAL step execution result
#[derive(Debug, Clone)]
pub struct HalStepResult {
    pub success: bool,
    pub output_dir: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub duration_secs: f64,
    pub diagnostics: Vec<adapter_core::Diagnostic>,
    pub state: HalBuildState,
}