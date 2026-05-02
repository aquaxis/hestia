//! HAL Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

use crate::register_map::RegisterMap;
use crate::memory_map::MemoryMap;
use crate::codegen::CodeGenerator;

/// HAL Conductor メッセージハンドラ
pub struct HalHandler;

#[async_trait::async_trait]
impl MessageHandler for HalHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "hal.parse.v1" => Self::handle_parse(params).await,
            "hal.validate.v1" => Self::handle_validate(params).await,
            "hal.generate.v1" => Self::handle_generate(params).await,
            "hal.export.v1" => Self::handle_export(params).await,
            "hal.diff.v1" => Self::handle_diff(params).await,
            "system.health.v1" => Self::handle_health().await,
            "system.readiness" => Self::handle_readiness().await,
            _ => {
                return Response::Error(ErrorResultResponse {
                    error: ErrorResponse {
                        code: -32601,
                        message: format!("Method not found: {method}"),
                        data: None,
                    },
                    id,
                    trace_id,
                });
            }
        };

        match result {
            Ok(value) => Response::Success(SuccessResponse {
                result: value,
                id,
                trace_id,
            }),
            Err(msg) => Response::Error(ErrorResultResponse {
                error: ErrorResponse {
                    code: -32000,
                    message: msg,
                    data: None,
                },
                id,
                trace_id,
            }),
        }
    }
}

impl HalHandler {
    async fn handle_parse(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let input_format = params.get("input_format").and_then(|v| v.as_str()).unwrap_or("systemrdl");
        let sources = params
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();
        Ok(serde_json::json!({
            "status": "ok",
            "method": "hal.parse.v1",
            "input_format": input_format,
            "sources": sources,
            "registers_parsed": 0,
        }))
    }

    async fn handle_validate(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "hal.validate.v1",
            "valid": true,
            "warnings": [],
            "errors": [],
        }))
    }

    async fn handle_generate(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target_lang = params.get("target_lang").and_then(|v| v.as_str()).unwrap_or("rust");
        let output_path = params.get("output_path").and_then(|v| v.as_str()).unwrap_or("./generated");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "hal.generate.v1",
            "target_lang": target_lang,
            "output_path": output_path,
        }))
    }

    async fn handle_export(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("systemverilog");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "hal.export.v1",
            "format": format,
        }))
    }

    async fn handle_diff(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let baseline = params.get("baseline").and_then(|v| v.as_str()).unwrap_or("");
        let current = params.get("current").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "hal.diff.v1",
            "baseline": baseline,
            "current": current,
            "added": 0,
            "removed": 0,
            "modified": 0,
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": [],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}