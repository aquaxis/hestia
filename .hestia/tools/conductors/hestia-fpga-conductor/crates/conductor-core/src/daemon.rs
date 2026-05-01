//! FPGA conductor daemon

use crate::state_machine::FpgaStateMachine;

/// FPGA conductor daemon.
pub struct FpgaDaemon {
    state_machine: FpgaStateMachine,
}

impl FpgaDaemon {
    /// Create a new FPGA conductor daemon.
    pub fn new() -> Self {
        Self {
            state_machine: FpgaStateMachine::new(),
        }
    }

    /// Run the daemon loop.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("fpga-conductor daemon starting");
        self.state_machine.transition(crate::FpgaDaemonState::Idle);
        Ok(())
    }
}

impl Default for FpgaDaemon {
    fn default() -> Self {
        Self::new()
    }
}