//! Apps build FSM states

use serde::{Deserialize, Serialize};

/// Apps conductor build state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppsBuildState {
    Idle,
    Resolving,
    Compiling,
    Linking,
    SizeChecking,
    Flashing,
    Testing,
    Done,
    Failed,
    Diagnosing,
}

impl std::fmt::Display for AppsBuildState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "idle"),
            Self::Resolving => write!(f, "resolving"),
            Self::Compiling => write!(f, "compiling"),
            Self::Linking => write!(f, "linking"),
            Self::SizeChecking => write!(f, "size_checking"),
            Self::Flashing => write!(f, "flashing"),
            Self::Testing => write!(f, "testing"),
            Self::Done => write!(f, "done"),
            Self::Failed => write!(f, "failed"),
            Self::Diagnosing => write!(f, "diagnosing"),
        }
    }
}

impl AppsBuildState {
    /// Check whether a state is terminal
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Done | Self::Failed)
    }

    /// Check whether a state is an error state
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed)
    }
}