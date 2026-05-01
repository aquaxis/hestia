//! Live observation of agent activity

use serde::{Deserialize, Serialize};
use tracing::debug;

/// Snapshot of an agent's live state, suitable for dashboard rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub agent_id: String,
    pub status: String,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
    pub message_count: u64,
}

/// Provides a real-time view of agent activity.
///
/// In production this would stream events via WebSocket or SSE.  The current
/// implementation maintains an in-memory map of the latest snapshot per agent.
#[derive(Debug)]
pub struct LiveView {
    snapshots: std::collections::HashMap<String, AgentSnapshot>,
}

impl LiveView {
    /// Create a new empty live view.
    pub fn new() -> Self {
        Self {
            snapshots: std::collections::HashMap::new(),
        }
    }

    /// Update (or insert) the snapshot for an agent.
    pub fn update(&mut self, snapshot: AgentSnapshot) {
        debug!(agent = %snapshot.agent_id, "updating live view snapshot");
        self.snapshots.insert(snapshot.agent_id.clone(), snapshot);
    }

    /// Retrieve the current snapshot for a specific agent.
    pub fn get(&self, agent_id: &str) -> Option<&AgentSnapshot> {
        self.snapshots.get(agent_id)
    }

    /// Return all current agent snapshots.
    pub fn all(&self) -> Vec<&AgentSnapshot> {
        self.snapshots.values().collect()
    }

    /// Remove an agent from the live view.
    pub fn remove(&mut self, agent_id: &str) -> Option<AgentSnapshot> {
        self.snapshots.remove(agent_id)
    }
}

impl Default for LiveView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_and_get() {
        let mut lv = LiveView::new();
        let snap = AgentSnapshot {
            agent_id: "a1".into(),
            status: "running".into(),
            last_activity: None,
            message_count: 5,
        };
        lv.update(snap);
        let got = lv.get("a1").unwrap();
        assert_eq!(got.message_count, 5);
    }

    #[test]
    fn all_returns_everything() {
        let mut lv = LiveView::new();
        lv.update(AgentSnapshot {
            agent_id: "a1".into(),
            status: "running".into(),
            last_activity: None,
            message_count: 1,
        });
        lv.update(AgentSnapshot {
            agent_id: "a2".into(),
            status: "idle".into(),
            last_activity: None,
            message_count: 0,
        });
        assert_eq!(lv.all().len(), 2);
    }
}