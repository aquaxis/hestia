use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObservabilityError {
    #[error("metrics error: {0}")]
    MetricsError(String),
    #[error("trace error: {0}")]
    TraceError(String),
    #[error("health check failed: {0}")]
    HealthCheckFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}