//! agent-cli IPC トランスポート

use crate::agent::ConductorId;
use crate::config::HestiaClientConfig;
use crate::error::HestiaError;
use crate::message::Payload;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
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

    /// 稼働中の peer 一覧を取得（agent-cli list を使用、フォールバックでレジストリ直読み）
    pub async fn list_peers(&self) -> Result<Vec<String>, HestiaError> {
        let output = Command::new("agent-cli")
            .arg("list")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                Ok(stdout.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
            }
            _ => {
                // agent-cli が利用できない場合、レジストリディレクトリから直読み
                self.list_peers_from_registry()
            }
        }
    }

    /// レジストリディレクトリから peer 一覧を直読み（フォールバック）
    fn list_peers_from_registry(&self) -> Result<Vec<String>, HestiaError> {
        let entries = std::fs::read_dir(&self.registry_dir)
            .map_err(|e| HestiaError::Transport(format!("failed to read registry dir: {e}")))?;

        let mut peers = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| HestiaError::Transport(format!("registry entry error: {e}")))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                            peers.push(name.to_string());
                        }
                    }
                }
            }
        }

        Ok(peers)
    }

    /// 指定 peer へペイロード送信（直接ソケット通信）
    pub async fn send(&self, peer: &str, payload: &Payload) -> Result<String, HestiaError> {
        let socket_path = self.find_peer_socket(peer).await?;

        let mut stream = UnixStream::connect(&socket_path).await
            .map_err(|e| HestiaError::Transport(format!("failed to connect to {peer} socket: {e}")))?;

        let payload_str = match payload {
            Payload::Structured(v) => v.to_string(),
            Payload::NaturalLanguage(t) => t.clone(),
        };

        stream.write_all(payload_str.as_bytes()).await
            .map_err(|e| HestiaError::Transport(format!("failed to send to {peer}: {e}")))?;
        stream.write_all(b"\n").await
            .map_err(|e| HestiaError::Transport(format!("failed to send newline to {peer}: {e}")))?;
        stream.shutdown().await
            .map_err(|e| HestiaError::Transport(format!("failed to shutdown write half: {e}")))?;

        let mut buf = vec![0u8; 16 * 1024 * 1024]; // 16 MiB max
        let n = stream.read(&mut buf).await
            .map_err(|e| HestiaError::Transport(format!("failed to read response from {peer}: {e}")))?;

        if n == 0 {
            return Err(HestiaError::Transport(format!("no response from {peer}")));
        }

        Ok(String::from_utf8_lossy(&buf[..n]).to_string())
    }

    /// 指定 peer へペイロード送信（agent-cli send コマンド経由）
    pub async fn send_via_cli(&self, peer: &str, payload: &Payload) -> Result<String, HestiaError> {
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
                "agent-cli send to {peer} exited with {}: {stderr}",
                output.status
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// レジストリから peer のソケットパスを検索
    async fn find_peer_socket(&self, peer: &str) -> Result<PathBuf, HestiaError> {
        let entries = std::fs::read_dir(&self.registry_dir)
            .map_err(|e| HestiaError::Transport(format!("failed to read registry dir: {e}")))?;

        for entry in entries {
            let entry = entry.map_err(|e| HestiaError::Transport(format!("registry entry error: {e}")))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| HestiaError::Transport(format!("failed to read {}: {e}", path.display())))?;

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if json.get("name").and_then(|v| v.as_str()) == Some(peer) {
                        if let Some(socket) = json.get("socket").and_then(|v| v.as_str()) {
                            return Ok(PathBuf::from(socket));
                        }
                    }
                }
            }
        }

        Err(HestiaError::Transport(format!("peer not found by id or name: {peer}")))
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