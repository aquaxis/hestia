//! adapter-yosys -- Yosys logic synthesis adapter

pub mod synth_script;

use serde::{Deserialize, Serialize};

/// Yosys synthesis strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SynthStrategy {
    /// Optimize for area.
    Area,
    /// Optimize for speed.
    Speed,
    /// Balanced area/speed.
    Balanced,
}

impl Default for SynthStrategy {
    fn default() -> Self {
        SynthStrategy::Balanced
    }
}

/// Yosys adapter for logic synthesis.
pub struct YosysAdapter {
    pub yosys_path: String,
    pub strategy: SynthStrategy,
}

impl YosysAdapter {
    /// Create a new Yosys adapter.
    pub fn new(yosys_path: &str) -> Self {
        Self {
            yosys_path: yosys_path.to_string(),
            strategy: SynthStrategy::default(),
        }
    }

    /// Create a Yosys adapter with a specific strategy.
    pub fn with_strategy(yosys_path: &str, strategy: SynthStrategy) -> Self {
        Self {
            yosys_path: yosys_path.to_string(),
            strategy,
        }
    }
}

/// Result of a Yosys synthesis run.
#[derive(Debug, Clone)]
pub struct YosysSynthResult {
    pub success: bool,
    pub netlist_path: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub cell_count: u64,
}

/// Run Yosys synthesis (stub).
pub fn run_synth(
    top_module: &str,
    _rtl_files: &[String],
    output_dir: &std::path::Path,
) -> Result<YosysSynthResult, Box<dyn std::error::Error>> {
    tracing::info!(top = %top_module, "Yosys: running synthesis");
    Ok(YosysSynthResult {
        success: true,
        netlist_path: output_dir.join(format!("{}.v", top_module)),
        log_path: output_dir.join("yosys_synth.log"),
        cell_count: 0,
    })
}