use thiserror::Error;

#[derive(Debug, Error)]
pub enum CicdError {
    #[error("pipeline not found: {0}")]
    PipelineNotFound(String),
    #[error("job failed: {job} in stage {stage}")]
    JobFailed { stage: String, job: String },
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("backend error: {0}")]
    BackendError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}