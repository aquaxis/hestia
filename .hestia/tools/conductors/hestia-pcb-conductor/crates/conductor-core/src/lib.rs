//! pcb-conductor-core -- Main daemon, state machine for PCB flow

pub mod daemon;
pub mod rpc_handler;
pub mod state_machine;

use serde::{Deserialize, Serialize};

/// PCB conductor daemon states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PcbDaemonState {
    Init,
    Idle,
    SchematicCapture,
    Placement,
    Routing,
    Drc,
    Export,
    Error,
    Shutdown,
}

/// PCB flow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PcbStep {
    SchematicCapture,
    Placement,
    Routing,
    Drc,
    Export,
}

impl std::fmt::Display for PcbStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PcbStep::SchematicCapture => write!(f, "schematic_capture"),
            PcbStep::Placement => write!(f, "placement"),
            PcbStep::Routing => write!(f, "routing"),
            PcbStep::Drc => write!(f, "drc"),
            PcbStep::Export => write!(f, "export"),
        }
    }
}