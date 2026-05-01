//! fpga-conductor-core -- Main daemon entry, RPC handler, state machine

pub mod daemon;
pub mod rpc_handler;
pub mod state_machine;

use serde::{Deserialize, Serialize};

/// FPGA conductor daemon states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FpgaDaemonState {
    Init,
    Idle,
    Synthesizing,
    Implementing,
    GeneratingBitstream,
    Programming,
    Error,
    Shutdown,
}

/// FPGA build pipeline step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FpgaStep {
    Synthesize,
    Implement,
    GenerateBitstream,
    ProgramDevice,
}

impl std::fmt::Display for FpgaStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FpgaStep::Synthesize => write!(f, "synthesize"),
            FpgaStep::Implement => write!(f, "implement"),
            FpgaStep::GenerateBitstream => write!(f, "generate_bitstream"),
            FpgaStep::ProgramDevice => write!(f, "program_device"),
        }
    }
}