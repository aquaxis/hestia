//! fpga-project-model -- fpga.toml parser (re-export from top-level project-model)

pub use project_model::fpga;

/// Parse a fpga.toml string into the configuration model.
pub fn parse_fpga_toml(input: &str) -> Result<fpga::FpgaToml, Box<dyn std::error::Error>> {
    let config: fpga::FpgaToml = toml::from_str(input)?;
    Ok(config)
}

/// Load and parse a fpga.toml file from disk.
pub fn load_fpga_toml(path: &std::path::Path) -> Result<fpga::FpgaToml, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    parse_fpga_toml(&contents)
}