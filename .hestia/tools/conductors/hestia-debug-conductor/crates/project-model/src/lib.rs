//! debug-project-model -- Configuration parser for debug conductor projects

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur while parsing the debug project configuration.
#[derive(Debug, Error)]
pub enum ProjectModelParseError {
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("invalid device identifier: {0}")]
    InvalidDevice(String),
}

/// Top-level debug project configuration (parsed from `debug.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugProjectConfig {
    /// Project name.
    pub name: String,
    /// Target device identifier (e.g. "stm32f407", "xc7z010").
    pub device: String,
    /// Debug interface configuration.
    pub interface: InterfaceConfig,
    /// Optional adapter overrides.
    #[serde(default)]
    pub adapters: Vec<AdapterConfig>,
}

/// Debug interface configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    /// Interface type: "jtag" or "swd".
    pub kind: String,
    /// Clock frequency in Hz.
    #[serde(default)]
    pub clock_hz: u64,
    /// Optional adapter-specific settings.
    #[serde(default)]
    pub extra: serde_json::Value,
}

/// Adapter configuration override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// Adapter name (e.g. "jtag", "swd", "ila").
    pub name: String,
    /// Whether the adapter is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Adapter-specific parameters.
    #[serde(default)]
    pub params: serde_json::Value,
}

fn default_true() -> bool {
    true
}

/// Configuration parser for debug project files.
pub struct ConfigParser;

impl ConfigParser {
    /// Parse a debug project configuration from a TOML string.
    pub fn parse_toml(input: &str) -> Result<DebugProjectConfig, ProjectModelParseError> {
        let config: DebugProjectConfig = toml::from_str(input)?;
        if config.device.is_empty() {
            return Err(ProjectModelParseError::InvalidDevice(
                "device field must not be empty".to_string(),
            ));
        }
        Ok(config)
    }

    /// Parse a debug project configuration from a JSON string.
    pub fn parse_json(input: &str) -> Result<DebugProjectConfig, ProjectModelParseError> {
        let config: DebugProjectConfig =
            serde_json::from_str(input).map_err(|e| ProjectModelParseError::MissingField(e.to_string()))?;
        if config.device.is_empty() {
            return Err(ProjectModelParseError::InvalidDevice(
                "device field must not be empty".to_string(),
            ));
        }
        Ok(config)
    }
}