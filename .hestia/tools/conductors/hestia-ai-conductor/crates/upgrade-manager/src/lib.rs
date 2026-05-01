pub mod version_policy;
pub mod rollout;
pub mod rollback;
pub mod pipeline;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpgradeError {
    #[error("version policy violation: {0}")]
    VersionPolicyViolation(String),
    #[error("rollout failed: {0}")]
    RolloutFailed(String),
    #[error("rollback failed: {0}")]
    RollbackFailed(String),
    #[error("upgrade pipeline error: {0}")]
    PipelineError(String),
    #[error("invalid semver: {0}")]
    InvalidSemver(String),
}
