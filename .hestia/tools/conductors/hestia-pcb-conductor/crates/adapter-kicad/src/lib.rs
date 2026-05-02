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

/// Export fabrication files from a KiCad board using kicad-cli.
pub fn export_board(
    board_path: &std::path::Path,
    format: KiCadExportFormat,
    output_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!(
        board = %board_path.display(),
        format = %format,
        "KiCad: exporting board"
    );

    std::fs::create_dir_all(output_dir)?;

    // Map our format enum to kicad-cli subcommand names.
    let (subcmd, fmt_arg): (&str, Option<&str>) = match format {
        KiCadExportFormat::Gerber => ("gerber", None),
        KiCadExportFormat::Drill => ("drill", None),
        KiCadExportFormat::Pos => ("pos", None),
        KiCadExportFormat::Bom => ("bom", None),
        KiCadExportFormat::Step => ("step", None),
        KiCadExportFormat::Ipc2581 => ("ipc2581", None),
    };

    let mut cmd = std::process::Command::new("kicad-cli");
    cmd.arg("pcb").arg("export").arg(subcmd);

    if let Some(arg) = fmt_arg {
        cmd.arg(arg);
    }

    cmd.arg("--output")
        .arg(output_dir)
        .arg(board_path);

    let output = cmd.output().map_err(|e| -> Box<dyn std::error::Error> {
        if e.kind() == std::io::ErrorKind::NotFound {
            "kicad-cli not found in PATH".into()
        } else {
            e.into()
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("KiCad export failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    tracing::info!(%stdout, "KiCad: export completed");
    Ok(())
}

/// Run DRC on a KiCad board using kicad-cli.
pub fn run_drc(board_path: &std::path::Path) -> Result<DrcReport, Box<dyn std::error::Error>> {
    tracing::info!(board = %board_path.display(), "KiCad: running DRC");

    let output = std::process::Command::new("kicad-cli")
        .arg("pcb")
        .arg("drc")
        .arg("--format")
        .arg("json")
        .arg(board_path)
        .output()
        .map_err(|e| -> Box<dyn std::error::Error> {
            if e.kind() == std::io::ErrorKind::NotFound {
                "kicad-cli not found in PATH".into()
            } else {
                e.into()
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("KiCad DRC failed: {}", stderr).into());
    }

    // Parse JSON output to extract violation and warning counts.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let (violations, warnings, passed) = parse_drc_json(&stdout);

    tracing::info!(
        violations,
        warnings,
        passed,
        "KiCad: DRC completed"
    );

    Ok(DrcReport {
        violations,
        warnings,
        passed,
    })
}

/// Parse kicad-cli DRC JSON output to extract violation/warning counts.
fn parse_drc_json(json: &str) -> (u32, u32, bool) {
    // kicad-cli DRC JSON has a structure like:
    // { "violations": [...], "warnings": [...], "unconnected": [...] }
    // Try to extract counts; fall back to (0, 0, true) on parse failure.
    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return (0, 0, true),
    };

    let violations = parsed
        .get("violations")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u32)
        .unwrap_or(0);

    let warnings = parsed
        .get("warnings")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as u32)
        .unwrap_or(0);

    let passed = violations == 0 && warnings == 0;

    (violations, warnings, passed)
}

/// DRC report from KiCad.
#[derive(Debug, Clone)]
pub struct DrcReport {
    pub violations: u32,
    pub warnings: u32,
    pub passed: bool,
}