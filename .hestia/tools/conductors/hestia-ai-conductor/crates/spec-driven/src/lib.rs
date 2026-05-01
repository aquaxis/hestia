pub mod parser;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpecError {
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("invalid requirement format: {0}")]
    InvalidRequirement(String),
    #[error("missing required field: {0}")]
    MissingField(String),
}
