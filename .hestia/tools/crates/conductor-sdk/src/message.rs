//! Structured message specification (Request / Response / Notification / Batch)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message ID (msg_{ISO8601 timestamp}_{random})
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

/// Trace ID (for cross-workflow tracking)
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

/// Method namespace: {domain}.{method_group}.{version_prefix}.{action}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MethodName(String);

impl MethodName {
    pub fn new(domain: &str, group: &str, version: &str, action: &str) -> Self {
        Self(format!("{domain}.{group}.{version}.{action}"))
    }

    /// Shorthand: {domain}.{action} (v1 default)
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

/// API version
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}

/// Deprecation notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationNotice {
    pub deprecated_since: ApiVersion,
    pub removal_scheduled: ApiVersion,
    pub replacement: String,
}

/// Structured request (hestia internal domain request representation)
///
/// Serialized as JSON and embedded in the `text` field of agent-cli's IPC wire format (`AgentCliPrompt`).
/// The `kind`/`from` fields are retained for legacy compatibility, but when sending via agent-cli,
/// the outer `AgentCliPrompt` handles wire-level `kind`/`from`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Message type (legacy compatibility): prompt | ack | error | ping | pong
    #[serde(default = "default_kind")]
    pub kind: String,
    /// Sender ID (legacy compatibility)
    #[serde(default = "default_from")]
    pub from: String,
    /// Method name (hestia domain request dispatch key)
    pub method: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

fn default_kind() -> String {
    "prompt".to_string()
}

fn default_from() -> String {
    "cli".to_string()
}

/// agent-cli IPC wire format (compatible with `IpcMessage::Prompt`)
///
/// Source: `IpcMessage::Prompt` in `agent-cli/src/ipc/mod.rs`.
/// Since it's an externally-tagged enum via `serde(tag = "kind", rename_all = "snake_case")`,
/// including `kind = "prompt"` as the first field will cause agent-cli's
/// `serde_json::from_str::<IpcMessage>` to deserialize it as `Prompt { from, from_name, text }`.
#[derive(Debug, Clone, Serialize)]
pub struct AgentCliPrompt {
    /// Must be "prompt"
    pub kind: &'static str,
    /// Sender agent-id (maps directly to agent-cli's `AgentId(pub String)`. Requires non-empty string)
    pub from: String,
    /// Display name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_name: Option<String>,
    /// Body (natural language or JSON-serialized domain message)
    pub text: String,
}

impl AgentCliPrompt {
    /// Construct a new `AgentCliPrompt`
    pub fn new(from: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            kind: "prompt",
            from: from.into(),
            from_name: None,
            text: text.into(),
        }
    }

    /// Construct with display name
    pub fn with_name(from: impl Into<String>, from_name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            kind: "prompt",
            from: from.into(),
            from_name: Some(from_name.into()),
            text: text.into(),
        }
    }
}

/// Success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub result: serde_json::Value,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResultResponse {
    pub error: crate::error::ErrorResponse,
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// Notification (no id, no response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub method: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,
}

/// Response (success or error)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Success(SuccessResponse),
    Error(ErrorResultResponse),
}

/// Batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest(pub Vec<Request>);

/// Payload (structured JSON or natural language text)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload {
    Structured(serde_json::Value),
    NaturalLanguage(String),
}

impl Payload {
    /// Determine whether the payload is structured JSON or natural language
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