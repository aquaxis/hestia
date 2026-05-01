//! pcb-project-model -- pcb.toml parser (re-export from top-level project-model)

pub use project_model::pcb;

/// Parse a pcb.toml string into the configuration model.
pub fn parse_pcb_toml(input: &str) -> Result<pcb::PcbToml, Box<dyn std::error::Error>> {
    let config: pcb::PcbToml = toml::from_str(input)?;
    Ok(config)
}

/// Load and parse a pcb.toml file from disk.
pub fn load_pcb_toml(path: &std::path::Path) -> Result<pcb::PcbToml, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    parse_pcb_toml(&contents)
}