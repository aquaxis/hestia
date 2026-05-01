//! Agent session tracking

use serde::{Deserialize, Serialize};

/// Status of an agent session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Idle,
    Closed,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => "active",
            Self::Idle => "idle",
            Self::Closed => "closed",
        }
        .fmt(f)
    }
}

/// Represents an active (or historical) agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier.
    pub id: String,
    /// The agent this session belongs to.
    pub agent_id: String,
    /// Current session status.
    pub status: SessionStatus,
    /// When the session was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Session {
    /// Create a new session for the given agent.
    pub fn new(id: String, agent_id: String) -> Self {
        Self {
            id,
            agent_id,
            status: SessionStatus::Active,
            created_at: chrono::Utc::now(),
        }
    }

    /// Mark the session as idle.
    pub fn idle(&mut self) {
        self.status = SessionStatus::Idle;
    }

    /// Close the session.
    pub fn close(&mut self) {
        self.status = SessionStatus::Closed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_is_active() {
        let s = Session::new("s-1".into(), "agent-1".into());
        assert_eq!(s.status, SessionStatus::Active);
    }

    #[test]
    fn idle_then_close() {
        let mut s = Session::new("s-1".into(), "agent-1".into());
        s.idle();
        assert_eq!(s.status, SessionStatus::Idle);
        s.close();
        assert_eq!(s.status, SessionStatus::Closed);
    }
}