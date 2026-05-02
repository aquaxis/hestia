//! Container updater — check for and apply diff-based container updates

use std::process::Command;

use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum UpdaterError {
    #[error("update check failed: {0}")]
    CheckFailed(String),
    #[error("apply failed: {0}")]
    ApplyFailed(String),
}

/// Describes an available update for a container image.
#[derive(Debug, Clone)]
pub struct ContainerUpdate {
    pub image: String,
    pub current_digest: String,
    pub new_digest: String,
}

pub struct ContainerUpdater;

impl ContainerUpdater {
    pub fn new() -> Self {
        Self
    }

    /// Check whether a newer image digest is available for the given image reference.
    ///
    /// Uses `skopeo inspect` to compare local and remote digests.
    /// Returns `Ok(None)` if no update is available or if skopeo is not installed.
    pub fn check_update(&self, image: &str) -> Result<Option<ContainerUpdate>, UpdaterError> {
        info!("checking for updates to image={image}");

        // Get local digest
        let local_output = Command::new("skopeo")
            .args(["inspect", &format!("containers-storage:{image}")])
            .output();

        let local_digest = match local_output {
            Ok(output) if output.status.success() => {
                let json: serde_json::Value = serde_json::from_slice(&output.stdout)
                    .map_err(|e| UpdaterError::CheckFailed(format!("failed to parse local inspect: {e}")))?;
                json.get("Digest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            }
            _ => {
                warn!("skopeo inspect failed for local image={image}, no update check possible");
                return Ok(None);
            }
        };

        // Get remote digest
        let remote_output = Command::new("skopeo")
            .args(["inspect", &format!("docker://{image}")])
            .output();

        let new_digest = match remote_output {
            Ok(output) if output.status.success() => {
                let json: serde_json::Value = serde_json::from_slice(&output.stdout)
                    .map_err(|e| UpdaterError::CheckFailed(format!("failed to parse remote inspect: {e}")))?;
                json.get("Digest")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string()
            }
            _ => {
                warn!("skopeo inspect failed for remote image={image}");
                return Ok(None);
            }
        };

        if local_digest == new_digest {
            info!("image={image} is up to date");
            Ok(None)
        } else {
            info!("update available for image={image}: {local_digest} -> {new_digest}");
            Ok(Some(ContainerUpdate {
                image: image.to_string(),
                current_digest: local_digest,
                new_digest,
            }))
        }
    }

    /// Apply a diff-based update by pulling only the changed layers.
    ///
    /// Uses `skopeo copy` to pull the new image layers and re-tag.
    pub fn apply_update(&self, update: &ContainerUpdate) -> Result<(), UpdaterError> {
        info!(
            "applying update for image={} old={} new={}",
            update.image, update.current_digest, update.new_digest
        );

        let status = Command::new("skopeo")
            .args([
                "copy",
                &format!("docker://{}", update.image),
                &format!("containers-storage:{}", update.image),
            ])
            .status()
            .map_err(|e| UpdaterError::ApplyFailed(format!("failed to run skopeo: {e}")))?;

        if status.success() {
            info!("update applied for image={}", update.image);
            Ok(())
        } else {
            Err(UpdaterError::ApplyFailed(format!(
                "skopeo copy exited with code {:?}",
                status.code()
            )))
        }
    }
}

impl Default for ContainerUpdater {
    fn default() -> Self {
        Self::new()
    }
}