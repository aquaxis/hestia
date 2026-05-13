//! Agent lifecycle management (spawn / stop / list)
//!
//! Spawns and manages sub-agents as agent-cli processes.
//!
//! Phase 126 — Migrated global parallelism cap from hardcoded 16 to
//! `ConductorLimiter`-based Semaphore with timeout. One slot is always
//! reserved for reviewer agents so that auto-spawned reviewers can
//! start even when the cap is exceeded.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use conductor_sdk::concurrency::ConductorLimiter;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: String,
    pub status: AgentStatus,
    pub conductor_id: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// PID of the agent-cli process (set after startup)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Phase 126 — Semaphore permit holding the global cap.
    /// Released on stop/drop, allowing another spawn to proceed.
    #[serde(skip)]
    pub permit: Option<OwnedSemaphorePermit>,
}

/// Manages the lifecycle of multiple agents via agent-cli processes.
#[derive(Debug)]
pub struct AgentManager {
    agents: HashMap<String, AgentInfo>,
    /// General spawn limiter (global_max - 1)
    limiter: ConductorLimiter,
    /// Slot reserved for reviewers (capacity 1)
    reviewer_slot: Arc<Semaphore>,
}

/// Default idle timeout in seconds
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// Default global cap (when env var is not set)
const DEFAULT_GLOBAL_MAX: usize = 8;

/// Default acquire timeout in seconds
const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 600;

impl AgentManager {
    /// Compatibility constructor: builds with env-driven defaults.
    /// - `HESTIA_GLOBAL_MAX_AGENTS` (default 8)
    /// - `HESTIA_ACQUIRE_TIMEOUT_SECS` (default 600)
    pub fn new() -> Self {
        Self::with_default_cap()
    }

    /// Build with explicit caps. Reserves 1 of global_max for reviewer use.
    pub fn with_caps(global_max: usize, timeout_secs: u64) -> Self {
        // At least 1 of global_max is reserved for general use
        let general = global_max.saturating_sub(1).max(1);
        Self {
            agents: HashMap::new(),
            limiter: ConductorLimiter::new(general, timeout_secs),
            reviewer_slot: Arc::new(Semaphore::new(1)),
        }
    }

    /// Build with env-driven configuration.
    pub fn with_default_cap() -> Self {
        let global = std::env::var("HESTIA_GLOBAL_MAX_AGENTS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_GLOBAL_MAX);
        let to = std::env::var("HESTIA_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_ACQUIRE_TIMEOUT_SECS);
        Self::with_caps(global, to)
    }

    /// Spawn a sub-agent as a process via agent-cli run
    pub async fn spawn(&mut self, agent_id: String, conductor_id: String) -> Result<(), String> {
        if self.agents.contains_key(&agent_id) {
            return Err(format!("agent {agent_id} already exists"));
        }

        // Phase 126 — Reviewers try the reserved slot first; if it is occupied,
        // they fall back to the general limiter.
        let permit = if Self::is_reviewer(&agent_id) {
            match self.reviewer_slot.clone().try_acquire_owned() {
                Ok(p) => {
                    info!(agent = %agent_id, "acquired reserved reviewer slot");
                    p
                }
                Err(_) => {
                    info!(
                        agent = %agent_id,
                        "reviewer reserved slot busy; falling back to general limiter"
                    );
                    self.limiter
                        .acquire()
                        .await
                        .map_err(|e| format!("global cap acquire: {e}"))?
                }
            }
        } else {
            self.limiter
                .acquire()
                .await
                .map_err(|e| format!("global cap acquire: {e}"))?
        };

        let persona_path = PathBuf::from(format!(".hestia/personas/{conductor_id}.md"));
        let workdir = PathBuf::from(format!(".hestia/workspaces/{agent_id}"));

        if !persona_path.exists() {
            // permit is automatically released on drop
            return Err(format!("persona file not found: {}", persona_path.display()));
        }

        if !workdir.exists() {
            std::fs::create_dir_all(&workdir)
                .map_err(|e| format!("failed to create workspace {}: {e}", workdir.display()))?;
        }

        info!(agent = %agent_id, "spawning agent via agent-cli run");

        let child = Command::new("agent-cli")
            .arg("run")
            .arg("--persona")
            .arg(&persona_path)
            .arg("--name")
            .arg(&agent_id)
            .arg("--auto-approve-tools")
            .current_dir(&workdir)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("failed to spawn agent-cli for {agent_id}: {e}"))?;

        let pid: u32 = child.id().expect("failed to get agent-cli PID");
        info!(agent = %agent_id, pid = pid, "agent process started");

        let info = AgentInfo {
            agent_id: agent_id.clone(),
            status: AgentStatus::Running,
            conductor_id,
            started_at: chrono::Utc::now(),
            pid: Some(pid),
            permit: Some(permit),
        };
        self.agents.insert(agent_id, info);
        Ok(())
    }

    /// Synchronous spawn (test compatibility). Stub that does not acquire a permit.
    pub fn spawn_sync(&mut self, agent_id: String, conductor_id: String) -> Result<(), String> {
        if self.agents.contains_key(&agent_id) {
            return Err(format!("agent {agent_id} already exists"));
        }

        let info = AgentInfo {
            agent_id: agent_id.clone(),
            status: AgentStatus::Starting,
            conductor_id,
            started_at: chrono::Utc::now(),
            pid: None,
            permit: None,
        };
        info!(agent = %agent_id, "spawning agent (sync stub)");
        self.agents.insert(agent_id, info);
        Ok(())
    }

    /// Stop a running agent by sending SIGTERM to the process.
    pub async fn stop(&mut self, agent_id: &str) -> Result<(), String> {
        let info = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("agent {agent_id} not found"))?;
        if info.status == AgentStatus::Stopped || info.status == AgentStatus::Stopping {
            return Err(format!("agent {agent_id} is already {}", info.status));
        }

        info!(agent = %agent_id, "stopping agent");
        info.status = AgentStatus::Stopping;

        if let Some(pid) = info.pid {
            let _ = Command::new("kill").arg(pid.to_string()).output().await;
        }

        info.status = AgentStatus::Stopped;
        // Explicitly drop the permit to release the slot
        info.permit.take();
        Ok(())
    }

