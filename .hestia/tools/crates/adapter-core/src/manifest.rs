//! AdapterManifest — アダプター自己記述

use serde::{Deserialize, Serialize};

/// アダプター設定情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterManifest {
    /// 識別子（例: "com.xilinx.vivado"）
    pub id: String,
    /// 表示名（例: "AMD Vivado"）
    pub name: String,
    /// アダプター自身のバージョン
    pub version: String,
    /// ベンダー名
    pub vendor: String,
    /// ABI 互換チェック用バージョン
    pub api_version: u32,
    /// サポートデバイス（glob パターン）
    #[serde(default)]
    pub supported_devices: Vec<String>,
    /// リリースノート URL（WatcherAgent が使用）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_notes_url: Option<String>,
}