//! Debug session state machine and session types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Debug session state machine.
///
/// Transitions:
///   Idle -> Connecting -> Connected -> Running <-> Paused
///   Running -> Capturing -> Running
///   Any -> Disconnected
///   Any -> Error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    Idle,
    Connecting,
    Connected,
    Running,
    Paused,
    Capturing,
    Disconnected,
    Error,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => "idle",
            Self::Connecting => "connecting",
            Self::Connected => "connected",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Capturing => "capturing",
            Self::Disconnected => "disconnected",
            Self::Error => "error",
        }
        .fmt(f)
    }
}

/// Session state machine with transition validation.
#[derive(Debug)]
pub struct SessionStateMachine {
    state: SessionState,
}

impl SessionStateMachine {
    /// Create a new state machine starting at `Idle`.
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
        }
    }

    /// Return the current state.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Attempt a transition to `next`. Returns `Ok(())` on success or
    /// `Err(current)` if the transition is invalid.
    pub fn transition(&mut self, next: SessionState) -> Result<(), SessionState> {
        if self.is_valid_transition(next) {
            self.state = next;
            Ok(())
        } else {
            Err(self.state)
        }
    }

    /// Check whether transitioning to `next` is allowed from the current state.
    fn is_valid_transition(&self, next: SessionState) -> bool {
        use SessionState::*;
        match (self.state, next) {
            // Normal forward flow
            (Idle, Connecting) => true,
            (Connecting, Connected) => true,
            (Connecting, Error) => true,
            (Connected, Running) => true,
            (Connected, Disconnected) => true,
            (Connected, Error) => true,
            (Running, Paused) => true,
            (Running, Capturing) => true,
            (Running, Disconnected) => true,
            (Running, Error) => true,
            (Paused, Running) => true,
            (Paused, Disconnected) => true,
            (Paused, Error) => true,
            (Capturing, Running) => true,
            (Capturing, Disconnected) => true,
            (Capturing, Error) => true,
            // Any state may transition to Disconnected or Error
            (_, Disconnected) => true,
            (_, Error) => true,
            _ => false,
        }
    }
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Reset type for debug targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResetType {
    /// Hardware reset (via reset pin)
    Hardware,
    /// Software reset (via debug register)
    Software,
    /// System reset (full system restart)
    System,
}

impl std::fmt::Display for ResetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hardware => "hardware",
            Self::Software => "software",
            Self::System => "system",
        }
        .fmt(f)
    }
}

/// A debug session bound to a specific device and interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    /// Unique session identifier.
    pub session_id: String,
    /// Target device identifier (e.g. "stm32f407", "xc7z010").
    pub device: String,
    /// Debug interface in use (e.g. "jtag", "swd").
    pub interface: String,
}

impl DebugSession {
    /// Create a new debug session with a random UUID.
    pub fn new(device: impl Into<String>, interface: impl Into<String>) -> Self {
        Self {
            session_id: format!("dbg_{}", Uuid::new_v4()),
            device: device.into(),
            interface: interface.into(),
        }
    }
}