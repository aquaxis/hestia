//! Linker script model for embedded firmware

use serde::{Deserialize, Serialize};

/// Linker script representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkerScript {
    /// Path to the linker script file
    pub path: std::path::PathBuf,
    /// Memory regions defined in the script
    #[serde(default)]
    pub memory_regions: Vec<MemoryRegion>,
    /// Section definitions
    #[serde(default)]
    pub sections: Vec<LinkerSection>,
}

/// A memory region in the linker script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    /// Region name (e.g., "FLASH", "RAM")
    pub name: String,
    /// Start origin address
    pub origin: u64,
    /// Region length in bytes
    pub length: u64,
    /// Access attributes (e.g., "rx", "rwx")
    pub attrs: String,
}

/// A section definition in the linker script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkerSection {
    /// Section name (e.g., ".text", ".data", ".bss")
    pub name: String,
    /// Target memory region
    pub region: String,
    /// Section type (e.g., "text", "data", "bss", "rodata")
    pub section_type: String,
}

impl LinkerScript {
    /// Create a new LinkerScript pointing at the given path
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            memory_regions: Vec::new(),
            sections: Vec::new(),
        }
    }

    /// Add a memory region
    pub fn add_memory_region(&mut self, region: MemoryRegion) {
        self.memory_regions.push(region);
    }

    /// Add a section definition
    pub fn add_section(&mut self, section: LinkerSection) {
        self.sections.push(section);
    }

    /// Look up a memory region by name
    pub fn find_region(&self, name: &str) -> Option<&MemoryRegion> {
        self.memory_regions.iter().find(|r| r.name == name)
    }

    /// Calculate total flash size from all executable regions
    pub fn total_flash(&self) -> u64 {
        self.memory_regions
            .iter()
            .filter(|r| r.attrs.contains('x'))
            .map(|r| r.length)
            .sum()
    }

    /// Calculate total RAM size from all writable regions
    pub fn total_ram(&self) -> u64 {
        self.memory_regions
            .iter()
            .filter(|r| r.attrs.contains('w') && !r.attrs.contains('x'))
            .map(|r| r.length)
            .sum()
    }
}