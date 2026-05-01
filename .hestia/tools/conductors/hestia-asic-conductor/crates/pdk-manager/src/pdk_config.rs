//! PDK configuration models

use serde::{Deserialize, Serialize};

/// PDK configuration for a specific process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdkConfig {
    /// PDK identifier.
    pub pdk_id: String,
    /// Display name.
    pub name: String,
    /// Minimum feature size in nanometers.
    pub node_nm: u32,
    /// Default metal layers.
    pub metal_layers: u32,
    /// Standard cell library name.
    pub std_cell_lib: String,
    /// OpenLane compatibility flag.
    pub openlane_compatible: bool,
}

/// Predefined PDK configurations.
pub fn sky130a_config() -> PdkConfig {
    PdkConfig {
        pdk_id: "sky130A".to_string(),
        name: "SkyWater 130nm A".to_string(),
        node_nm: 130,
        metal_layers: 5,
        std_cell_lib: "sky130_fd_sc_hd".to_string(),
        openlane_compatible: true,
    }
}

pub fn gf180mcuc_config() -> PdkConfig {
    PdkConfig {
        pdk_id: "gf180mcuC".to_string(),
        name: "GlobalFoundries 180nm MCU C".to_string(),
        node_nm: 180,
        metal_layers: 5,
        std_cell_lib: "gf180mcu_fd_sc_mcu7t5v0".to_string(),
        openlane_compatible: true,
    }
}

pub fn ihp_sg13g2_config() -> PdkConfig {
    PdkConfig {
        pdk_id: "ihp-sg13g2".to_string(),
        name: "IHP 130nm SG13G2".to_string(),
        node_nm: 130,
        metal_layers: 6,
        std_cell_lib: "sg13g2_stdcell".to_string(),
        openlane_compatible: true,
    }
}