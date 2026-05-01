pub mod registry;
pub mod skill;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("skill not found: {0}")]
    NotFound(String),
    #[error("skill already registered: {0}")]
    AlreadyRegistered(String),
    #[error("skill execution failed: {0}")]
    ExecutionFailed(String),
}
