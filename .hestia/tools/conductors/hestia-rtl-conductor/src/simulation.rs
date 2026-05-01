//! Simulation runner for RTL designs

use crate::adapter::{RtlBuildContext, RtlStepResult};
use crate::fsm_states::RtlBuildState;

/// RTL simulation runner
#[derive(Debug)]
pub struct SimRunner {
    /// Simulator identifier (e.g., "verilator", "iverilog", "vcs")
    pub simulator: String,
    /// Extra arguments for the simulator
    pub extra_args: Vec<String>,
}

impl SimRunner {
    /// Create a new SimRunner for the given simulator
    pub fn new(simulator: &str) -> Self {
        Self {
            simulator: simulator.to_string(),
            extra_args: Vec::new(),
        }
    }

    /// Add extra arguments to the simulator invocation
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }

    /// Run simulation on the given build context
    pub async fn run(&self, ctx: &RtlBuildContext) -> Result<RtlStepResult, adapter_core::error::AdapterError> {
        tracing::info!(simulator = %self.simulator, top = %ctx.top_module, "running simulation");
        // TODO: invoke actual simulator binary, parse output
        Ok(RtlStepResult {
            success: true,
            output_dir: ctx.project_dir.join("build").join("sim"),
            log_path: ctx.project_dir.join("build").join("sim").join("sim.log"),
            duration_secs: 0.0,
            diagnostics: Vec::new(),
            state: RtlBuildState::Simulating,
        })
    }
}