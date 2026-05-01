//! ASIC conductor state machine

use crate::AsicDaemonState;

/// State machine for the ASIC conductor daemon.
pub struct AsicStateMachine {
    state: AsicDaemonState,
}

impl AsicStateMachine {
    /// Create a new state machine in the Init state.
    pub fn new() -> Self {
        Self {
            state: AsicDaemonState::Init,
        }
    }

    /// Get the current state.
    pub fn current(&self) -> AsicDaemonState {
        self.state
    }

    /// Transition to a new state.
    pub fn transition(&mut self, next: AsicDaemonState) {
        tracing::info!(from = ?self.state, to = ?next, "state transition");
        self.state = next;
    }
}

impl Default for AsicStateMachine {
    fn default() -> Self {
        Self::new()
    }
}