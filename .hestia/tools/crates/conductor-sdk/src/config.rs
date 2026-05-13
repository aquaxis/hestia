//! Client configuration

use serde::{Deserialize, Serialize};

/// Hestia client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HestiaClientConfig {
    /// agent-cli registry directory
    /// Default: $XDG_RUNTIME_DIR/agent-cli/
    #[serde(default)]
    pub agent_cli_registry_dir: String,

    /// Request timeout (milliseconds)
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// Reconnect interval (milliseconds)
    #[serde(default = "default_reconnect_interval")]
    pub reconnect_interval: u64,

    /// Maximum reconnect attempts
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,

    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Retry policy
    #[serde(default)]
    pub retry_policy: RetryPolicy,

    /// Maximum frame length (bytes)
    #[serde(default = "default_max_frame_length")]
    pub max_frame_length: u64,

    /// `from` agent-id used when sending via agent-cli IPC (empty = default "agent-hestia-cli")
    #[serde(default)]
    pub agent_cli_from_id: String,
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
            agent_cli_from_id: String::new(),
        }
    }
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub multiplier: f64,
    /// Retryable error code list
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

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// agent-cli backend configuration (config.toml [agent_cli])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCliConfig {
    /// Backend type: "claude" | "codex" | "ollama" | "llama_cpp"
    #[serde(default = "default_backend")]
    pub backend: String,

    /// agent-cli binary path (empty = resolve via $PATH)
    #[serde(default)]
    pub binary_path: String,

    /// Anthropic API base URL (empty = official)
    #[serde(default)]
    pub anthropic_base_url: String,

    /// Environment variable name storing the API key
    #[serde(default = "default_api_key_env")]
    pub anthropic_api_key_env: String,

    /// LLM model identifier
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum response token count
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// IPC registry directory (empty = $XDG_RUNTIME_DIR/agent-cli)
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

/// Common CLI options
///
/// All flags are set to `global = true`, so they can be specified before or after the subcommand:
/// - `hestia-rtl-cli --output json lint`  <- Traditional form (before subcommand)
/// - `hestia-rtl-cli lint --output json`  <- Preferred form (after subcommand)
#[derive(Debug, Clone, clap::Parser)]
pub struct CommonOpts {
    /// Output format: human | json
    #[arg(long, global = true, default_value = "human")]
    pub output: String,

    /// RPC timeout (seconds)
    #[arg(long, global = true)]
    pub timeout: Option<u64>,

    /// agent-cli registry path
    #[arg(long, global = true)]
    pub registry: Option<String>,

    /// Configuration file path
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Verbose log output
    #[arg(long, global = true)]
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