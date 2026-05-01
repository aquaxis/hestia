//! agent-cli IPC トランスポート

use crate::agent::ConductorId;
use crate::config::HestiaClientConfig;
use crate::error::HestiaError;
use crate::message::Payload;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// agent-cli IPC クライアント
pub struct AgentCliClient {
    config: HestiaClientConfig,
    registry_dir: PathBuf,
}

impl AgentCliClient {
    pub fn new(config: HestiaClientConfig) -> Result<Self, HestiaError> {
        let registry_dir = if config.agent_cli_registry_dir.is_empty() {
            std::env::var("XDG_RUNTIME_DIR")
                .map(|d| PathBuf::from(d).join("agent-cli"))
                .unwrap_or_else(|_| PathBuf::from("/tmp/agent-cli"))
        } else {
            PathBuf::from(&config.agent_cli_registry_dir)
        };

        Ok(Self {
            config,
            registry_dir,
        })
    }

    /// レジストリディレクトリを返す
    pub fn registry_dir(&self) -> &PathBuf {
        &self.registry_dir
    }

    /// 稼働中の peer 一覧を取得
    pub async fn list_peers(&self) -> Result<Vec<String>, HestiaError> {
        let output = Command::new("agent-cli")
            .arg("list")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| HestiaError::Transport(format!("agent-cli list failed: {e}")))?;

        if !output.status.success() {
            return Err(HestiaError::Transport(format!(
                "agent-cli list exited with {}",
                output.status
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
    }

    /// 指定 peer へペイロード送信
    pub async fn send(&self, peer: &str, payload: &Payload) -> Result<String, HestiaError> {
        let payload_str = match payload {
            Payload::Structured(v) => v.to_string(),
            Payload::NaturalLanguage(t) => t.clone(),
        };

        let output = Command::new("agent-cli")
            .arg("send")
            .arg(peer)
            .arg(&payload_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| HestiaError::Transport(format!("agent-cli send failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HestiaError::Transport(format!(
                "agent-cli send to {peer} failed: {stderr}"
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Conductor へ構造化メッセージ送信
    pub async fn send_to_conductor(
        &self,
        conductor: ConductorId,
        payload: &Payload,
    ) -> Result<String, HestiaError> {
        self.send(conductor.peer_name(), payload).await
    }

    /// 設定の参照
    pub fn config(&self) -> &HestiaClientConfig {
        &self.config
    }
}