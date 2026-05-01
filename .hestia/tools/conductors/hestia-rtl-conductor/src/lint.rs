//! Lint runner for RTL designs

use crate::adapter::{RtlBuildContext, RtlStepResult};
use crate::fsm_states::RtlBuildState;

/// RTL lint runner
#[derive(Debug)]
pub struct LintRunner {
    /// Linter identifier (e.g., "verilator", "svlint")
    pub linter: String,
    /// Extra arguments for the linter
    pub extra_args: Vec<String>,
}

impl LintRunner {
    /// Create a new LintRunner for the given linter
    pub fn new(linter: &str) -> Self {
        Self {
            linter: linter.to_string(),
            extra_args: Vec::new(),
        }
    }

    /// Add extra arguments to the linter invocation
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    /// Run lint on the given build context
    pub async fn run(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError> {
        tracing::info!(linter = %self.linter, top = %ctx.top_module, "running lint");
        // TODO: invoke actual linter binary, parse output
        Ok(RtlStepResult {
            success: true,
            output_dir: ctx.project_dir.join("build").join("lint"),
            log_path: ctx.project_dir.join("build").join("lint").join("lint.log"),
            duration_secs: 0.0,
            diagnostics: Vec::new(),
            state: RtlBuildState::Linting,
        })
    }
}