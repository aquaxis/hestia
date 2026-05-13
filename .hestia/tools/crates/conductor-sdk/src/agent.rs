//! Conductor ID, status, and peer name definitions

use serde::{Deserialize, Serialize};

/// Conductor identifier (1:1 correspondence with agent-cli peer name)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConductorId {
    Ai,
    Rtl,
    Fpga,
    Asic,
    Pcb,
    Hal,
    Apps,
    Debug,
    Rag,
}

impl ConductorId {
    /// Return agent-cli peer name
    pub fn peer_name(&self) -> &'static str {
        match self {
            Self::Ai => "ai",
            Self::Rtl => "rtl",
            Self::Fpga => "fpga",
            Self::Asic => "asic",
            Self::Pcb => "pcb",
            Self::Hal => "hal",
            Self::Apps => "apps",
            Self::Debug => "debug",
            Self::Rag => "rag",
        }
    }

    /// Return all ConductorId values
    pub fn all() -> &'static [ConductorId] {
        &[
            Self::Ai,
            Self::Rtl,
            Self::Fpga,
            Self::Asic,
            Self::Pcb,
            Self::Hal,
            Self::Apps,
            Self::Debug,
            Self::Rag,
        ]
    }
}

impl std::fmt::Display for ConductorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.peer_name())
    }
}

impl std::str::FromStr for ConductorId {
    type Err = crate::error::HestiaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ai" => Ok(Self::Ai),
            "rtl" => Ok(Self::Rtl),
            "fpga" => Ok(Self::Fpga),
            "asic" => Ok(Self::Asic),
            "pcb" => Ok(Self::Pcb),
            "hal" => Ok(Self::Hal),
            "apps" => Ok(Self::Apps),
            "debug" => Ok(Self::Debug),
            "rag" => Ok(Self::Rag),
            _ => Err(crate::error::HestiaError::InvalidConductorId(s.to_string())),
        }
    }
}

/// Shared service peer name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServicePeer {
    Lsp,
    ConstraintBridge,
    IpManager,
    Cicd,
    Observability,
    Waveform,
    Mcp,
}

impl ServicePeer {
    pub fn peer_name(&self) -> &'static str {
        match self {
            Self::Lsp => "lsp",
            Self::ConstraintBridge => "constraint-bridge",
            Self::IpManager => "ip-manager",
            Self::Cicd => "cicd",
            Self::Observability => "observability",
            Self::Waveform => "waveform",
            Self::Mcp => "mcp",
        }
    }
}

/// Frontend peer name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrontendPeer {
    Vscode,
    Tauri,
    Cli,
}

impl FrontendPeer {
    pub fn peer_name(&self) -> &'static str {
        match self {
            Self::Vscode => "vscode",
            Self::Tauri => "tauri",
            Self::Cli => "cli",
        }
    }
}

/// Conductor status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConductorStatus {
    Online,
    Offline,
    Degraded,
    Upgrading,
}

impl std::fmt::Display for ConductorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Degraded => "degraded",
            Self::Upgrading => "upgrading",
        }
        .fmt(f)
    }
}

/// Conductor info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConductorInfo {
    pub id: ConductorId,
    pub status: ConductorStatus,
    pub version: String,
    pub uptime_secs: u64,
}