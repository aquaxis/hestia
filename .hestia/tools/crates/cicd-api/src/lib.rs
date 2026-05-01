pub mod error;
pub mod types;

pub use error::CicdError;
pub use types::{Backend, PipelineDefinition, PipelineJob, PipelineStage, StageCondition};