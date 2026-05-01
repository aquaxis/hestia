//! Bus protocol definitions for HAL conductor

use serde::{Deserialize, Serialize};

/// Supported bus protocols for register map interfaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BusProtocol {
    Axi4Lite,
    Axi4,
    WishboneB4,
    AhbLite,
}

impl BusProtocol {
    /// Default data width for this protocol
    pub fn default_data_width(&self) -> u32 {
        match self {
            Self::Axi4Lite => 32,
            Self::Axi4 => 32,
            Self::WishboneB4 => 32,
            Self::AhbLite => 32,
        }
    }

    /// Default address width for this protocol
    pub fn default_addr_width(&self) -> u32 {
        match self {
            Self::Axi4Lite => 32,
            Self::Axi4 => 32,
            Self::WishboneB4 => 32,
            Self::AhbLite => 32,
        }
    }

    /// Human-readable name
    pub fn display_name(&self) -> &str {
        match self {
            Self::Axi4Lite => "AXI4-Lite",
            Self::Axi4 => "AXI4",
            Self::WishboneB4 => "Wishbone B4",
            Self::AhbLite => "AHB-Lite",
        }
    }

    /// Parse from TOML/CLI string
    pub fn from_config_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "axi4-lite" | "axi4lite" => Some(Self::Axi4Lite),
            "axi4" => Some(Self::Axi4),
            "wishbone-b4" | "wishboneb4" => Some(Self::WishboneB4),
            "ahb-lite" | "ahblite" => Some(Self::AhbLite),
            _ => None,
        }
    }
}

impl std::fmt::Display for BusProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}