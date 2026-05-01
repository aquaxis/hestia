//! Tool provisioner — translate tool entries into install commands inside a container

use thiserror::Error;
use tracing::info;

use crate::builder::ToolEntry;

#[derive(Debug, Error)]
pub enum ProvisionerError {
    #[error("unsupported tool: {0}")]
    UnsupportedTool(String),
    #[error("install command failed: {0}")]
    InstallFailed(String),
}

pub struct ToolProvisioner;

impl ToolProvisioner {
    pub fn new() -> Self {
        Self
    }

    /// Translate a tool entry into a shell install command.
    pub fn install_tool(&self, tool: &ToolEntry) -> Result<String, ProvisionerError> {
        info!("provisioning tool={} version={}", tool.name, tool.version);
        match tool.name.as_str() {
            "curl" | "wget" | "git" | "build-essential" => {
                Ok(format!("apt-get install -y {}={}", tool.name, tool.version))
            }
            "cargo" | "rustc" => {
                Ok(format!("rustup default {}", tool.version))
            }
            "python" | "python3" => {
                Ok(format!("apt-get install -y python3={}", tool.version))
            }
            other => Err(ProvisionerError::UnsupportedTool(other.to_string())),
        }
    }
}

impl Default for ToolProvisioner {
    fn default() -> Self {
        Self::new()
    }
}