//! Agent lifecycle management (spawn / stop / list)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;

/// Status of a managed agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
        .fmt(f)
    }
}

/// Snapshot of a managed agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: String,
    pub status: AgentStatus,
    pub conductor_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Manages the lifecycle of multiple agents.
#[derive(Debug)]
pub struct AgentManager {
    agents: HashMap<String, AgentInfo>,
}

impl AgentManager {
    /// Create an empty agent manager.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// Spawn a new agent, recording it with `Starting` status.
    ///
    /// Returns `Err` if an agent with the same ID already exists.
    pub fn spawn(&mut self, agent_id: String, conductor_id: String) -> Result<(), String> {
        if self.agents.contains_key(&agent_id) {
            return Err(format!("agent {agent_id} already exists"));
        }
        let info = AgentInfo {
            agent_id: agent_id.clone(),
            status: AgentStatus::Starting,
            conductor_id,
            started_at: chrono::Utc::now(),
        };
        info!(agent = %agent_id, "spawning agent");
        self.agents.insert(agent_id, info);
        Ok(())
    }

    /// Stop a running agent, transitioning it to `Stopping`.
    ///
    /// Returns `Err` if the agent is not found or already stopped.
    pub fn stop(&mut self, agent_id: &str) -> Result<(), String> {
        let info = self.agents.get_mut(agent_id).ok_or_else(|| format!("agent {agent_id} not found"))?;
        if info.status == AgentStatus::Stopped || info.status == AgentStatus::Stopping {
            return Err(format!("agent {agent_id} is already {}", info.status));
        }
        info!(agent = %agent_id, "stopping agent");
        info.status = AgentStatus::Stopping;
        Ok(())
    }

    /// Return a snapshot of all managed agents.
    pub fn list(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_and_list() {
        let mut mgr = AgentManager::new();
        mgr.spawn("agent-1".into(), "ai".into()).unwrap();
        let list = mgr.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].agent_id, "agent-1");
    }

    #[test]
    fn duplicate_spawn_fails() {
        let mut mgr = AgentManager::new();
        mgr.spawn("agent-1".into(), "ai".into()).unwrap();
        assert!(mgr.spawn("agent-1".into(), "ai".into()).is_err());
    }

    #[test]
    fn stop_nonexistent_fails() {
        let mut mgr = AgentManager::new();
        assert!(mgr.stop("nope").is_err());
    }
}