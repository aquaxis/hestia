//! adapter-quartus -- Quartus QSF/QIP adapter

pub mod qsf_templates;

use adapter_core::manifest::AdapterManifest;
use adapter_core::capability::CapabilitySet;
use adapter_core::error::AdapterError;
use adapter_core::{BuildContext, ProgramContext, SimContext, SimResult, StepResult, TimingReport, VendorAdapter};
use async_trait::async_trait;

/// Intel Quartus vendor adapter.
pub struct QuartusAdapter {
    manifest: AdapterManifest,
    capabilities: CapabilitySet,
}

impl QuartusAdapter {
    /// Create a new Quartus adapter.
    pub fn new() -> Self {
        Self {
            manifest: AdapterManifest {
                id: "com.intel.quartus".to_string(),
                name: "Intel Quartus".to_string(),
                version: "24.1".to_string(),
                vendor: "intel".to_string(),
                api_version: 1,
                supported_devices: vec!["5CE*".to_string(), "10M*".to_string(), "AGFB*".to_string()],
                release_notes_url: None,
            },
            capabilities: CapabilitySet {
                synthesis: true,
                implementation: true,
                bitstream: true,
                timing_analysis: true,
                on_chip_debug: true,
                device_program: true,
                hls: false,
                simulation: true,
                ip_catalog: true,
            },
        }
    }
}

#[async_trait]
impl VendorAdapter for QuartusAdapter {
    fn manifest(&self) -> &AdapterManifest {
        &self.manifest
    }

    fn capabilities(&self) -> CapabilitySet {
        self.capabilities.clone()
    }

    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Quartus: synthesize");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("quartus/synth"),
            log_path: ctx.project_dir.join("quartus/synth.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Quartus: implement");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("quartus/impl"),
            log_path: ctx.project_dir.join("quartus/impl.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Quartus: generate bitstream (SOFE)");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("quartus/output_files"),
            log_path: ctx.project_dir.join("quartus/bitstream.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Quartus: timing analysis (TimeQuest)");
        Ok(TimingReport {
            wns: 0.0,
            tns: 0.0,
            whs: 0.0,
            ths: 0.0,
            met: true,
            paths: vec![],
        })
    }

    async fn program_device(&self, _ctx: &ProgramContext) -> Result<(), AdapterError> {
        tracing::info!("Quartus: program device via JTAG (stub)");
        Ok(())
    }

    async fn simulate(&self, ctx: &SimContext) -> Result<SimResult, AdapterError> {
        tracing::info!(tb = %ctx.testbench, "Quartus: simulate (ModelSim)");
        Ok(SimResult {
            passed: true,
            vcd_path: Some(ctx.work_dir.join("wave.vcd")),
            log_path: ctx.work_dir.join("simulate.log"),
            duration_secs: 0.0,
        })
    }
}

impl Default for QuartusAdapter {
    fn default() -> Self {
        Self::new()
    }
}