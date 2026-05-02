//! adapter-efinity -- Efinity Python API adapter

pub mod python_api;

use adapter_core::manifest::AdapterManifest;
use adapter_core::capability::CapabilitySet;
use adapter_core::error::AdapterError;
use adapter_core::{BuildContext, ProgramContext, StepResult, TimingReport, VendorAdapter};
use async_trait::async_trait;

/// Efinity vendor adapter.
pub struct EfinityAdapter {
    manifest: AdapterManifest,
    capabilities: CapabilitySet,
}

impl EfinityAdapter {
    /// Create a new Efinity adapter.
    pub fn new() -> Self {
        Self {
            manifest: AdapterManifest {
                id: "com.efinix.efinity".to_string(),
                name: "Efinix Efinity".to_string(),
                version: "2024.2".to_string(),
                vendor: "efinix".to_string(),
                api_version: 1,
                supported_devices: vec!["T8*".to_string(), "T20*".to_string(), "T120*".to_string()],
                release_notes_url: None,
            },
            capabilities: CapabilitySet {
                synthesis: true,
                implementation: true,
                bitstream: true,
                timing_analysis: true,
                on_chip_debug: false,
                device_program: true,
                hls: false,
                simulation: false,
                ip_catalog: true,
            },
        }
    }
}

#[async_trait]
impl VendorAdapter for EfinityAdapter {
    fn manifest(&self) -> &AdapterManifest {
        &self.manifest
    }

    fn capabilities(&self) -> CapabilitySet {
        self.capabilities.clone()
    }

    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Efinity: synthesize");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("efinity/synth"),
            log_path: ctx.project_dir.join("efinity/synth.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Efinity: implement (place & route)");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("efinity/impl"),
            log_path: ctx.project_dir.join("efinity/impl.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Efinity: generate bitstream");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("efinity/bitstream"),
            log_path: ctx.project_dir.join("efinity/bitstream.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Efinity: timing analysis");
        Ok(TimingReport {
            wns: 0.0,
            tns: 0.0,
            whs: 0.0,
            ths: 0.0,
            met: true,
            paths: vec![],
        })
    }

    async fn program_device(&self, ctx: &ProgramContext) -> Result<(), AdapterError> {
        tracing::info!(
            device = %ctx.device,
            bitstream = %ctx.bitstream.display(),
            "Efinity: programming device via SPI"
        );

        let output = std::process::Command::new("efx_pgm")
            .arg("-c")
            .arg("spi")
            .arg("--bitstream")
            .arg(&ctx.bitstream)
            .arg("--device")
            .arg(&ctx.device)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AdapterError::ToolNotFound("efx_pgm".to_string())
                } else {
                    AdapterError::Io(e)
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(%stderr, "Efinity programming failed");
            return Err(AdapterError::BuildFailed {
                exit_code: output.status.code().unwrap_or(-1),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!(%stdout, "Efinity: device programmed successfully");
        Ok(())
    }
}

impl Default for EfinityAdapter {
    fn default() -> Self {
        Self::new()
    }
}