//! 5-level constraint verifier implementation

use crate::{
    VerificationLevel, VerificationReport, VerificationResult,
};
use std::path::Path;

/// 5-level constraint verifier for PCB designs.
pub struct ConstraintVerifier {
    enabled_levels: Vec<VerificationLevel>,
}

impl ConstraintVerifier {
    /// Create a verifier with all 5 levels enabled.
    pub fn new() -> Self {
        Self {
            enabled_levels: vec![
                VerificationLevel::Syntax,
                VerificationLevel::Electrical,
                VerificationLevel::DesignRule,
                VerificationLevel::Manufacturing,
                VerificationLevel::System,
            ],
        }
    }

    /// Create a verifier with only specific levels enabled.
    pub fn with_levels(levels: Vec<VerificationLevel>) -> Self {
        Self {
            enabled_levels: levels,
        }
    }

    /// Run all enabled verification levels on a project directory.
    pub fn verify(&self, project_dir: &Path) -> VerificationReport {
        tracing::info!(path = %project_dir.display(), "running 5-level constraint verification");
        let mut results = Vec::new();
        for &level in &self.enabled_levels {
            results.push(self.verify_level(project_dir, level));
        }
        let overall_passed = results.iter().all(|r| r.passed);
        VerificationReport {
            results,
            overall_passed,
        }
    }

    fn verify_level(&self, project_dir: &Path, level: VerificationLevel) -> VerificationResult {
        tracing::info!(level = %level, "running verification level");

        let mut details = Vec::new();

        match level {
            VerificationLevel::Syntax => self.check_syntax(project_dir, &mut details),
            VerificationLevel::Electrical => self.check_electrical(project_dir, &mut details),
            VerificationLevel::DesignRule => self.check_design_rules(project_dir, &mut details),
            VerificationLevel::Manufacturing => self.check_manufacturing(project_dir, &mut details),
            VerificationLevel::System => self.check_system(project_dir, &mut details),
        }

        let violations = details.len() as u32;
        let passed = details.iter().all(|v| v.severity != crate::ViolationSeverity::Error);

        VerificationResult {
            level,
            passed,
            violations,
            details,
        }
    }

