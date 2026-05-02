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

/// Run Yosys synthesis.
pub fn run_synth(
    top_module: &str,
    rtl_files: &[String],
    output_dir: &std::path::Path,
) -> Result<YosysSynthResult, Box<dyn std::error::Error>> {
    tracing::info!(top = %top_module, "Yosys: running synthesis");

    std::fs::create_dir_all(output_dir)?;

    let netlist_path = output_dir.join(format!("{}.v", top_module));
    let log_path = output_dir.join("yosys_synth.log");

    // Build the Yosys synthesis command.
    // Use synth command with -top and -output to produce a netlist.
    let synth_cmd = format!(
        "synth -top {} -flatten; write_verilog {}",
        top_module,
        netlist_path.display()
    );

    let output = std::process::Command::new("yosys")
        .arg("-p")
        .arg(&synth_cmd)
        .arg("-l")
        .arg(&log_path)
        .args(rtl_files)
        .output()
        .map_err(|e| -> Box<dyn std::error::Error> {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!("yosys not found in PATH").into()
            } else {
                e.into()
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Yosys synthesis failed: {}", stderr).into());
    }

    // Parse cell count from the log file.
    let cell_count = parse_cell_count(&log_path).unwrap_or(0);

    let stdout = String::from_utf8_lossy(&output.stdout);
    tracing::info!(%stdout, "Yosys: synthesis completed");

    Ok(YosysSynthResult {
        success: true,
        netlist_path,
        log_path,
        cell_count,
    })
}

/// Parse the number of cells from a Yosys log file.
fn parse_cell_count(log_path: &std::path::Path) -> Option<u64> {
    let contents = std::fs::read_to_string(log_path).ok()?;
    // Yosys prints "Number of cells: <n>" after synthesis.
    for line in contents.lines().rev() {
        if let Some(rest) = line.strip_prefix("   Number of cells:") {
            return rest.trim().parse().ok();
        }
    }
    None
}