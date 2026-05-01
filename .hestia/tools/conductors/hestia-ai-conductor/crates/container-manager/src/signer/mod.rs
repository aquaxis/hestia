//! Container signer — sign and verify container images via cosign

use std::process::Command;

use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum SignerError {
    #[error("cosign command failed: {0}")]
    CommandFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ContainerSigner {
    /// Optional key reference for signing (e.g. "env://COSIGN_PRIVATE_KEY").
    key_ref: Option<String>,
}

impl ContainerSigner {
    pub fn new(key_ref: Option<String>) -> Self {
        Self { key_ref }
    }

    /// Sign the given image reference using cosign.
    pub fn sign(&self, image: &str) -> Result<(), SignerError> {
        info!("signing image={image}");
        let mut cmd = Command::new("cosign");
        cmd.arg("sign").arg(image);
        if let Some(ref key) = self.key_ref {
            cmd.arg("--key").arg(key);
        }
        let status = cmd.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(SignerError::CommandFailed(format!(
                "cosign sign exited with code {:?}",
                status.code()
            )))
        }
    }

    /// Verify the signature of the given image reference using cosign.
    pub fn verify(&self, image: &str) -> Result<(), SignerError> {
        info!("verifying image={image}");
        let mut cmd = Command::new("cosign");
        cmd.arg("verify").arg(image);
        if let Some(ref key) = self.key_ref {
            cmd.arg("--key").arg(key);
        }
        let status = cmd.status()?;

        if status.success() {
            Ok(())
        } else {
            Err(SignerError::CommandFailed(format!(
                "cosign verify exited with code {:?}",
                status.code()
            )))
        }
    }
}

impl Default for ContainerSigner {
    fn default() -> Self {
        Self::new(None)
    }
}