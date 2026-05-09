//! agent-cli IPC トランスポート

use crate::agent::ConductorId;
use crate::config::HestiaClientConfig;
use crate::error::HestiaError;
use crate::message::{AgentCliPrompt, Payload};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;

/// CLI 短命プロセス用のデフォルト agent-id。
///
/// agent-cli の `AgentId(pub String)` は非空文字列のみ要求し、レジストリ照合は
/// 受信側 conductor の判断に委ねられる。CLI はレジストリ登録されないため、
/// 識別子としては固定文字列で十分。
const DEFAULT_FROM_ID: &str = "agent-hestia-cli";

/// Phase 125 — `HESTIA_ENGINE_BINARY` が claude-cli-shim を指しているかを判定する純関数。
///
/// `crate::workspace::engine_binary()` の戻り値 (env-driven) を basename match で評価。
/// claude-cli-shim engine では IPC が FIFO unidirectional のため、Unix socket 経路を
/// 取らずに `<engine_bin> send` subprocess にルートする必要がある (本 phase の主旨)。
fn engine_is_claude_cli_shim() -> bool {
    crate::workspace::engine_binary().contains("claude-cli-shim")
}

/// Phase 125 — claude-cli-shim engine の registry 既定パスを返す。
///
/// `~/.local/share/claude-cli-shim/registry/` 既定。`$HOME` 解決失敗時は
/// `/tmp/claude-cli-shim/registry` にフォールバック (defensive)。
fn default_claude_cli_shim_registry_dir() -> PathBuf {
    match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => PathBuf::from(h).join(".local/share/claude-cli-shim/registry"),
        _ => PathBuf::from("/tmp/claude-cli-shim/registry"),
    }
}

/// agent-cli IPC クライアント
pub struct AgentCliClient {
    config: HestiaClientConfig,
    registry_dir: PathBuf,
}

