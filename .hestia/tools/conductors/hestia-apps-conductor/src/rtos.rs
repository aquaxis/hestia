//! RTOS configuration types for embedded applications

use serde::{Deserialize, Serialize};

/// Supported RTOS kernels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RtosConfig {
    FreeRtos,
    Zephyr,
    EmbassyRs,
    BareMetal,
}

impl RtosConfig {
    /// Human-readable name
    pub fn display_name(&self) -> &str {
        match self {
            Self::FreeRtos => "FreeRTOS",
            Self::Zephyr => "Zephyr",
            Self::EmbassyRs => "Embassy (Rust)",
            Self::BareMetal => "Bare Metal",
        }
    }

    /// Whether this RTOS requires an OS kernel
    pub fn has_kernel(&self) -> bool {
        !matches!(self, Self::BareMetal)
    }

    /// Parse from configuration string
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "freertos" | "free-rtos" => Some(Self::FreeRtos),
            "zephyr" => Some(Self::Zephyr),
            "embassy-rs" | "embassy" => Some(Self::EmbassyRs),
            "bare-metal" | "bare_metal" | "baremetal" | "none" => Some(Self::BareMetal),
            _ => None,
        }
    }
}

impl std::fmt::Display for RtosConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}