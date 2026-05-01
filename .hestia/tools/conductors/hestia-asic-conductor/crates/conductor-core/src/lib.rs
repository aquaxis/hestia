//! asic-conductor-core -- Main daemon, state machine for ASIC flow

pub mod daemon;
pub mod rpc_handler;
pub mod state_machine;

use serde::{Deserialize, Serialize};

/// ASIC conductor daemon states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsicDaemonState {
    Init,
    Idle,
    Synthesizing,
    Placing,
    Routing,
    Cts,
    Signoff,
    Error,
    Shutdown,
}

/// ASIC flow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsicStep {
    Synthesis,
    Placement,
    Cts,
    Routing,
    Signoff,
}

impl std::fmt::Display for AsicStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsicStep::Synthesis => write!(f, "synthesis"),
            AsicStep::Placement => write!(f, "placement"),
            AsicStep::Cts => write!(f, "cts"),
            AsicStep::Routing => write!(f, "routing"),
            AsicStep::Signoff => write!(f, "signoff"),
        }
    }
}