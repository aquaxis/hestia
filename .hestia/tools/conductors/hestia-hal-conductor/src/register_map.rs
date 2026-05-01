//! Register map data structures — SystemRDL / IP-XACT derived models

use serde::{Deserialize, Serialize};

/// A single register field within a register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterField {
    /// Field name
    pub name: String,
    /// Bit offset within the register
    pub bit_offset: u32,
    /// Bit width of the field
    pub bit_width: u32,
    /// Access mode: "ro", "wo", "rw", "w1c", "w1s"
    pub access: String,
    /// Reset value for this field
    #[serde(default)]
    pub reset_value: u64,
    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RegisterField {
    /// Bit mask for this field
    pub fn mask(&self) -> u64 {
        ((1u64 << self.bit_width) - 1) << self.bit_offset
    }
}

/// A single register
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterBlock {
    /// Register name
    pub name: String,
    /// Byte address offset from the base
    pub offset: u64,
    /// Register width in bits (typically 32 or 64)
    pub width: u32,
    /// Fields within this register
    #[serde(default)]
    pub fields: Vec<RegisterField>,
    /// Access mode for the whole register
    pub access: String,
    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RegisterBlock {
    /// Total byte size of this register
    pub fn size_bytes(&self) -> u32 {
        self.width / 8
    }
}

/// A complete register map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMap {
    /// Register map name
    pub name: String,
    /// Base address
    pub base_address: u64,
    /// Data bus width in bits
    pub data_width: u32,
    /// Address bus width in bits
    pub addr_width: u32,
    /// Bus protocol
    pub bus_protocol: String,
    /// Register blocks in this map
    #[serde(default)]
    pub registers: Vec<RegisterBlock>,
}

impl RegisterMap {
    /// Create a new empty register map
    pub fn new(name: &str, base_address: u64) -> Self {
        Self {
            name: name.to_string(),
            base_address,
            data_width: 32,
            addr_width: 32,
            bus_protocol: "axi4-lite".to_string(),
            registers: Vec::new(),
        }
    }

    /// Add a register block
    pub fn add_register(&mut self, reg: RegisterBlock) {
        self.registers.push(reg);
    }

    /// Look up a register by name
    pub fn find_register(&self, name: &str) -> Option<&RegisterBlock> {
        self.registers.iter().find(|r| r.name == name)
    }

    /// Total address span consumed by registers
    pub fn address_span(&self) -> u64 {
        self.registers
            .iter()
            .map(|r| r.offset + r.size_bytes() as u64)
            .max()
            .unwrap_or(0)
    }
}