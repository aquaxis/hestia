use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WaveformFormat {
    Vcd,
    Fst,
    Ghw,
    Evcd,
}

impl std::fmt::Display for WaveformFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vcd => write!(f, "VCD"),
            Self::Fst => write!(f, "FST"),
            Self::Ghw => write!(f, "GHW"),
            Self::Evcd => write!(f, "EVCD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalType {
    Wire,
    Reg,
    Integer,
    Real,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignalValue {
    Logic(char),
    Vector { bits: String, hex: String },
    Real(f64),
    StringVal(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signal {
    pub id: String,
    pub full_name: String,
    pub display_name: String,
    pub bit_width: u32,
    pub signal_type: SignalType,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WaveformHeader {
    pub version: Option<String>,
    pub date: Option<String>,
    pub timescale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WaveformData {
    pub header: WaveformHeader,
    pub signals: Vec<Signal>,
    pub format: WaveformFormat,
}