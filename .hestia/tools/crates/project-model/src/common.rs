//! Common project configuration model

use serde::{Deserialize, Serialize};

/// Common project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCommon {
    pub name: String,
    #[serde(default)]
    pub version: String,
}