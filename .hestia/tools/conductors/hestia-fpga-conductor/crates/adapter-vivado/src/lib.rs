//! adapter-vivado -- Vivado TCL template adapter

pub mod tcl_templates;

use adapter_core::manifest::AdapterManifest;
use adapter_core::capability::CapabilitySet;
use adapter_core::error::AdapterError;
use adapter_core::{BuildContext, ProgramContext, SimContext, SimResult, StepResult, TimingReport, VendorAdapter};
use async_trait::async_trait;

/// Vivado vendor adapter.
pub struct VivadoAdapter {
    manifest: AdapterManifest,
    capabilities: CapabilitySet,
}

impl VivadoAdapter {
    /// Create a new Vivado adapter.
    pub fn new() -> Self {
        Self {
            manifest: AdapterManifest {
                id: "com.xilinx.vivado".to_string(),
                name: "AMD Vivado".to_string(),
                version: "2024.1".to_string(),
                vendor: "xilinx".to_string(),
                api_version: 1,
                supported_devices: vec!["xc7*".to_string(), "xzu*".to_string(), "xcvu*".to_string()],
                release_notes_url: None,
            },
            capabilities: CapabilitySet {
                synthesis: true,
                implementation: true,
                bitstream: true,
                timing_analysis: true,
                on_chip_debug: true,
                device_program: true,
                hls: true,
                simulation: true,
                ip_catalog: true,
            },
        }
    }
}

#[async_trait]
impl VendorAdapter for VivadoAdapter {
    fn manifest(&self) -> &AdapterManifest {
        &self.manifest
    }

    fn capabilities(&self) -> CapabilitySet {
        self.capabilities.clone()
    }

    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Vivado: synthesize");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("vivado/synth"),
            log_path: ctx.project_dir.join("vivado/synth.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Vivado: implement");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("vivado/impl"),
            log_path: ctx.project_dir.join("vivado/impl.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Vivado: generate bitstream");
        Ok(StepResult {
            success: true,
            output_dir: ctx.project_dir.join("vivado/bitstream"),
            log_path: ctx.project_dir.join("vivado/bitstream.log"),
            duration_secs: 0.0,
            diagnostics: vec![],
        })
    }

    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError> {
        tracing::info!(project = %ctx.project_dir.display(), "Vivado: timing analysis");
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
        tracing::info!("Vivado: program device (stub)");
        Ok(())
    }

    async fn simulate(&self, ctx: &SimContext) -> Result<SimResult, AdapterError> {
        tracing::info!(tb = %ctx.testbench, "Vivado: simulate");
        Ok(SimResult {
            passed: true,
            vcd_path: Some(ctx.work_dir.join("wave.vcd")),
            log_path: ctx.work_dir.join("simulate.log"),
            duration_secs: 0.0,
        })
    }
}

impl Default for VivadoAdapter {
    fn default() -> Self {
        Self::new()
    }
}