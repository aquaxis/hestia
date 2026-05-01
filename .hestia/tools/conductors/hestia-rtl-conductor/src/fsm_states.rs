//! RTL build FSM states

use serde::{Deserialize, Serialize};

/// RTL conductor build state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RtlBuildState {
    Idle,
    Resolving,
    Linting,
    Compiling,
    Simulating,
    FormalChecking,
    Reporting,
    Done,
    Failed,
    Diagnosing,
}

impl std::fmt::Display for RtlBuildState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Resolving => write!(f, "resolving"),
            Self::Linting => write!(f, "linting"),
            Self::Compiling => write!(f, "compiling"),
            Self::Simulating => write!(f, "simulating"),
            Self::FormalChecking => write!(f, "formal_checking"),
            Self::Reporting => write!(f, "reporting"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
            Self::Diagnosing => write!(f, "diagnosing"),
        }
    }
}

/// Check whether a state is terminal
impl RtlBuildState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed)
    }

    /// Check whether a state is an error state
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed)
    }
}