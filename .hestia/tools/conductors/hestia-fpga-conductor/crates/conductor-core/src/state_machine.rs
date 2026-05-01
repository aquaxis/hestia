//! FPGA conductor state machine

use crate::FpgaDaemonState;

/// State machine for the FPGA conductor daemon.
pub struct FpgaStateMachine {
    state: FpgaDaemonState,
}

impl FpgaStateMachine {
    /// Create a new state machine in the Init state.
    pub fn new() -> Self {
        Self {
            state: FpgaDaemonState::Init,
        }
    }

    /// Get the current state.
    pub fn current(&self) -> FpgaDaemonState {
        self.state
    }

    /// Transition to a new state.
    pub fn transition(&mut self, next: FpgaDaemonState) {
        tracing::info!(from = ?self.state, to = ?next, "state transition");
        self.state = next;
    }
}

impl Default for FpgaStateMachine {
    fn default() -> Self {
        Self::new()
    }
}