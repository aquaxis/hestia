//! Container registry — push, pull, and list images via skopeo

use std::process::Command;

use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Skopeo command failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ContainerRegistry {
    /// Registry URL (e.g. "docker.io/org")
    registry_url: String,
}

impl ContainerRegistry {
    pub fn new(registry_url: &str) -> Self {
        Self {
            registry_url: registry_url.to_string(),
        }
    }

    /// Push a local image to the remote registry.
    pub fn push(&self, image: &str) -> Result<(), RegistryError> {
        let dest = format!("{}/{}", self.registry_url, image);
        info!("pushing image to {dest}");
        let status = Command::new("skopeo")
            .args(["copy", &format!("containers-storage:{image}"), &format!("docker://{dest}")])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(RegistryError::CommandFailed(format!(
                "skopeo push exited with code {:?}",
                status.code()
            )))
        }
    }

    /// Pull an image from the remote registry into local storage.
    pub fn pull(&self, image: &str) -> Result<(), RegistryError> {
        let src = format!("{}/{}", self.registry_url, image);
        info!("pulling image from {src}");
        let status = Command::new("skopeo")
            .args(["copy", &format!("docker://{src}"), &format!("containers-storage:{image}")])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(RegistryError::CommandFailed(format!(
                "skopeo pull exited with code {:?}",
                status.code()
            )))
        }
    }

    /// List images available in the remote registry.
    pub fn list(&self) -> Result<Vec<String>, RegistryError> {
        info!("listing images in registry {}", self.registry_url);
        // Minimal implementation — delegate to skopeo list-tags
        let output = Command::new("skopeo")
            .args(["list-tags", &format!("docker://{}", self.registry_url)])
            .output()?;

        if !output.status.success() {
            return Err(RegistryError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        // Best-effort parse; return empty on failure
        let tags: Vec<String> = serde_json::from_slice(&output.stdout)
            .map(|v: serde_json::Value| {
                v.get("Tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| arr.iter().filter_map(|t| t.as_str().map(String::from)).collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        Ok(tags)
    }
}