//! agent-cli IPC transport

use crate::agent::ConductorId;
use crate::config::HestiaClientConfig;
use crate::error::HestiaError;
use crate::message::{AgentCliPrompt, Payload};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::process::Command;

/// Default agent-id for CLI short-lived processes.
///
/// agent-cli's `AgentId(pub String)` requires only a non-empty string, and registry matching
/// is left to the receiving conductor's discretion. Since the CLI is not registered in the
/// registry, a fixed string is sufficient as an identifier.
const DEFAULT_FROM_ID: &str = "agent-hestia-cli";

/// Phase 125 — Pure function to determine whether `HESTIA_ENGINE_BINARY` points to claude-cli-shim.
///
/// Evaluates the return value of `crate::workspace::engine_binary()` (env-driven) by basename match.
/// Since the claude-cli-shim engine uses FIFO unidirectional IPC, it must route to
/// the `<engine_bin> send` subprocess instead of the Unix socket path (purpose of this phase).
fn engine_is_claude_cli_shim() -> bool {
    crate::workspace::engine_binary().contains("claude-cli-shim")
}

/// Phase 125 — Return the default registry path for the claude-cli-shim engine.
///
/// Default: `~/.local/share/claude-cli-shim/registry/`. Falls back to
/// `/tmp/claude-cli-shim/registry` if `$HOME` resolution fails (defensive).
fn default_claude_cli_shim_registry_dir() -> PathBuf {
    match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => PathBuf::from(h).join(".local/share/claude-cli-shim/registry"),
        _ => PathBuf::from("/tmp/claude-cli-shim/registry"),
    }
}

/// agent-cli IPC client
pub struct AgentCliClient {
    config: HestiaClientConfig,
    registry_dir: PathBuf,
}

impl AgentCliClient {
    pub fn new(config: HestiaClientConfig) -> Result<Self, HestiaError> {
        // Phase 125: branch registry_dir resolution by engine.
        // (1) explicit override (HestiaClientConfig.agent_cli_registry_dir) takes highest priority.
        // (2) For claude_cli_shim engine, default is ~/.local/share/claude-cli-shim/registry/.
        // (3) For agent_cli engine (default), use XDG_RUNTIME_DIR/agent-cli (existing behavior).
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

    /// Return registry directory
    pub fn registry_dir(&self) -> &PathBuf {
        &self.registry_dir
    }

    /// Get list of active peers (via engine binary, fallback to direct registry read)
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
                // If agent-cli is unavailable, read directly from registry directory
                self.list_peers_from_registry()
            }
        }
    }

    /// Read peer list directly from registry directory (fallback)
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

    /// Send payload to specified peer (direct socket communication)
    ///
    /// Wrap and send in agent-cli's `IpcMessage::Prompt` wire format
    /// `{"kind":"prompt","from":"<agent-id>","text":"..."}`.
    /// Domain payloads (`Payload::Structured`) are packed into `text` as JSON strings.
    ///
    /// Phase 125: For the claude_cli_shim engine, since IPC is FIFO unidirectional,
    /// route to `send_via_cli` (`<engine_bin> send <peer> <text>` subprocess).
    /// If stdout is empty, return a synthesized OK response to satisfy the caller's
    /// round-trip expectation (the caller falls back to `{"raw": ...}` on parse failure,
    /// but this synthesis provides structured JSON to maintain integration).
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

        // Pack payload into agent-cli wire format `text` field
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

    /// Resolve the `from` agent-id for sending.
    ///
    /// If `agent_cli_from_id` is specified in config, use it; otherwise return `DEFAULT_FROM_ID`.
    fn from_id(&self) -> String {
        let id = self.config.agent_cli_from_id.trim();
        if id.is_empty() {
            DEFAULT_FROM_ID.to_string()
        } else {
            id.to_string()
        }
    }

    /// Send payload to specified peer (via engine binary `send` command)
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

    /// Search for peer's socket path from registry
    ///
    /// Phase 125: For the claude_cli_shim engine, this function is not applicable because
    /// IPC is FIFO unidirectional (registry JSON has no `socket` field but has `fifo_path`).
    /// The `send` side guards for the engine and routes to `send_via_cli`, so this is normally
    /// not called, but defensively returns a clear error.
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

    /// Send structured message to Conductor
    pub async fn send_to_conductor(
        &self,
        conductor: ConductorId,
        payload: &Payload,
    ) -> Result<String, HestiaError> {
        self.send(conductor.peer_name(), payload).await
    }

    /// Reference to config
    pub fn config(&self) -> &HestiaClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phase 125 — env-driven determination of `engine_is_claude_cli_shim`.
    ///
    /// Note: `std::env::set_var` affects the entire process, so to avoid races
    /// across multiple env tests, this test combines "set → assert → restore" into
    /// a single test. It verifies both the client's new() path and registry_dir
    /// resolution at once.
    #[test]
    fn engine_is_claude_cli_shim_and_registry_dir_branch() {
        // Save original value
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

        // (2) agent_cli engine (default — env removed)
        std::env::remove_var("HESTIA_ENGINE_BINARY");
        assert!(!engine_is_claude_cli_shim());
        let cfg = HestiaClientConfig::default();
        let client = AgentCliClient::new(cfg).expect("client::new ok");
        let dir_str = client.registry_dir().to_string_lossy().to_string();
        assert!(
            dir_str.ends_with("agent-cli") || dir_str.ends_with("/tmp/agent-cli"),
            "default registry_dir must point to agent-cli path, got {dir_str}"
        );

        // Restore original value (race mitigation)
        match original {
            Some(v) => std::env::set_var("HESTIA_ENGINE_BINARY", v),
            None => std::env::remove_var("HESTIA_ENGINE_BINARY"),
        }
    }

    /// Phase 125 — explicit override takes highest priority and wins over engine default.
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