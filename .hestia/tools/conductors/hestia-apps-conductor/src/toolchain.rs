//! Toolchain manager — detect, install, and select cross-compilation toolchains

use std::path::PathBuf;

/// Toolchain manager for embedded cross-compilation
#[derive(Debug)]
pub struct ToolchainManager {
    /// Known toolchain installations
    installations: Vec<ToolchainInstallation>,
}

/// A detected or installed toolchain
#[derive(Debug, Clone)]
pub struct ToolchainInstallation {
    /// Toolchain identifier (e.g., "arm-none-eabi-gcc", "cargo")
    pub id: String,
    /// Path to the compiler binary
    pub path: PathBuf,
    /// Version string
    pub version: String,
    /// Target triple
    pub target_triple: String,
}

impl ToolchainManager {
    /// Create a new ToolchainManager
    pub fn new() -> Self {
        Self {
            installations: Vec::new(),
        }
    }

    /// Detect available toolchains on the system PATH
    pub async fn detect(&mut self) -> Result<Vec<&ToolchainInstallation>, anyhow::Error> {
        // TODO: scan PATH for known cross-compiler prefixes
        tracing::info!("detecting toolchains");
        Ok(self.installations.iter().collect())
    }

    /// Install a toolchain by name
    pub async fn install(&mut self, name: &str, version: Option<&str>) -> Result<(), anyhow::Error> {
        tracing::info!(toolchain = %name, version = ?version, "installing toolchain");
        // TODO: download and install the toolchain
        Ok(())
    }

    /// Select the best toolchain for the given target triple
    pub fn select(&self, target_triple: &str) -> Option<&ToolchainInstallation> {
        self.installations
            .iter()
            .find(|t| t.target_triple == target_triple)
    }
}

impl Default for ToolchainManager {
    fn default() -> Self {
        Self::new()
    }
}