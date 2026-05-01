//! Natural language requirements parser for PCB design

use serde::{Deserialize, Serialize};

/// Parsed requirements from a natural language description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRequirements {
    /// Functional blocks identified.
    pub functional_blocks: Vec<FunctionalBlock>,
    /// Electrical constraints.
    pub electrical_constraints: Vec<ElectricalConstraint>,
    /// Interface requirements.
    pub interfaces: Vec<InterfaceRequirement>,
    /// Package preferences.
    pub package_preferences: Vec<String>,
}

/// A functional block in the design.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalBlock {
    pub name: String,
    pub block_type: BlockType,
    pub description: String,
}

/// Type of functional block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockType {
    Power,
    Processing,
    Memory,
    Communication,
    Analog,
    Sensor,
    Actuator,
}

/// An electrical constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectricalConstraint {
    pub parameter: String,
    pub value: String,
    pub unit: String,
}

/// An interface requirement (e.g., I2C, SPI, UART).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceRequirement {
    pub protocol: String,
    pub role: String,
    pub speed_mbps: Option<f64>,
}

/// Parse a natural language description into structured requirements.
pub fn parse_requirements(description: &str) -> ParsedRequirements {
    tracing::info!(desc_len = description.len(), "parsing requirements from description");
    // Stub: in production, this would use an LLM to extract structured requirements.
    ParsedRequirements {
        functional_blocks: vec![],
        electrical_constraints: vec![],
        interfaces: vec![],
        package_preferences: vec![],
    }
}