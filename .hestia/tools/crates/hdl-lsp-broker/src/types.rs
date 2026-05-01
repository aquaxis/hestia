use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HdlLanguage {
    Verilog,
    SystemVerilog,
    Vhdl,
    VerilogAms,
}

impl std::fmt::Display for HdlLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verilog => write!(f, "verilog"),
            Self::SystemVerilog => write!(f, "systemverilog"),
            Self::Vhdl => write!(f, "vhdl"),
            Self::VerilogAms => write!(f, "verilog-ams"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    pub language: HdlLanguage,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_max_instances")]
    pub max_instances: usize,
}

fn default_idle_timeout() -> u64 {
    300
}

fn default_max_instances() -> usize {
    4
}

impl LspServerConfig {
    pub fn svls() -> Self {
        Self {
            language: HdlLanguage::Verilog,
            command: "svls".to_string(),
            args: vec![],
            idle_timeout_secs: 300,
            max_instances: 4,
        }
    }

    pub fn vhdl_ls() -> Self {
        Self {
            language: HdlLanguage::Vhdl,
            command: "vhdl_ls".to_string(),
            args: vec![],
            idle_timeout_secs: 300,
            max_instances: 4,
        }
    }

    pub fn verilog_ams_ls() -> Self {
        Self {
            language: HdlLanguage::VerilogAms,
            command: "verilog-ams-ls".to_string(),
            args: vec![],
            idle_timeout_secs: 300,
            max_instances: 4,
        }
    }
}