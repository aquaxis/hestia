//! constraint-verifier -- 5-level verification for PCB design constraints

pub mod verifier;

use serde::{Deserialize, Serialize};

/// Verification level (5 levels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// Level 1: Syntax check.
    Syntax = 1,
    /// Level 2: Electrical rule check.
    Electrical = 2,
    /// Level 3: Design rule check.
    DesignRule = 3,
    /// Level 4: Manufacturing constraint check.
    Manufacturing = 4,
    /// Level 5: System-level constraint check.
    System = 5,
}

impl std::fmt::Display for VerificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationLevel::Syntax => write!(f, "L1-Syntax"),
            VerificationLevel::Electrical => write!(f, "L2-Electrical"),
            VerificationLevel::DesignRule => write!(f, "L3-DesignRule"),
            VerificationLevel::Manufacturing => write!(f, "L4-Manufacturing"),
            VerificationLevel::System => write!(f, "L5-System"),
        }
    }
}

/// Result of a single verification check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// The level that was checked.
    pub level: VerificationLevel,
    /// Whether the check passed.
    pub passed: bool,
    /// Number of violations found.
    pub violations: u32,
    /// Details of the violations.
    pub details: Vec<VerificationViolation>,
}

/// A single verification violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationViolation {
    pub code: String,
    pub message: String,
    pub severity: ViolationSeverity,
    pub location: Option<String>,
}

/// Severity of a verification violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Error,
    Warning,
    Info,
}

/// Full 5-level verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub results: Vec<VerificationResult>,
    pub overall_passed: bool,
}

impl VerificationReport {
    /// Check if all levels passed.
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.passed)
    }
}