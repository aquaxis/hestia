//! AdapterError — アダプターエラー型

use thiserror::Error;

/// アダプターエラー
#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Build failed: exit code {exit_code}")]
    BuildFailed { exit_code: i32 },

    #[error("Timeout after {secs}s")]
    Timeout { secs: u64 },

    #[error("Capability unsupported: {0}")]
    Unsupported(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}