    /// Synchronous stop
    pub fn stop_sync(&mut self, agent_id: &str) -> Result<(), String> {
        let info = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| format!("agent {agent_id} not found"))?;
        if info.status == AgentStatus::Stopped || info.status == AgentStatus::Stopping {
            return Err(format!("agent {agent_id} is already {}", info.status));
        }
        info!(agent = %agent_id, "stopping agent (sync stub)");
        info.status = AgentStatus::Stopped;
        info.permit.take();
        Ok(())
    }

    /// Return a snapshot of all managed agents.
    pub fn list(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }

    /// Return the default idle timeout in seconds.
    pub fn idle_timeout_secs() -> u64 {
        DEFAULT_IDLE_TIMEOUT_SECS
    }

    /// Current number of available slots in the general limiter.
    pub fn available(&self) -> usize {
        self.limiter.available()
    }

    /// Capacity of the general limiter.
    pub fn capacity(&self) -> usize {
        self.limiter.capacity()
    }

    /// Check if agent is a reviewer (peer name or persona name contains "reviewer")
    fn is_reviewer(agent_id: &str) -> bool {
        agent_id.contains("reviewer")
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
        mgr.spawn_sync("agent-1".into(), "ai".into()).unwrap();
        let list = mgr.list();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].agent_id, "agent-1");
    }

    #[test]
    fn duplicate_spawn_fails() {
        let mut mgr = AgentManager::new();
        mgr.spawn_sync("agent-1".into(), "ai".into()).unwrap();
        assert!(mgr.spawn_sync("agent-1".into(), "ai".into()).is_err());
    }

    #[test]
    fn stop_nonexistent_fails() {
        let mut mgr = AgentManager::new();
        assert!(mgr.stop_sync("nope").is_err());
    }

    #[test]
    fn with_caps_reserves_one_for_reviewer() {
        // global_max=2 so general limiter is 1
        let mgr = AgentManager::with_caps(2, 60);
        assert_eq!(mgr.capacity(), 1);
        // Reviewer reserved slot is separately allocated as 1
        assert_eq!(mgr.reviewer_slot.available_permits(), 1);
    }

    #[test]
    fn is_reviewer_detects_name() {
        assert!(AgentManager::is_reviewer("ai-reviewer"));
        assert!(AgentManager::is_reviewer("rtl-reviewer-xyz"));
        assert!(!AgentManager::is_reviewer("ai-designer"));
    }
}
