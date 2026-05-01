//! Formal verification runner for RTL designs

use crate::adapter::{RtlBuildContext, RtlStepResult};
use crate::fsm_states::RtlBuildState;

/// Formal verification runner
#[derive(Debug)]
pub struct FormalRunner {
    /// Formal tool identifier (e.g., "sby", "formal", "jasper")
    pub tool: String,
    /// Extra arguments for the formal tool
    pub extra_args: Vec<String>,
}

impl FormalRunner {
    /// Create a new FormalRunner for the given formal tool
    pub fn new(tool: &str) -> Self {
        Self {
            tool: tool.to_string(),
            extra_args: Vec::new(),
        }
    }

    /// Add extra arguments to the formal tool invocation
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    /// Run formal verification on the given build context
    pub async fn run(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError> {
        tracing::info!(tool = %self.tool, top = %ctx.top_module, "running formal verification");
        // TODO: invoke actual formal verification tool, parse output
        Ok(RtlStepResult {
            success: true,
            output_dir: ctx.project_dir.join("build").join("formal"),
            log_path: ctx.project_dir.join("build").join("formal").join("formal.log"),
            duration_secs: 0.0,
            diagnostics: Vec::new(),
            state: RtlBuildState::FormalChecking,
        })
    }
}