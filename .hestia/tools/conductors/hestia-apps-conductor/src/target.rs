//! Target configuration — CPU, flash, and RAM specifications

use serde::{Deserialize, Serialize};

/// Embedded target configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    /// CPU / core identifier (e.g., "cortex-m4", "rv32imac", "esp32")
    pub cpu: String,
    /// Total flash size in bytes
    pub flash: u64,
    /// Total RAM size in bytes
    pub ram: u64,
    /// Target triple (e.g., "thumbv7em-none-eabihf")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triple: Option<String>,
    /// Clock frequency in Hz
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clock_hz: Option<u64>,
}

impl TargetConfig {
    /// Create a new TargetConfig
    pub fn new(cpu: &str, flash: u64, ram: u64) -> Self {
        Self {
            cpu: cpu.to_string(),
            flash,
            ram,
            triple: None,
            clock_hz: None,
        }
    }

    /// Create a target for ARM Cortex-M4 with typical memory sizes
    pub fn cortex_m4(flash: u64, ram: u64) -> Self {
        Self {
            cpu: "cortex-m4".to_string(),
            flash,
            ram,
            triple: Some("thumbv7em-none-eabihf".to_string()),
            clock_hz: None,
        }
    }

    /// Create a target for RISC-V RV32IMAC
    pub fn riscv32(flash: u64, ram: u64) -> Self {
        Self {
            cpu: "rv32imac".to_string(),
            flash,
            ram,
            triple: Some("riscv32imac-unknown-none-elf".to_string()),
            clock_hz: None,
        }
    }

    /// Create a target for ESP32 (Xtensa)
    pub fn esp32(flash: u64, ram: u64) -> Self {
        Self {
            cpu: "esp32".to_string(),
            flash,
            ram,
            triple: Some("xtensa-esp32-none-elf".to_string()),
            clock_hz: None,
        }
    }

    /// Flash utilization percentage given a used amount
    pub fn flash_utilization(&self, used: u64) -> f64 {
        if self.flash == 0 {
            0.0
        } else {
            (used as f64 / self.flash as f64) * 100.0
        }
    }

    /// RAM utilization percentage given a used amount
    pub fn ram_utilization(&self, used: u64) -> f64 {
        if self.ram == 0 {
            0.0
        } else {
            (used as f64 / self.ram as f64) * 100.0
        }
    }
}