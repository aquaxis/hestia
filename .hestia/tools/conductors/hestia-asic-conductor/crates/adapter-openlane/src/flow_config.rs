//! OpenLane flow configuration

use serde::{Deserialize, Serialize};

/// OpenLane 2 configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLaneConfig {
    /// PDK variant (e.g., "sky130A", "gf180mcuC", "ihp-sg13g2").
    pub pdk: String,
    /// Design name.
    pub design_name: String,
    /// Clock port name.
    pub clock_port: String,
    /// Clock period in nanoseconds.
    pub clock_period_ns: f64,
    /// Target density for placement.
    #[serde(default = "default_density")]
    pub target_density: f64,
    /// Extra configuration overrides.
    #[serde(default)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

fn default_density() -> f64 {
    0.55
}

impl Default for OpenLaneConfig {
    fn default() -> Self {
        Self {
            pdk: "sky130A".to_string(),
            design_name: "top".to_string(),
            clock_port: "clk".to_string(),
            clock_period_ns: 10.0,
            target_density: default_density(),
            extra: std::collections::HashMap::new(),
        }
    }
}

/// Generate an OpenLane 2 JSON configuration file.
pub fn generate_openlane_json(config: &OpenLaneConfig) -> String {
    serde_json::to_string_pretty(config).unwrap_or_default()
}