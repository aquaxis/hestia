//! SBOM manager — generate and scan Software Bill of Materials via syft / grype

use std::process::Command;

use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum SbomError {
    #[error("syft command failed: {0}")]
    SyftFailed(String),
    #[error("grype command failed: {0}")]
    GrypeFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Vulnerability reported by grype.
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub id: String,
    pub severity: String,
    pub package: String,
    pub version: String,
}

pub struct SbomManager;

impl SbomManager {
    pub fn new() -> Self {
        Self
    }

    /// Generate an SBOM for the given image using syft, returning the JSON output.
    pub fn generate(&self, image: &str) -> Result<String, SbomError> {
        info!("generating SBOM for image={image}");
        let output = Command::new("syft")
            .args(["-o", "json", image])
            .output()?;

        if !output.status.success() {
            return Err(SbomError::SyftFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Scan the given image for vulnerabilities using grype.
    pub fn scan(&self, image: &str) -> Result<Vec<Vulnerability>, SbomError> {
        info!("scanning image={image} for vulnerabilities");
        let output = Command::new("grype")
            .args(["-o", "json", image])
            .output()?;

        if !output.status.success() {
            return Err(SbomError::GrypeFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Best-effort parse; return empty on failure
        let vulns: Vec<Vulnerability> = serde_json::from_slice(&output.stdout)
            .map(|v: serde_json::Value| {
                v.get("matches")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|entry| {
                            let vuln = entry.get("vulnerability")?;
                            Some(Vulnerability {
                                id: vuln.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                severity: vuln.get("severity").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                package: entry.get("artifact").and_then(|a| a.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string(),
                                version: entry.get("artifact").and_then(|a| a.get("version")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            })
                        }).collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        Ok(vulns)
    }
}

impl Default for SbomManager {
    fn default() -> Self {
        Self::new()
    }
}