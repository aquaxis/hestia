//! Debug Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// Debug Conductor メッセージハンドラ
pub struct DebugHandler;

#[async_trait::async_trait]
impl MessageHandler for DebugHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "debug.connect" => Self::handle_connect(params).await,
            "debug.disconnect" => Self::handle_disconnect(params).await,
            "debug.reset" => Self::handle_reset(params).await,
            "debug.status" => Self::handle_status().await,
            "debug.setBreakpoint" => Self::handle_set_breakpoint(params).await,
            "debug.removeBreakpoint" => Self::handle_remove_breakpoint(params).await,
            "debug.run" => Self::handle_run(params).await,
            "debug.pause" => Self::handle_pause().await,
            "debug.stepOver" => Self::handle_step_over().await,
            "debug.stepInto" => Self::handle_step_into().await,
            "debug.readMemory" => Self::handle_read_memory(params).await,
            "debug.writeMemory" => Self::handle_write_memory(params).await,
            "debug.startCapture" => Self::handle_start_capture(params).await,
            "debug.stopCapture" => Self::handle_stop_capture().await,
            "debug.read_signals" => Self::handle_read_signals(params).await,
            "debug.set_trigger" => Self::handle_set_trigger(params).await,
            "debug.program" => Self::handle_program(params).await,
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

impl DebugHandler {
    async fn handle_connect(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let protocol = params.get("protocol").and_then(|v| v.as_str()).unwrap_or("jtag");
        let device = params.get("device").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.connect",
            "protocol": protocol,
            "device": device,
            "session_id": format!("dbg_{}", uuid::Uuid::new_v4().simple()),
        }))
    }

    async fn handle_disconnect(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let session_id = params.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.disconnect",
            "session_id": session_id,
        }))
    }

    async fn handle_reset(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("hardware");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.reset",
            "mode": mode,
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.status",
            "session_state": "idle",
        }))
    }

    async fn handle_set_breakpoint(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let bp_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("source");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.setBreakpoint",
            "type": bp_type,
            "bp_id": format!("bp_{}", uuid::Uuid::new_v4().simple()),
        }))
    }

    async fn handle_remove_breakpoint(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.removeBreakpoint",
        }))
    }

    async fn handle_run(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.run",
        }))
    }

    async fn handle_pause() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.pause",
        }))
    }

    async fn handle_step_over() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.stepOver",
        }))
    }

    async fn handle_step_into() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.stepInto",
        }))
    }

    async fn handle_read_memory(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("0x00000000");
        let size = params.get("size").and_then(|v| v.as_u64()).unwrap_or(4);
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.readMemory",
            "address": address,
            "size": size,
            "data": [],
        }))
    }

    async fn handle_write_memory(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("0x00000000");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.writeMemory",
            "address": address,
        }))
    }

    async fn handle_start_capture(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let signals = params.get("signals").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.startCapture",
            "capture_id": format!("cap_{}", uuid::Uuid::new_v4().simple()),
            "signals": signals,
        }))
    }

    async fn handle_stop_capture() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.stopCapture",
        }))
    }

    async fn handle_read_signals(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.read_signals",
            "signals": {},
        }))
    }

    async fn handle_set_trigger(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.set_trigger",
        }))
    }

    async fn handle_program(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let method = params.get("method").and_then(|v| v.as_str()).unwrap_or("probe-rs");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.program",
            "flash_method": method,
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["openocd", "probe-rs", "sigrok"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}