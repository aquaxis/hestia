//! CapabilitySet — Adapter capability set

use serde::{Deserialize, Serialize};

/// FPGA adapter capability set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub synthesis: bool,
    pub implementation: bool,
    pub bitstream: bool,
    pub timing_analysis: bool,
    pub on_chip_debug: bool,
    pub device_program: bool,
    pub hls: bool,
    pub simulation: bool,
    pub ip_catalog: bool,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            synthesis: true,
            implementation: true,
            bitstream: true,
            timing_analysis: false,
            on_chip_debug: false,
            device_program: false,
            hls: false,
            simulation: false,
            ip_catalog: false,
        }
    }
}