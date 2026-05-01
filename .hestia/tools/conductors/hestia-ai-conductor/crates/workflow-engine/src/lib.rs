pub mod dag;
pub mod pipeline;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("cycle detected in workflow DAG")]
    CycleDetected,
    #[error("pipeline execution failed: {0}")]
    PipelineFailed(String),
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("storage error: {0}")]
    StorageError(String),
}
