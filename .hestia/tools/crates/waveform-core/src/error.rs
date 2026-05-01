use crate::types::WaveformFormat;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WaveformError {
    #[error("parse error in {format}: {message}")]
    ParseError {
        format: WaveformFormat,
        message: String,
    },
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}