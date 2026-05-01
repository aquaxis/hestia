//! ScriptAdapter — adapter.toml 宣言方式（原則2: ゼロ変更での拡張）

use serde::{Deserialize, Serialize};
use std::path::Path;

/// adapter.toml のスキーマ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterToml {
    pub manifest: super::manifest::AdapterManifest,
    pub tool: ToolConfig,
    #[serde(default)]
    pub commands: CommandConfig,
    #[serde(default)]
    pub log_parsing: LogParsingConfig,
    #[serde(default)]
    pub report_extraction: ReportExtractionConfig,
}

/// ツール設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

/// コマンド設定（各ビルドステップのコマンドマッピング）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandConfig {
    #[serde(default)]
    pub synthesize: Option<StepCommand>,
    #[serde(default)]
    pub implement: Option<StepCommand>,
    #[serde(default)]
    pub bitstream: Option<StepCommand>,
    #[serde(default)]
    pub timing: Option<StepCommand>,
    #[serde(default)]
    pub program: Option<StepCommand>,
}

/// ステップコマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepCommand {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// ログパースルール
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogParsingConfig {
    #[serde(default)]
    pub error_pattern: Option<String>,
    #[serde(default)]
    pub warning_pattern: Option<String>,
    #[serde(default)]
    pub info_pattern: Option<String>,
}

/// レポート抽出ルール
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportExtractionConfig {
    #[serde(default)]
    pub timing_pattern: Option<String>,
    #[serde(default)]
    pub resource_pattern: Option<String>,
}

/// adapter.toml をファイルから読み込む
pub fn load_adapter_toml(path: &Path) -> Result<AdapterToml, super::error::AdapterError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        super::error::AdapterError::Io(std::io::Error::other(format!(
            "Failed to read adapter.toml at {}: {e}",
            path.display()
        )))
    })?;
    toml::from_str(&content)
        .map_err(|e| super::error::AdapterError::Parse(format!("adapter.toml parse error: {e}")))
}