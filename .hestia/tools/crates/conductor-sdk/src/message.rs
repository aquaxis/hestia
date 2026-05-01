//! 構造化メッセージ仕様（Request / Response / Notification / Batch）

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// メッセージ ID（msg_{ISO8601 timestamp}_{random}）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(String);

impl MessageId {
    pub fn new() -> Self {
        let now: DateTime<Utc> = Utc::now();
        let random = Uuid::new_v4();
        Self(format!("msg_{}_{}", now.format("%Y-%m-%dT%H:%M:%SZ"), random))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// トレース ID（ワークフロー横断の追跡用）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(String);

impl TraceId {
    pub fn new() -> Self {
        Self(format!("trace_{}", Uuid::new_v4()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

/// メソッド名前空間: {domain}.{method_group}.{version_prefix}.{action}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodName(String);

impl MethodName {
    pub fn new(domain: &str, group: &str, version: &str, action: &str) -> Self {
        Self(format!("{domain}.{group}.{version}.{action}"))
    }

    /// 簡略形: {domain}.{action}（v1 既定）
    pub fn simple(domain: &str, action: &str) -> Self {
        Self(format!("{domain}.{action}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MethodName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for MethodName {
    type Err = crate::error::HestiaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(crate::error::HestiaError::Parse("Empty method name".to_string()));
        }
        Ok(Self(s.to_string()))
    }
}

/// API バージョン
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}

/// 廃止予告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationNotice {
    pub deprecated_since: ApiVersion,
    pub removal_scheduled: ApiVersion,
    pub replacement: String,
}

/// 構造化リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// 成功応答
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub result: serde_json::Value,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// エラー応答
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResultResponse {
    pub error: crate::error::ErrorResponse,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// 通知（id なし、応答なし）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub method: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// 応答（成功またはエラー）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Success(SuccessResponse),
    Error(ErrorResultResponse),
}

/// バッチリクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest(pub Vec<Request>);

/// ペイロード（構造化 JSON または自然言語テキスト）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Structured(serde_json::Value),
    NaturalLanguage(String),
}

impl Payload {
    /// ペイロードが構造化 JSON か自然言語かを判定
    pub fn is_structured(&self) -> bool {
        matches!(self, Self::Structured(_))
    }
}

impl From<String> for Payload {
    fn from(s: String) -> Self {
        if s.starts_with('{') {
            match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(v) => Self::Structured(v),
                Err(_) => Self::NaturalLanguage(s),
            }
        } else {
            Self::NaturalLanguage(s)
        }
    }
}