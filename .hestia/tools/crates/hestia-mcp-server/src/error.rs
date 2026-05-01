use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("tool not found: {0}")]
    ToolNotFound(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("conductor error: {0}")]
    ConductorError(String),
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}