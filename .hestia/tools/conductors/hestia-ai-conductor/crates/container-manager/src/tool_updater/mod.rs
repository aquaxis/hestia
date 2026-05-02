//! Tool updater — detect installed versions and check for semver updates

use std::process::Command;

use thiserror::Error;
use tracing::{info, warn};

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

    /// Detect the currently installed version of a tool by running `<tool> --version`.
    ///
    /// Falls back to `<tool> -V` if `--version` fails. Returns "unknown" if
    /// the tool is not installed or its version string cannot be parsed.
    pub fn detect_version(&self, tool_name: &str) -> Result<String, ToolUpdaterError> {
        info!("detecting version for tool={tool_name}");

        // Try --version first, then -V
        for args in [&["--version"][..], &["-V"]] {
            let output = Command::new(tool_name)
                .args(args)
                .output();

            match output {
                Ok(output) if output.status.success() => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Extract the first version-like pattern (e.g., "1.2.3" or "v1.2.3")
                    let version = extract_version(&stdout);
                    if !version.is_empty() {
                        info!("detected version for tool={tool_name}: {version}");
                        return Ok(version);
                    }
                }
                _ => continue,
            }
        }

        // Tool not found or version not parseable
        warn!("could not detect version for tool={tool_name}");
        Ok("unknown".to_string())
    }

    /// Check whether a newer semver version of the tool is available.
    ///
    /// Compares the current version against the latest available version.
    /// For tools that support `--version` and have a known update mechanism,
    /// this attempts to determine if an update is available.
    pub fn check_updates(&self, tool_name: &str, current_version: &str) -> Result<VersionInfo, ToolUpdaterError> {
        info!("checking updates for tool={tool_name} current={current_version}");

        // Use the same detection method to get current version
        let detected = self.detect_version(tool_name)?;
        let current = if detected != "unknown" { detected } else { current_version.to_string() };

        // For now, report current version as latest since we cannot reliably
        // query remote version databases for arbitrary tools.
        // Future: integrate with GitHub releases API or similar for known tools.
        let latest = current.clone();

        Ok(VersionInfo {
            name: tool_name.to_string(),
            current,
            latest,
            update_available: false,
        })
    }
}

/// Extract a semver-like version string from tool output.
fn extract_version(output: &str) -> String {
    for line in output.lines() {
        for part in line.split_whitespace() {
            // Match patterns like "1.2.3", "v1.2.3", "1.2.3-beta"
            if let Some(v) = part.strip_prefix('v') {
                if v.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    return v.to_string();
                }
            }
            if part.chars().next().map_or(false, |c| c.is_ascii_digit())
                && part.contains('.')
            {
                return part.to_string();
            }
        }
    }
    String::new()
}

impl Default for ToolUpdater {
    fn default() -> Self {
        Self::new()
    }
}