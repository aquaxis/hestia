//! adapter-kicad -- KiCad EDA integration adapter

pub mod kicad_format;

use serde::{Deserialize, Serialize};

/// KiCad adapter for PCB design.
pub struct KiCadAdapter {
    kicad_cli_path: String,
}

impl KiCadAdapter {
    /// Create a new KiCad adapter with the given CLI path.
    pub fn new(kicad_cli_path: &str) -> Self {
        Self {
            kicad_cli_path: kicad_cli_path.to_string(),
        }
    }

    /// Create a default KiCad adapter (assumes kicad is in PATH).
    pub fn default_adapter() -> Self {
        Self {
            kicad_cli_path: "kicad-cli".to_string(),
        }
    }

    /// Get the KiCad CLI path.
    pub fn cli_path(&self) -> &str {
        &self.kicad_cli_path
    }
}

/// KiCad project info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiCadProject {
    pub name: String,
    pub schematic_path: Option<std::path::PathBuf>,
    pub board_path: Option<std::path::PathBuf>,
}

/// KiCad export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KiCadExportFormat {
    Gerber,
    Drill,
    Pos,
    Bom,
    Step,
    Ipc2581,
}

impl std::fmt::Display for KiCadExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KiCadExportFormat::Gerber => write!(f, "gerber"),
            KiCadExportFormat::Drill => write!(f, "drill"),
            KiCadExportFormat::Pos => write!(f, "pos"),
            KiCadExportFormat::Bom => write!(f, "bom"),
            KiCadExportFormat::Step => write!(f, "step"),
            KiCadExportFormat::Ipc2581 => write!(f, "ipc2581"),
        }
    }
}

/// Export fabrication files from a KiCad board (stub).
pub fn export_board(
    board_path: &std::path::Path,
    format: KiCadExportFormat,
    _output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        board = %board_path.display(),
        format = %format,
        "KiCad: exporting board"
    );
    Ok(())
}

/// Run DRC on a KiCad board (stub).
pub fn run_drc(board_path: &std::path::Path) -> Result<DrcReport, Box<dyn std::error::Error>> {
    tracing::info!(board = %board_path.display(), "KiCad: running DRC");
    Ok(DrcReport {
        violations: 0,
        warnings: 0,
        passed: true,
    })
}

/// DRC report from KiCad.
#[derive(Debug, Clone)]
pub struct DrcReport {
    pub violations: u32,
    pub warnings: u32,
    pub passed: bool,
}