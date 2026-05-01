//! Tool updater — detect installed versions and check for semver updates

use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum ToolUpdaterError {
    #[error("version detection failed: {0}")]
    DetectionFailed(String),
    #[error("update check failed: {0}")]
    CheckFailed(String),
}

/// Result of a version check.
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub update_available: bool,
}

pub struct ToolUpdater;

impl ToolUpdater {
    pub fn new() -> Self {
        Self
    }

    /// Detect the currently installed version of a tool.
    pub fn detect_version(&self, tool_name: &str) -> Result<String, ToolUpdaterError> {
        info!("detecting version for tool={tool_name}");
        todo!("implement tool version detection via `tool --version` or similar")
    }

    /// Check whether a newer semver version of the tool is available.
    pub fn check_updates(&self, tool_name: &str, current_version: &str) -> Result<VersionInfo, ToolUpdaterError> {
        info!("checking updates for tool={tool_name} current={current_version}");
        todo!("implement remote version lookup and semver comparison")
    }
}

impl Default for ToolUpdater {
    fn default() -> Self {
        Self::new()
    }
}