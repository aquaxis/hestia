//! PCB Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// PCB Conductor メッセージハンドラ
pub struct PcbHandler;

#[async_trait::async_trait]
impl MessageHandler for PcbHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "pcb.init" => Self::handle_init(params).await,
            "pcb.build" => Self::handle_build(params).await,
            "pcb.generate_schematic" => Self::handle_generate_schematic(params).await,
            "pcb.ai_synthesize" => Self::handle_ai_synthesize(params).await,
            "pcb.run_drc" => Self::handle_run_drc(params).await,
            "pcb.run_erc" => Self::handle_run_erc(params).await,
            "pcb.generate_bom" => Self::handle_generate_bom(params).await,
            "pcb.place_components" => Self::handle_place_components(params).await,
            "pcb.route_traces" => Self::handle_route_traces(params).await,
            "pcb.generate_output" => Self::handle_generate_output(params).await,
            "pcb.status" => Self::handle_status().await,
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

impl PcbHandler {
    async fn handle_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.init",
            "project": project,
        }))
    }

    async fn handle_build(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.build",
            "project": project,
        }))
    }

    async fn handle_generate_schematic(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.generate_schematic",
            "project": project,
        }))
    }

    async fn handle_ai_synthesize(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");
        let input_format = params.get("input_format").and_then(|v| v.as_str()).unwrap_or("natural");
        let output_format = params.get("output_format").and_then(|v| v.as_str()).unwrap_or("kicad");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.ai_synthesize",
            "spec": spec,
            "input_format": input_format,
            "output_format": output_format,
            "confidence": 0.0,
        }))
    }

    async fn handle_run_drc(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let pcb_file = params.get("pcb_file").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.run_drc",
            "pcb_file": pcb_file,
            "violations": 0,
        }))
    }

    async fn handle_run_erc(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.run_erc",
            "violations": 0,
        }))
    }

    async fn handle_generate_bom(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.generate_bom",
            "components": 0,
        }))
    }

    async fn handle_place_components(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.place_components",
        }))
    }

    async fn handle_route_traces(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.route_traces",
        }))
    }

    async fn handle_generate_output(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("gerber");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.generate_output",
            "format": format,
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "pcb.status",
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["kicad"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}