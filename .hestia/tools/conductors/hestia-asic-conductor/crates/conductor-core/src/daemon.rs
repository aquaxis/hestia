//! ASIC conductor daemon

use crate::state_machine::AsicStateMachine;

/// ASIC conductor daemon.
pub struct AsicDaemon {
    state_machine: AsicStateMachine,
}

impl AsicDaemon {
    /// Create a new ASIC conductor daemon.
    pub fn new() -> Self {
        Self {
            state_machine: AsicStateMachine::new(),
        }
    }

    /// Run the daemon loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("asic-conductor daemon starting");
        self.state_machine.transition(crate::AsicDaemonState::Idle);
        Ok(())
    }
}

impl Default for AsicDaemon {
    fn default() -> Self {
        Self::new()
    }
}