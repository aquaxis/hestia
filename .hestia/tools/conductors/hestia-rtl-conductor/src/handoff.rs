//! Handoff manager — transfers RTL artifacts to downstream conductors

use crate::adapter::RtlBuildContext;
use crate::fsm_states::RtlBuildState;

/// Handoff manager for RTL-to-downstream conductor transfers
#[derive(Debug)]
pub struct HandoffManager {
    /// Project directory
    project_dir: std::path::PathBuf,
}

impl HandoffManager {
    /// Create a new HandoffManager
    pub fn new(project_dir: std::path::PathBuf) -> Self {
        Self { project_dir }
    }

    /// Hand off RTL artifacts to FPGA conductor
    pub async fn handoff_to_fpga(&self, ctx: &RtlBuildContext) -> Result<HandoffResult, adapter_core::error::AdapterError> {
        tracing::info!(top = %ctx.top_module, "handing off to FPGA conductor");
        // TODO: serialize artifacts, notify FPGA conductor via transport
        Ok(HandoffResult {
            success: true,
            target: HandoffTarget::Fpga,
            artifact_dir: self.project_dir.join("build").join("handoff").join("fpga"),
            state: RtlBuildState::Done,
        })
    }

    /// Hand off RTL artifacts to ASIC conductor
    pub async fn handoff_to_asic(&self, ctx: &RtlBuildContext) -> Result<HandoffResult, adapter_core::error::AdapterError> {
        tracing::info!(top = %ctx.top_module, "handing off to ASIC conductor");
        // TODO: serialize artifacts, notify ASIC conductor via transport
        Ok(HandoffResult {
            success: true,
            target: HandoffTarget::Asic,
            artifact_dir: self.project_dir.join("build").join("handoff").join("asic"),
            state: RtlBuildState::Done,
        })
    }
}

/// Handoff target conductor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandoffTarget {
    Fpga,
    Asic,
}

/// Result of a handoff operation
#[derive(Debug)]
pub struct HandoffResult {
    pub success: bool,
    pub target: HandoffTarget,
    pub artifact_dir: std::path::PathBuf,
    pub state: RtlBuildState,
}