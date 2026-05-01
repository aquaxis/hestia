//! クライアント設定

use serde::{Deserialize, Serialize};

/// Hestia クライアント設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HestiaClientConfig {
    /// agent-cli レジストリディレクトリ
    /// 既定: $XDG_RUNTIME_DIR/agent-cli/
    #[serde(default)]
    pub agent_cli_registry_dir: String,

    /// リクエストタイムアウト（ミリ秒）
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// 再接続間隔（ミリ秒）
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval: u64,

    /// 最大再接続試行回数
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,

    /// ログレベル
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// リトライポリシー
    #[serde(default)]
    pub retry_policy: RetryPolicy,

    /// 最大フレーム長（バイト）
    #[serde(default = "default_max_frame_length")]
    pub max_frame_length: u64,
}

impl Default for HestiaClientConfig {
    fn default() -> Self {
        Self {
            agent_cli_registry_dir: String::new(),
            request_timeout: default_request_timeout(),
            reconnect_interval: default_reconnect_interval(),
            max_reconnect_attempts: default_max_reconnect_attempts(),
            log_level: default_log_level(),
            retry_policy: RetryPolicy::default(),
            max_frame_length: default_max_frame_length(),
        }
    }
}

/// リトライポリシー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub multiplier: f64,
    /// リトライ可能なエラーコード一覧
    pub retryable_codes: Vec<i32>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 60000,
            multiplier: 2.0,
            retryable_codes: vec![-32001, -32006],
        }
    }
}

/// 接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// agent-cli バックエンド設定（config.toml [agent_cli]）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCliConfig {
    /// バックエンド種別: "claude" | "codex" | "ollama" | "llama_cpp"
    #[serde(default = "default_backend")]
    pub backend: String,

    /// agent-cli バイナリパス（空 = $PATH 解決）
    #[serde(default)]
    pub binary_path: String,

    /// Anthropic API ベース URL（空 = 公式）
    #[serde(default)]
    pub anthropic_base_url: String,

    /// API キーを格納する環境変数名
    #[serde(default = "default_api_key_env")]
    pub anthropic_api_key_env: String,

    /// LLM モデル識別子
    #[serde(default = "default_model")]
    pub model: String,

    /// 応答上限トークン数
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// IPC レジストリディレクトリ（空 = $XDG_RUNTIME_DIR/agent-cli）
    #[serde(default)]
    pub registry_dir: String,
}

impl Default for AgentCliConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
            binary_path: String::new(),
            anthropic_base_url: String::new(),
            anthropic_api_key_env: default_api_key_env(),
            model: default_model(),
            max_tokens: default_max_tokens(),
            registry_dir: String::new(),
        }
    }
}

fn default_request_timeout() -> u64 {
    30000
}
fn default_reconnect_interval() -> u64 {
    3000
}
fn default_max_reconnect_attempts() -> u32 {
    5
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_max_frame_length() -> u64 {
    16 * 1024 * 1024 // 16 MiB
}
fn default_backend() -> String {
    "claude".to_string()
}
fn default_api_key_env() -> String {
    "ANTHROPIC_API_KEY".to_string()
}
fn default_model() -> String {
    "claude-opus-4-7".to_string()
}
fn default_max_tokens() -> u32 {
    4096
}

/// 共通 CLI オプション
#[derive(Debug, Clone, clap::Parser)]
pub struct CommonOpts {
    /// 出力フォーマット: human | json
    #[arg(long, default_value = "human")]
    pub output: String,

    /// RPC タイムアウト（秒）
    #[arg(long)]
    pub timeout: Option<u64>,

    /// agent-cli レジストリパス
    #[arg(long)]
    pub registry: Option<String>,

    /// 設定ファイルパス
    #[arg(long)]
    pub config: Option<String>,

    /// 詳細ログ出力
    #[arg(long)]
    pub verbose: bool,
}

/// CLI Exit Code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    RpcError = 2,
    ConfigError = 3,
    Timeout = 4,
    NotConnected = 5,
    InvalidArgs = 6,
    SocketNotFound = 7,
    PermissionDenied = 8,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}