use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConductorName {
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

impl std::fmt::Display for ConductorName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ai => write!(f, "ai"),
            Self::Rtl => write!(f, "rtl"),
            Self::Fpga => write!(f, "fpga"),
            Self::Asic => write!(f, "asic"),
            Self::Pcb => write!(f, "pcb"),
            Self::Hal => write!(f, "hal"),
            Self::Apps => write!(f, "apps"),
            Self::Debug => write!(f, "debug"),
            Self::Rag => write!(f, "rag"),
        }
    }
}