    /// Level 1: Syntax check -- verify KiCad files are well-formed S-expressions.
    fn check_syntax(&self, project_dir: &Path, details: &mut Vec<crate::VerificationViolation>) {
        let kicad_files = self.collect_kicad_files(project_dir);

        if kicad_files.is_empty() {
            details.push(crate::VerificationViolation {
                code: "SYN-001".to_string(),
                message: "No KiCad schematic or PCB files found in project directory".to_string(),
                severity: crate::ViolationSeverity::Warning,
                location: Some(project_dir.display().to_string()),
            });
            return;
        }

        for file_path in &kicad_files {
            match std::fs::read_to_string(file_path) {
                Ok(content) => {
                    // Basic S-expression validation: must start with '(' and end with ')'
                    let trimmed = content.trim();
                    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
                        details.push(crate::VerificationViolation {
                            code: "SYN-002".to_string(),
                            message: format!("File does not appear to be a valid S-expression: {}", file_path.display()),
                            severity: crate::ViolationSeverity::Error,
                            location: Some(file_path.display().to_string()),
                        });
                    }

                    // Check for balanced parentheses
                    let depth = trimmed.chars().fold(0i32, |acc, c| {
                        if acc < 0 { acc } else { acc + if c == '(' { 1 } else if c == ')' { -1 } else { 0 } }
                    });
                    if depth != 0 {
                        details.push(crate::VerificationViolation {
                            code: "SYN-003".to_string(),
                            message: format!("Unbalanced parentheses (depth={}) in: {}", depth, file_path.display()),
                            severity: crate::ViolationSeverity::Error,
                            location: Some(file_path.display().to_string()),
                        });
                    }

                    // Check for required KiCad header tokens
                    if !content.contains("kicad_sch") && !content.contains("kicad_pcb") && !content.contains("kicad_sym") && !content.contains("kicad_mod") {
                        details.push(crate::VerificationViolation {
                            code: "SYN-004".to_string(),
                            message: format!("Missing KiCad header token in: {}", file_path.display()),
                            severity: crate::ViolationSeverity::Warning,
                            location: Some(file_path.display().to_string()),
                        });
                    }
                }
                Err(e) => {
                    details.push(crate::VerificationViolation {
                        code: "SYN-005".to_string(),
                        message: format!("Cannot read file {}: {}", file_path.display(), e),
                        severity: crate::ViolationSeverity::Error,
                        location: Some(file_path.display().to_string()),
                    });
                }
            }
        }
    }

    /// Level 2: Electrical rule check -- clearance rules between nets.
    fn check_electrical(&self, project_dir: &Path, details: &mut Vec<crate::VerificationViolation>) {
        let kicad_files = self.collect_kicad_files(project_dir);

        // Default minimum clearance in mm (IPC-2221 generic)
        const MIN_CLEARANCE_MM: f64 = 0.15;

        for file_path in &kicad_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                // Look for clearance / spacing definitions in the file
                let lower = content.to_ascii_lowercase();

                // Check for explicit clearance constraints
                self.extract_clearance_violations(&lower, MIN_CLEARANCE_MM, file_path, details);

                // Check for unconnected pins (lines with "unconnected" flag)
                if lower.contains("unconnected") {
                    // Count unconnected pins
                    let count = lower.matches("unconnected").count();
                    details.push(crate::VerificationViolation {
                        code: "ELC-001".to_string(),
                        message: format!("Found {} unconnected pin(s) in {}", count, file_path.display()),
                        severity: crate::ViolationSeverity::Warning,
                        location: Some(file_path.display().to_string()),
                    });
                }

                // Check for power pins without nets
                if lower.contains("power_in") && !lower.contains("power_out") {
                    // Look for power input pins that might lack a source
                    let power_in_count = lower.matches("power_in").count();
                    if power_in_count > 0 {
                        details.push(crate::VerificationViolation {
                            code: "ELC-002".to_string(),
                            message: format!("Found {} power input pin(s) but no power output pins in {}", power_in_count, file_path.display()),
                            severity: crate::ViolationSeverity::Info,
                            location: Some(file_path.display().to_string()),
                        });
                    }
                }
            }
        }

        // If no files found, emit informational notice
        if kicad_files.is_empty() {
            details.push(crate::VerificationViolation {
                code: "ELC-003".to_string(),
                message: "No KiCad files found; electrical check could not be performed".to_string(),
                severity: crate::ViolationSeverity::Info,
                location: Some(project_dir.display().to_string()),
            });
        }
    }

    /// Level 3: Design rule check -- trace width rules.
    fn check_design_rules(&self, project_dir: &Path, details: &mut Vec<crate::VerificationViolation>) {
        let kicad_files = self.collect_kicad_files(project_dir);

        // Default minimum trace width in mm (IPC-2221 for 0.5A on 1oz copper)
        const MIN_TRACE_WIDTH_MM: f64 = 0.15;
        // Default minimum trace width for power traces
        const MIN_POWER_TRACE_WIDTH_MM: f64 = 0.30;

        for file_path in &kicad_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let lower = content.to_ascii_lowercase();

                // Look for trace/line width definitions in PCB files
                self.extract_trace_width_violations(&lower, MIN_TRACE_WIDTH_MM, MIN_POWER_TRACE_WIDTH_MM, file_path, details);

                // Check for via definitions and sizes
                self.check_via_sizes(&lower, file_path, details);

                // Check for pad sizes
                self.check_pad_sizes(&lower, file_path, details);
            }
        }

        if kicad_files.is_empty() {
            details.push(crate::VerificationViolation {
                code: "DRC-003".to_string(),
                message: "No KiCad files found; design rule check could not be performed".to_string(),
                severity: crate::ViolationSeverity::Info,
                location: Some(project_dir.display().to_string()),
            });
        }
    }

    /// Level 4: Manufacturing constraint check.
    fn check_manufacturing(&self, project_dir: &Path, details: &mut Vec<crate::VerificationViolation>) {
        let kicad_files = self.collect_kicad_files(project_dir);

        // Default minimum drill size in mm
        const MIN_DRILL_MM: f64 = 0.20;
        // Default minimum silkscreen text height in mm
        const MIN_SILK_TEXT_MM: f64 = 0.80;

        for file_path in &kicad_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let lower = content.to_ascii_lowercase();

                // Check drill sizes
                self.check_drill_sizes(&lower, MIN_DRILL_MM, file_path, details);

                // Check for board outline / edge cuts
                if !lower.contains("edge_cuts") && !lower.contains("edgecuts") && content.contains("kicad_pcb") {
                    details.push(crate::VerificationViolation {
                        code: "MFG-001".to_string(),
                        message: format!("No board outline (Edge.Cuts) defined in {}", file_path.display()),
                        severity: crate::ViolationSeverity::Error,
                        location: Some(file_path.display().to_string()),
                    });
                }

                // Check for silkscreen text size
                self.check_silkscreen_sizes(&lower, MIN_SILK_TEXT_MM, file_path, details);
            }
        }

        if kicad_files.is_empty() {
            details.push(crate::VerificationViolation {
                code: "MFG-002".to_string(),
                message: "No KiCad files found; manufacturing check could not be performed".to_string(),
                severity: crate::ViolationSeverity::Info,
                location: Some(project_dir.display().to_string()),
            });
        }
    }

    /// Level 5: System-level constraint check -- impedance matching requirements.
    fn check_system(&self, project_dir: &Path, details: &mut Vec<crate::VerificationViolation>) {
        let kicad_files = self.collect_kicad_files(project_dir);

        // Target impedance for high-speed signals (ohms)
        const TARGET_IMPEDANCE: f64 = 50.0;
        const IMPEDANCE_TOLERANCE_PCT: f64 = 10.0;

        for file_path in &kicad_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let lower = content.to_ascii_lowercase();

                // Check for impedance-controlled nets (USB, Ethernet, HDMI, DDR, RF)
                let high_speed_protocols = ["usb", "ethernet", "hdmi", "ddr", "pcie", "sata", "rf", "antenna", "microwave"];
                for proto in &high_speed_protocols {
                    if lower.contains(proto) {
                        // Check if impedance constraints are defined
                        if !lower.contains("impedance") && !lower.contains("diff_pair") && !lower.contains("diffpair") {
                            details.push(crate::VerificationViolation {
                                code: "SYS-001".to_string(),
                                message: format!(
                                    "High-speed protocol '{}' found but no impedance constraints defined in {}",
                                    proto,
                                    file_path.display()
                                ),
                                severity: crate::ViolationSeverity::Warning,
                                location: Some(file_path.display().to_string()),
                            });
                        }

                        // Check for length-matching constraints on differential pairs
                        if lower.contains("diff_pair") || lower.contains("differential") {
                            if !lower.contains("length_match") && !lower.contains("lengthmatch") && !lower.contains("skew") {
                                details.push(crate::VerificationViolation {
                                    code: "SYS-002".to_string(),
                                    message: format!(
                                        "Differential pair found for '{}' without length-matching constraints in {}",
                                        proto,
                                        file_path.display()
                                    ),
                                    severity: crate::ViolationSeverity::Info,
                                    location: Some(file_path.display().to_string()),
                                });
                            }
                        }
                    }
                }

                // Verify defined impedance values are within tolerance
                self.check_impedance_values(&lower, TARGET_IMPEDANCE, IMPEDANCE_TOLERANCE_PCT, file_path, details);
            }
        }

        if kicad_files.is_empty() {
            details.push(crate::VerificationViolation {
                code: "SYS-003".to_string(),
                message: "No KiCad files found; system-level check could not be performed".to_string(),
                severity: crate::ViolationSeverity::Info,
                location: Some(project_dir.display().to_string()),
            });
        }
    }

    /// Collect all KiCad files (.kicad_sch, .kicad_pcb, .kicad_sym, .kicad_mod) from a directory.
    fn collect_kicad_files(&self, project_dir: &Path) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(project_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if matches!(ext, "kicad_sch" | "kicad_pcb" | "kicad_sym" | "kicad_mod") {
                        files.push(path);
                    }
                }
            }
        }
        files
    }

    /// Check clearance values extracted from S-expression content.
    fn extract_clearance_violations(
        &self,
        lower: &str,
        min_clearance_mm: f64,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        // Look for (clearance X) patterns where X < min_clearance_mm
        if let Ok(re) = regex::Regex::new(r"clearance\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        if val > 0.0 && val < min_clearance_mm {
                            details.push(crate::VerificationViolation {
                                code: "ELC-004".to_string(),
                                message: format!(
                                    "Clearance {:.4}mm is below minimum {:.4}mm",
                                    val, min_clearance_mm
                                ),
                                severity: crate::ViolationSeverity::Error,
                                location: Some(file_path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Check trace width values against minimum rules.
    fn extract_trace_width_violations(
        &self,
        lower: &str,
        min_trace_mm: f64,
        min_power_trace_mm: f64,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        // Look for (width X) patterns in PCB files
        if let Ok(re) = regex::Regex::new(r"width\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        if val > 0.0 && val < min_trace_mm {
                            details.push(crate::VerificationViolation {
                                code: "DRC-001".to_string(),
                                message: format!(
                                    "Trace width {:.4}mm is below minimum {:.4}mm",
                                    val, min_trace_mm
                                ),
                                severity: crate::ViolationSeverity::Error,
                                location: Some(file_path.display().to_string()),
                            });
                        } else if val > 0.0 && val < min_power_trace_mm {
                            // Check if this could be a power trace
                            let ctx_start = m.start().saturating_sub(80);
                            let context = &lower[ctx_start..m.end().min(lower.len())];
                            if context.contains("power") || context.contains("vcc") || context.contains("gnd") || context.contains("vdd") {
                                details.push(crate::VerificationViolation {
                                    code: "DRC-002".to_string(),
                                    message: format!(
                                        "Power trace width {:.4}mm is below recommended minimum {:.4}mm",
                                        val, min_power_trace_mm
                                    ),
                                    severity: crate::ViolationSeverity::Warning,
                                    location: Some(file_path.display().to_string()),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check via sizes against manufacturing minimums.
    fn check_via_sizes(
        &self,
        lower: &str,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        const MIN_VIA_DRILL_MM: f64 = 0.20;
        // Look for (via (at ...) (size X) (drill Y)) patterns
        if let Ok(re) = regex::Regex::new(r"via\s+.*?drill\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        if val > 0.0 && val < MIN_VIA_DRILL_MM {
                            details.push(crate::VerificationViolation {
                                code: "DRC-004".to_string(),
                                message: format!(
                                    "Via drill {:.4}mm is below minimum {:.4}mm",
                                    val, MIN_VIA_DRILL_MM
                                ),
                                severity: crate::ViolationSeverity::Error,
                                location: Some(file_path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Check pad sizes.
    fn check_pad_sizes(
        &self,
        lower: &str,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        const MIN_PAD_SIZE_MM: f64 = 0.20;
        // Look for (pad ... (size X Y)) patterns
        if let Ok(re) = regex::Regex::new(r"pad\s+.*?size\s+([\d.]+)\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let (Some(mx), Some(my)) = (cap.get(1), cap.get(2)) {
                    if let (Ok(x), Ok(y)) = (mx.as_str().parse::<f64>(), my.as_str().parse::<f64>()) {
                        let min_dim = x.min(y);
                        if min_dim > 0.0 && min_dim < MIN_PAD_SIZE_MM {
                            details.push(crate::VerificationViolation {
                                code: "DRC-005".to_string(),
                                message: format!(
                                    "Pad size {:.4}x{:.4}mm has a dimension below minimum {:.4}mm",
                                    x, y, MIN_PAD_SIZE_MM
                                ),
                                severity: crate::ViolationSeverity::Warning,
                                location: Some(file_path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Check drill sizes against manufacturing minimums.
    fn check_drill_sizes(
        &self,
        lower: &str,
        min_drill_mm: f64,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        if let Ok(re) = regex::Regex::new(r"drill\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        if val > 0.0 && val < min_drill_mm {
                            details.push(crate::VerificationViolation {
                                code: "MFG-003".to_string(),
                                message: format!(
                                    "Drill size {:.4}mm is below minimum {:.4}mm",
                                    val, min_drill_mm
                                ),
                                severity: crate::ViolationSeverity::Error,
                                location: Some(file_path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Check silkscreen text sizes.
    fn check_silkscreen_sizes(
        &self,
        lower: &str,
        min_text_mm: f64,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        // Look for (fp_text ... (size X Y)) in silkscreen layer
        if let Ok(re) = regex::Regex::new(r"fp_text\s+.*?size\s+([\d.]+)\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let (Some(mx), Some(my)) = (cap.get(1), cap.get(2)) {
                    if let (Ok(x), Ok(y)) = (mx.as_str().parse::<f64>(), my.as_str().parse::<f64>()) {
                        let height = y.min(x); // text height is typically the smaller dimension
                        if height > 0.0 && height < min_text_mm {
                            details.push(crate::VerificationViolation {
                                code: "MFG-004".to_string(),
                                message: format!(
                                    "Silkscreen text size {:.4}x{:.4}mm is below minimum height {:.4}mm",
                                    x, y, min_text_mm
                                ),
                                severity: crate::ViolationSeverity::Warning,
                                location: Some(file_path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Check impedance values against target with tolerance.
    fn check_impedance_values(
        &self,
        lower: &str,
        target_impedance: f64,
        tolerance_pct: f64,
        file_path: &Path,
        details: &mut Vec<crate::VerificationViolation>,
    ) {
        // Look for (impedance X) patterns
        if let Ok(re) = regex::Regex::new(r"impedance\s+([\d.]+)") {
            for cap in re.captures_iter(lower) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f64>() {
                        let deviation = (val - target_impedance).abs() / target_impedance * 100.0;
                        if deviation > tolerance_pct {
                            details.push(crate::VerificationViolation {
                                code: "SYS-004".to_string(),
                                message: format!(
                                    "Impedance {:.1} ohms deviates {:.1}% from target {:.1} ohms (tolerance {:.1}%)",
                                    val, deviation, target_impedance, tolerance_pct
                                ),
                                severity: crate::ViolationSeverity::Warning,
                                location: Some(file_path.display().to_string()),
                            });
                        }
                    }
                }
            }
        }
    }
}

impl Default for ConstraintVerifier {
    fn default() -> Self {
        Self::new()
    }
}