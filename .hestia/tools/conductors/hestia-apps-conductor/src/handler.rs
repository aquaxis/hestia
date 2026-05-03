//! Apps Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// Apps Conductor メッセージハンドラ
pub struct AppsHandler;

#[async_trait::async_trait]
impl MessageHandler for AppsHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "apps.init" => Self::handle_init(params).await,
            "apps.build.v1" => Self::handle_build(params).await,
            "apps.flash.v1" => Self::handle_flash(params).await,
            "apps.test.v1" => Self::handle_test(params).await,
            "apps.size.v1" => Self::handle_size(params).await,
            "apps.debug.v1" => Self::handle_debug(params).await,
            "apps.status" => Self::handle_status().await,
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

impl AppsHandler {
    async fn handle_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "apps.init",
            "project": project,
        }))
    }

    async fn handle_build(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("thumbv7em-none-eabihf");
        let compiler = params.get("compiler").and_then(|v| v.as_str()).unwrap_or("arm-none-eabi-gcc");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "apps.build.v1",
            "project": project,
            "target": target,
            "compiler": compiler,
        }))
    }

    async fn handle_flash(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("");
        let probe = params.get("probe").and_then(|v| v.as_str()).unwrap_or("stlink-v3");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "apps.flash.v1",
            "target": target,
            "probe": probe,
        }))
    }

    async fn handle_test(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("sil");
        let probe = params.get("probe").and_then(|v| v.as_str()).unwrap_or("stlink-v3");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "apps.test.v1",
            "mode": mode,
            "probe": probe,
            "passed": 0,
            "failed": 0,
        }))
    }

    async fn handle_size(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "apps.size.v1",
            "text": 0,
            "data": 0,
            "bss": 0,
            "total_flash": 0,
            "total_ram": 0,
        }))
    }

    async fn handle_debug(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "apps.debug.v1",
            "target": target,
            "session_id": format!("dbg_{}", uuid::Uuid::new_v4().simple()),
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "apps.status",
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["arm-none-eabi-gcc", "probe-rs", "cargo-embed"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}