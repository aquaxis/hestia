//! Hestia エラー型・エラーコード規約

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Hestia SDK エラー型
#[derive(Debug, Error)]
pub enum HestiaError {
    #[error("Invalid conductor ID: {0}")]
    InvalidConductorId(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Not connected to conductor")]
    NotConnected,

    #[error("Conductor not found: {0}")]
    ConductorNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("RPC error {code}: {message}")]
    Rpc { code: i32, message: String },

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 構造化エラー応答の data フィールド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    pub tool: String,
    pub exit_code: i32,
    pub log_path: String,
    pub errors: Vec<String>,
    pub retry_possible: bool,
    pub suggested_action: String,
}

/// 構造化エラー応答
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ErrorData>,
}

impl From<HestiaError> for ErrorResponse {
    fn from(err: HestiaError) -> Self {
        match &err {
            HestiaError::Timeout(_) => ErrorResponse {
                code: -32001,
                message: err.to_string(),
                data: None,
            },
            HestiaError::ConductorNotFound(name) => ErrorResponse {
                code: -32002,
                message: format!("Not found: {name}"),
                data: None,
            },
            HestiaError::PermissionDenied(msg) => ErrorResponse {
                code: -32004,
                message: msg.clone(),
                data: None,
            },
            HestiaError::ServiceUnavailable(msg) => ErrorResponse {
                code: -32006,
                message: msg.clone(),
                data: None,
            },
            HestiaError::Parse(_) => ErrorResponse {
                code: -32700,
                message: err.to_string(),
                data: None,
            },
            HestiaError::Rpc { code, message } => ErrorResponse {
                code: *code,
                message: message.clone(),
                data: None,
            },
            HestiaError::NotConnected => ErrorResponse {
                code: -32006,
                message: "Not connected".to_string(),
                data: None,
            },
            _ => ErrorResponse {
                code: -32000,
                message: err.to_string(),
                data: None,
            },
        }
    }
}

/// エラーコード定数（HESTIA 共通）
pub mod error_code {
    // 標準エラー（JSON-RPC 2.0 流用）
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // HESTIA 共通（-32000 ~ -32099）
    pub const INTERNAL: i32 = -32000;
    pub const TIMEOUT: i32 = -32001;
    pub const NOT_FOUND: i32 = -32002;
    pub const ALREADY_EXISTS: i32 = -32003;
    pub const PERMISSION_DENIED: i32 = -32004;
    pub const INVALID_STATE: i32 = -32005;
    pub const SERVICE_UNAVAILABLE: i32 = -32006;

    // ai-conductor（-32100 ~ -32199）
    pub const AI_ORCHESTRATION_START: i32 = -32100;
    pub const AI_AGENT_MANAGEMENT_START: i32 = -32120;
    pub const AI_SPEC_DRIVEN_START: i32 = -32140;
    pub const AI_VERSION_TRACKING_START: i32 = -32160;
    pub const AI_LLM_START: i32 = -32180;

    // fpga-conductor（-32200 ~ -32299）
    pub const FPGA_SYNTHESIS_START: i32 = -32200;
    pub const FPGA_IMPLEMENTATION_START: i32 = -32210;
    pub const FPGA_BITSTREAM_START: i32 = -32220;
    pub const FPGA_TIMING_START: i32 = -32230;
    pub const FPGA_DEBUG_START: i32 = -32240;
    pub const FPGA_HLS_START: i32 = -32250;
    pub const FPGA_DEVICE_START: i32 = -32260;
    pub const FPGA_SIMULATION_START: i32 = -32270;
    pub const FPGA_CONSTRAINT_START: i32 = -32280;
    pub const FPGA_ADAPTER_START: i32 = -32290;

    // asic-conductor（-32300 ~ -32399）
    pub const ASIC_START: i32 = -32300;

    // pcb-conductor（-32400 ~ -32499）
    pub const PCB_START: i32 = -32400;

    // debug-conductor（-32500 ~ -32599）
    pub const DEBUG_START: i32 = -32500;

    // rag-conductor（-32600 ~ -32699）
    pub const RAG_START: i32 = -32600;
}