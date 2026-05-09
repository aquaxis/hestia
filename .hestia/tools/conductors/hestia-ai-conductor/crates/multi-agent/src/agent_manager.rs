//! Agent lifecycle management (spawn / stop / list)
//!
//! サブエージェントを agent-cli プロセスとして生成・管理する。
//!
//! Phase 126 — グローバル並列度 cap を hardcode 16 から
//! `ConductorLimiter` ベースの timeout 付き Semaphore へ移行。reviewer
//! 系 agent には予約 slot を 1 つ常に確保し、auto-spawn reviewer が
//! cap 超過下でも起動できるようにする。

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
    /// agent-cli プロセスの PID（起動後設定）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    /// Phase 126 — グローバル cap を保持する Semaphore permit。
    /// stop / drop で release され、別 spawn が進めるようになる。
    #[serde(skip)]
    pub permit: Option<OwnedSemaphorePermit>,
}

/// Manages the lifecycle of multiple agents via agent-cli processes.
#[derive(Debug)]
pub struct AgentManager {
    agents: HashMap<String, AgentInfo>,
    /// 一般 spawn 用 limiter（global_max - 1）
    limiter: ConductorLimiter,
    /// reviewer 用に予約された slot（capacity 1）
    reviewer_slot: Arc<Semaphore>,
}

/// デフォルトのアイドルタイムアウト（秒）
const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// 既定のグローバル cap（env 未設定時）
const DEFAULT_GLOBAL_MAX: usize = 8;

/// 既定の acquire timeout（秒）
const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 600;

impl AgentManager {
    /// 互換用: env 駆動の既定値で構築。
    /// - `HESTIA_GLOBAL_MAX_AGENTS` (既定 8)
    /// - `HESTIA_ACQUIRE_TIMEOUT_SECS` (既定 600)
    pub fn new() -> Self {
        Self::with_default_cap()
    }

    /// 明示 cap で構築。global_max のうち 1 を reviewer 専用に予約する。
    pub fn with_caps(global_max: usize, timeout_secs: u64) -> Self {
        // global_max のうち最低 1 は一般用に確保
        let general = global_max.saturating_sub(1).max(1);
        Self {
            agents: HashMap::new(),
            limiter: ConductorLimiter::new(general, timeout_secs),
            reviewer_slot: Arc::new(Semaphore::new(1)),
        }
    }

    /// env 駆動で構築。
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

    /// agent-cli run でサブエージェントをプロセスとして起動する
    pub async fn spawn(&mut self, agent_id: String, conductor_id: String) -> Result<(), String> {
        if self.agents.contains_key(&agent_id) {
            return Err(format!("agent {agent_id} already exists"));
        }

        // Phase 126 — reviewer は予約 slot を最初に試行し、塞がっていれば
        // 一般 limiter にフォールバック。
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
            // permit は drop で自動 release される
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

    /// 同期版 spawn（テスト互換用）。permit を取得しない簡易スタブ。
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
        // permit を明示 drop して slot を解放
        info.permit.take();
        Ok(())
    }

    /// 同期版 stop
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

    /// 一般 limiter の現在空き slot 数。
    pub fn available(&self) -> usize {
        self.limiter.available()
    }

    /// 一般 limiter の cap。
    pub fn capacity(&self) -> usize {
        self.limiter.capacity()
    }

    /// reviewer 判定（peer 名 / persona 名どちらかに "reviewer" を含む）
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
        // global_max=2 なので一般 limiter は 1
        let mgr = AgentManager::with_caps(2, 60);
        assert_eq!(mgr.capacity(), 1);
        // reviewer 予約 slot は別途 1 確保される
        assert_eq!(mgr.reviewer_slot.available_permits(), 1);
    }

    #[test]
    fn is_reviewer_detects_name() {
        assert!(AgentManager::is_reviewer("ai-reviewer"));
        assert!(AgentManager::is_reviewer("rtl-reviewer-xyz"));
        assert!(!AgentManager::is_reviewer("ai-designer"));
    }
}
