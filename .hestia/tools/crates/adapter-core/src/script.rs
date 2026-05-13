//! ScriptAdapter — adapter.toml declaration style (Principle 2: Zero-modification extension)

use serde::{Deserialize, Serialize};
use std::path::Path;

/// adapter.toml schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterToml {
    pub manifest: super::manifest::AdapterManifest,
    pub tool: ToolConfig,
    #[serde(default)]
    pub commands: CommandConfig,
    #[serde(default)]
    pub log_parsing: LogParsingConfig,
    #[serde(default)]
    pub report_extraction: ReportExtractionConfig,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

/// Command configuration (command mapping for each build step)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandConfig {
    #[serde(default)]
    pub synthesize: Option<StepCommand>,
    #[serde(default)]
    pub implement: Option<StepCommand>,
    #[serde(default)]
    pub bitstream: Option<StepCommand>,
    #[serde(default)]
    pub timing: Option<StepCommand>,
    #[serde(default)]
    pub program: Option<StepCommand>,
}

/// Step command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCommand {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Log parsing rules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogParsingConfig {
    #[serde(default)]
    pub error_pattern: Option<String>,
    #[serde(default)]
    pub warning_pattern: Option<String>,
    #[serde(default)]
    pub info_pattern: Option<String>,
}

/// Report extraction rules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportExtractionConfig {
    #[serde(default)]
    pub timing_pattern: Option<String>,
    #[serde(default)]
    pub resource_pattern: Option<String>,
}

/// Load adapter.toml from file
pub fn load_adapter_toml(path: &Path) -> Result<AdapterToml, super::error::AdapterError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        super::error::AdapterError::Io(std::io::Error::other(format!(
            "Failed to read adapter.toml at {}: {e}",
            path.display()
        )))
    })?;
    toml::from_str(&content)
        .map_err(|e| super::error::AdapterError::Parse(format!("adapter.toml parse error: {e}")))
}