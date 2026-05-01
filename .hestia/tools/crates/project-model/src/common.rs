//! 共通プロジェクト設定モデル

use serde::{Deserialize, Serialize};

/// 共通プロジェクト情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCommon {
    pub name: String,
    #[serde(default)]
    pub version: String,
}