use serde::{Deserialize, Serialize};

use crate::error::ConstraintFormat;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimingKind {
    MaxDelay,
    MinDelay,
    MultiCycle,
    FalsePath,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClockConstraint {
    pub name: String,
    pub period_ns: f64,
    pub waveform: Option<String>,
    pub target_pins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PinConstraint {
    pub port_name: String,
    pub pin_id: String,
    pub io_standard: Option<String>,
    pub drive_strength: Option<String>,
    pub slew_rate: Option<String>,
    pub differential_pair: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimingConstraint {
    pub kind: TimingKind,
    pub from_clock: String,
    pub to_clock: String,
    pub delay_ns: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlacementConstraint {
    pub instance: String,
    pub site: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawConstraint {
    pub format: ConstraintFormat,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Constraint {
    Clock(ClockConstraint),
    Pin(PinConstraint),
    Timing(TimingConstraint),
    Placement(PlacementConstraint),
    Raw(RawConstraint),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintModel {
    pub constraints: Vec<Constraint>,
    pub source_format: ConstraintFormat,
}