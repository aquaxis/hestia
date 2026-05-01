//! HAL build FSM states

use serde::{Deserialize, Serialize};

/// HAL conductor build state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HalBuildState {
    Idle,
    Parsing,
    Validating,
    Generating,
    Reporting,
    Done,
    Failed,
    Diagnosing,
}

impl std::fmt::Display for HalBuildState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Parsing => write!(f, "parsing"),
            Self::Validating => write!(f, "validating"),
            Self::Generating => write!(f, "generating"),
            Self::Reporting => write!(f, "reporting"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
            Self::Diagnosing => write!(f, "diagnosing"),
        }
    }
}

impl HalBuildState {
    /// Check whether a state is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed)
    }

    /// Check whether a state is an error state
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed)
    }
}