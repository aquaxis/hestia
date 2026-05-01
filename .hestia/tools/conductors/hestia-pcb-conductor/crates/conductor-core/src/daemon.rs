//! PCB conductor daemon

use crate::state_machine::PcbStateMachine;

/// PCB conductor daemon.
pub struct PcbDaemon {
    state_machine: PcbStateMachine,
}

impl PcbDaemon {
    /// Create a new PCB conductor daemon.
    pub fn new() -> Self {
        Self {
            state_machine: PcbStateMachine::new(),
        }
    }

    /// Run the daemon loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("pcb-conductor daemon starting");
        self.state_machine.transition(crate::PcbDaemonState::Idle);
        Ok(())
    }
}

impl Default for PcbDaemon {
    fn default() -> Self {
        Self::new()
    }
}