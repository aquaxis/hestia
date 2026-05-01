use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConstraintFormat {
    Xdc,
    Pcf,
    Sdc,
    EfinityXml,
    Qsf,
    Ucf,
}

impl fmt::Display for ConstraintFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xdc => write!(f, "XDC"),
            Self::Pcf => write!(f, "PCF"),
            Self::Sdc => write!(f, "SDC"),
            Self::EfinityXml => write!(f, "EfinityXML"),
            Self::Qsf => write!(f, "QSF"),
            Self::Ucf => write!(f, "UCF"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConstraintError {
    #[error("parse error in {format}: {message}")]
    ParseError {
        format: ConstraintFormat,
        message: String,
    },
    #[error("generate error for {format}: {message}")]
    GenerateError {
        format: ConstraintFormat,
        message: String,
    },
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}