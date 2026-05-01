//! asic-project-model -- asic.toml parser (re-export from top-level project-model)

pub use project_model::asic;

/// Parse an asic.toml string into the configuration model.
pub fn parse_asic_toml(input: &str) -> Result<asic::AsicToml, Box<dyn std::error::Error>> {
    let config: asic::AsicToml = toml::from_str(input)?;
    Ok(config)
}

/// Load and parse an asic.toml file from disk.
pub fn load_asic_toml(path: &std::path::Path) -> Result<asic::AsicToml, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    parse_asic_toml(&contents)
}