//! PCB conductor state machine

use crate::PcbDaemonState;

/// State machine for the PCB conductor daemon.
pub struct PcbStateMachine {
    state: PcbDaemonState,
}

impl PcbStateMachine {
    /// Create a new state machine in the Init state.
    pub fn new() -> Self {
        Self {
            state: PcbDaemonState::Init,
        }
    }

    /// Get the current state.
    pub fn current(&self) -> PcbDaemonState {
        self.state
    }

    /// Transition to a new state.
    pub fn transition(&mut self, next: PcbDaemonState) {
        tracing::info!(from = ?self.state, to = ?next, "state transition");
        self.state = next;
    }
}

impl Default for PcbStateMachine {
    fn default() -> Self {
        Self::new()
    }
}