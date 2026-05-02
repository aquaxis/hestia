//! Agent lifecycle management (spawn / stop / list)
//!
//! サブエージェントを agent-cli プロセスとして生成・管理する。

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tracing::info;
use tokio::process::Command;

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
    /// agent-cli プロセスの PID（起動後設定）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// Manages the lifecycle of multiple agents via agent-cli processes.
#[derive(Debug)]
pub struct AgentManager {
    agents: HashMap<String, AgentInfo>,
}

/// デフォルトのアイドルタイムアウト（秒）
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// 最大並列サブエージェント数
const MAX_PARALLEL_AGENTS: usize = 16;

impl AgentManager {
    /// Create an empty agent manager.
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    /// agent-cli run でサブエージェントをプロセスとして起動する
    pub async fn spawn(&mut self, agent_id: String, conductor_id: String) -> Result<(), String> {
        if self.agents.contains_key(&agent_id) {
            return Err(format!("agent {agent_id} already exists"));
        }

        if self.agents.values().filter(|a| a.status == AgentStatus::Running).count() >= MAX_PARALLEL_AGENTS {
            return Err(format!("max parallel agents ({MAX_PARALLEL_AGENTS}) reached"));
        }

        let persona_path = PathBuf::from(format!(".hestia/personas/{conductor_id}.md"));
        let workdir = PathBuf::from(format!(".hestia/workspaces/{agent_id}"));

        if !persona_path.exists() {
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
        };
        self.agents.insert(agent_id, info);
        Ok(())
    }

    /// 同期版 spawn（テスト互換用）
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
        };
        info!(agent = %agent_id, "spawning agent (sync stub)");
        self.agents.insert(agent_id, info);
        Ok(())
    }

    /// Stop a running agent by sending SIGTERM to the process.
    pub async fn stop(&mut self, agent_id: &str) -> Result<(), String> {
        let info = self.agents.get_mut(agent_id).ok_or_else(|| format!("agent {agent_id} not found"))?;
        if info.status == AgentStatus::Stopped || info.status == AgentStatus::Stopping {
            return Err(format!("agent {agent_id} is already {}", info.status));
        }

        info!(agent = %agent_id, "stopping agent");
        info.status = AgentStatus::Stopping;

        if let Some(pid) = info.pid {
            let _ = Command::new("kill")
                .arg(pid.to_string())
                .output()
                .await;
        }

        info.status = AgentStatus::Stopped;
        Ok(())
    }

    /// 同期版 stop
    pub fn stop_sync(&mut self, agent_id: &str) -> Result<(), String> {
        let info = self.agents.get_mut(agent_id).ok_or_else(|| format!("agent {agent_id} not found"))?;
        if info.status == AgentStatus::Stopped || info.status == AgentStatus::Stopping {
            return Err(format!("agent {agent_id} is already {}", info.status));
        }
        info!(agent = %agent_id, "stopping agent (sync stub)");
        info.status = AgentStatus::Stopped;
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

    /// Return the max parallel agents limit.
    pub fn max_parallel() -> usize {
        MAX_PARALLEL_AGENTS
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
}