//! Container updater — check for and apply diff-based container updates

use thiserror::Error;
use tracing::info;

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
    pub fn check_update(&self, image: &str) -> Result<Option<ContainerUpdate>, UpdaterError> {
        info!("checking for updates to image={image}");
        todo!("implement remote digest comparison via skopeo inspect")
    }

    /// Apply a diff-based update by pulling only the changed layers.
    pub fn apply_update(&self, update: &ContainerUpdate) -> Result<(), UpdaterError> {
        info!(
            "applying update for image={} old={} new={}",
            update.image, update.current_digest, update.new_digest
        );
        todo!("implement layer-diff pull and re-tag")
    }
}

impl Default for ContainerUpdater {
    fn default() -> Self {
        Self::new()
    }
}