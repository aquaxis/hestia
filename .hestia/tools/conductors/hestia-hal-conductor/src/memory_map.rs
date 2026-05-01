//! Memory map validation — overlap detection and address range checking

use crate::register_map::RegisterBlock;

/// Memory map with validation support
#[derive(Debug, Clone)]
pub struct MemoryMap {
    /// Name of the memory map
    pub name: String,
    /// Base address of the memory-mapped region
    pub base_address: u64,
    /// Total size in bytes
    pub size: u64,
    /// Memory regions (register blocks with absolute addresses)
    pub regions: Vec<MemoryRegion>,
}

/// A contiguous region in the memory map
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Region name
    pub name: String,
    /// Start address (absolute)
    pub start: u64,
    /// Size in bytes
    pub size: u64,
    /// Access mode: "ro", "wo", "rw"
    pub access: String,
}

impl MemoryRegion {
    /// End address (exclusive)
    pub fn end(&self) -> u64 {
        self.start + self.size
    }
}

/// Validation error for address overlaps
#[derive(Debug)]
pub struct OverlapError {
    pub region_a: String,
    pub region_b: String,
    pub overlap_start: u64,
    pub overlap_end: u64,
}

impl std::fmt::Display for OverlapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "address overlap between '{}' and '{}' at 0x{:08X}..0x{:08X}",
            self.region_a, self.region_b, self.overlap_start, self.overlap_end
        )
    }
}

impl std::error::Error for OverlapError {}

impl MemoryMap {
    /// Create a new empty memory map
    pub fn new(name: &str, base_address: u64, size: u64) -> Self {
        Self {
            name: name.to_string(),
            base_address,
            size,
            regions: Vec::new(),
        }
    }

    /// Add a register block as a memory region
    pub fn add_register_block(&mut self, block: &RegisterBlock) {
        let region = MemoryRegion {
            name: block.name.clone(),
            start: self.base_address + block.offset,
            size: block.size_bytes() as u64,
            access: block.access.clone(),
        };
        self.regions.push(region);
    }

    /// Validate that no regions have overlapping addresses
    pub fn validate_addresses(&self) -> Result<(), Vec<OverlapError>> {
        let mut errors = Vec::new();
        for i in 0..self.regions.len() {
            for j in (i + 1)..self.regions.len() {
                let a = &self.regions[i];
                let b = &self.regions[j];
                if a.start < b.end() && b.start < a.end() {
                    let overlap_start = a.start.max(b.start);
                    let overlap_end = a.end().min(b.end());
                    errors.push(OverlapError {
                        region_a: a.name.clone(),
                        region_b: b.name.clone(),
                        overlap_start,
                        overlap_end,
                    });
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}