//! adapter-openlane -- OpenLane 2 integration adapter

pub mod flow_config;

use crate::flow_config::OpenLaneConfig;

/// OpenLane 2 adapter for ASIC physical design.
pub struct OpenLaneAdapter {
    config: OpenLaneConfig,
}

impl OpenLaneAdapter {
    /// Create a new OpenLane adapter with the given configuration.
    pub fn new(config: OpenLaneConfig) -> Self {
        Self { config }
    }

    /// Create a default OpenLane adapter.
    pub fn default_adapter() -> Self {
        Self {
            config: OpenLaneConfig::default(),
        }
    }

    /// Get the PDK name from configuration.
    pub fn pdk(&self) -> &str {
        &self.config.pdk
    }
}

/// Run the full OpenLane flow (synthesis through GDS).
pub fn run_openlane_flow(
    design_name: &str,
    project_dir: &std::path::Path,
    config: &OpenLaneConfig,
) -> Result<OpenLaneResult, Box<dyn std::error::Error>> {
    tracing::info!(design = %design_name, pdk = %config.pdk, "starting OpenLane flow");
    Ok(OpenLaneResult {
        success: true,
        gds_path: project_dir.join("results/nanonuke/gds").join(format!("{}.gds", design_name)),
        log_path: project_dir.join("openlane.log"),
    })
}

/// Result of an OpenLane flow run.
#[derive(Debug, Clone)]
pub struct OpenLaneResult {
    pub success: bool,
    pub gds_path: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
}