impl AgentCliClient {
    pub fn new(config: HestiaClientConfig) -> Result<Self, HestiaError> {
        // Phase 125: registry_dir 解決を engine 別に分岐。
        // (1) explicit override (HestiaClientConfig.agent_cli_registry_dir) を最優先。
        // (2) claude_cli_shim engine 時は ~/.local/share/claude-cli-shim/registry/ 既定。
        // (3) agent_cli engine (default) は XDG_RUNTIME_DIR/agent-cli (既存挙動)。
        let registry_dir = if !config.agent_cli_registry_dir.is_empty() {
            PathBuf::from(&config.agent_cli_registry_dir)
        } else if engine_is_claude_cli_shim() {
            default_claude_cli_shim_registry_dir()
        } else {
            std::env::var("XDG_RUNTIME_DIR")
                .map(|d| PathBuf::from(d).join("agent-cli"))
                .unwrap_or_else(|_| PathBuf::from("/tmp/agent-cli"))
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

    /// 稼働中の peer 一覧を取得（engine binary 経由、フォールバックでレジストリ直読み）
    pub async fn list_peers(&self) -> Result<Vec<String>, HestiaError> {
        let bin = crate::workspace::engine_binary();
        let output = Command::new(&bin)
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
    ///
    /// agent-cli の `IpcMessage::Prompt` ワイヤフォーマット
    /// `{"kind":"prompt","from":"<agent-id>","text":"..."}` でラップして送る。
    /// ドメインペイロード（`Payload::Structured`）は JSON 文字列として `text` に詰める。
    ///
    /// Phase 125: claude_cli_shim engine 時は IPC が FIFO unidirectional のため
    /// `send_via_cli` (`<engine_bin> send <peer> <text>` subprocess) にルートする。
    /// stdout が空なら synthesized OK レスポンスを返し、呼び元の round-trip 期待を
    /// 満たす (caller は parse 失敗時 `{"raw": ...}` に fallback するが、
    /// 本 synthesize で structured JSON を提供することで integration を保てる)。
    pub async fn send(&self, peer: &str, payload: &Payload) -> Result<String, HestiaError> {
        if engine_is_claude_cli_shim() {
            let raw = self.send_via_cli(peer, payload).await?;
            if raw.trim().is_empty() {
                return Ok(serde_json::json!({
                    "status": "ok",
                    "transport": "claude-cli-shim",
                    "peer": peer,
                    "note": "fire-and-forget send (no synchronous response from FIFO peer)"
                })
                .to_string());
            }
            return Ok(raw);
        }
        let socket_path = self.find_peer_socket(peer).await?;

        let mut stream = UnixStream::connect(&socket_path).await
            .map_err(|e| HestiaError::Transport(format!("failed to connect to {peer} socket: {e}")))?;

        // ペイロードを agent-cli wire format の `text` フィールドに詰める
        let text = match payload {
            Payload::Structured(v) => v.to_string(),
            Payload::NaturalLanguage(t) => t.clone(),
        };
        let wire = AgentCliPrompt::new(self.from_id(), text);
        let line = serde_json::to_string(&wire)
            .map_err(|e| HestiaError::Transport(format!("failed to serialize prompt: {e}")))?;

        stream.write_all(line.as_bytes()).await
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

    /// 送信時の `from` agent-id を解決する。
    ///
    /// 設定で `agent_cli_from_id` が指定されていればそれを使い、未指定なら
    /// `DEFAULT_FROM_ID` を返す。
    fn from_id(&self) -> String {
        let id = self.config.agent_cli_from_id.trim();
        if id.is_empty() {
            DEFAULT_FROM_ID.to_string()
        } else {
            id.to_string()
        }
    }

    /// 指定 peer へペイロード送信（engine binary `send` コマンド経由）
    pub async fn send_via_cli(&self, peer: &str, payload: &Payload) -> Result<String, HestiaError> {
        let payload_str = match payload {
            Payload::Structured(v) => v.to_string(),
            Payload::NaturalLanguage(t) => t.clone(),
        };

        let bin = crate::workspace::engine_binary();
        let output = Command::new(&bin)
            .arg("send")
            .arg(peer)
            .arg(&payload_str)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| HestiaError::Transport(format!("{bin} send failed: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HestiaError::Transport(format!(
                "{bin} send to {peer} exited with {}: {stderr}",
                output.status
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// レジストリから peer のソケットパスを検索
    ///
    /// Phase 125: claude_cli_shim engine では FIFO unidirectional のため本関数は
    /// 適用外 (registry JSON に `socket` field が無く `fifo_path` を持つ)。`send`
    /// 側で engine ガードして `send_via_cli` にルートしているため通常は呼ばれないが、
    /// 防衛的に明確エラーを返す。
    async fn find_peer_socket(&self, peer: &str) -> Result<PathBuf, HestiaError> {
        if engine_is_claude_cli_shim() {
            return Err(HestiaError::Transport(format!(
                "find_peer_socket('{peer}') called under claude_cli_shim engine — \
                 this engine uses FIFO transport, route via send_via_cli instead"
            )));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Phase 125 — `engine_is_claude_cli_shim` の env 駆動判定。
    ///
    /// 注: `std::env::set_var` はプロセス全体に影響するため、複数 env テストの
    /// race を避けるべく本テストでは「set → assert → restore」の単一テストに
    /// まとめる。クライアントの new() 経路 + registry_dir 解決の双方を一度に
    /// 検証する。
    #[test]
    fn engine_is_claude_cli_shim_and_registry_dir_branch() {
        // 元値を保存
        let original = std::env::var("HESTIA_ENGINE_BINARY").ok();

        // (1) shim engine
        std::env::set_var("HESTIA_ENGINE_BINARY", "claude-cli-shim");
        assert!(engine_is_claude_cli_shim());
        let cfg = HestiaClientConfig::default();
        let client = AgentCliClient::new(cfg).expect("client::new ok");
        let dir_str = client.registry_dir().to_string_lossy().to_string();
        assert!(
            dir_str.ends_with(".local/share/claude-cli-shim/registry")
                || dir_str.ends_with("/tmp/claude-cli-shim/registry"),
            "shim registry_dir must point to claude-cli-shim path, got {dir_str}"
        );

        // (2) agent_cli engine (default — env 削除)
        std::env::remove_var("HESTIA_ENGINE_BINARY");
        assert!(!engine_is_claude_cli_shim());
        let cfg = HestiaClientConfig::default();
        let client = AgentCliClient::new(cfg).expect("client::new ok");
        let dir_str = client.registry_dir().to_string_lossy().to_string();
        assert!(
            dir_str.ends_with("agent-cli") || dir_str.ends_with("/tmp/agent-cli"),
            "default registry_dir must point to agent-cli path, got {dir_str}"
        );

        // 元値を復元 (race 緩和)
        match original {
            Some(v) => std::env::set_var("HESTIA_ENGINE_BINARY", v),
            None => std::env::remove_var("HESTIA_ENGINE_BINARY"),
        }
    }

    /// Phase 125 — explicit override が最優先で engine 既定より勝つこと。
    #[test]
    fn explicit_registry_override_wins_over_engine_default() {
        let original = std::env::var("HESTIA_ENGINE_BINARY").ok();

        std::env::set_var("HESTIA_ENGINE_BINARY", "claude-cli-shim");
        let mut cfg = HestiaClientConfig::default();
        cfg.agent_cli_registry_dir = "/tmp/custom-registry".to_string();
        let client = AgentCliClient::new(cfg).expect("client::new ok");
        assert_eq!(
            client.registry_dir().to_string_lossy(),
            "/tmp/custom-registry"
        );

        match original {
            Some(v) => std::env::set_var("HESTIA_ENGINE_BINARY", v),
            None => std::env::remove_var("HESTIA_ENGINE_BINARY"),
        }
    }
}