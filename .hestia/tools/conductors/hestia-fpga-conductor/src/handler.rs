//! FPGA Conductor メッセージハンドラ
//!
//! FPGA ドメイン固有のメソッドをディスパッチする。

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// FPGA Conductor メッセージハンドラ
pub struct FpgaHandler;

#[async_trait::async_trait]
impl MessageHandler for FpgaHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "fpga.init" => Self::handle_init(params).await,
            "fpga.synthesize" => Self::handle_synthesize(params).await,
            "fpga.implement" => Self::handle_implement(params).await,
            "fpga.bitstream" => Self::handle_bitstream(params).await,
            "fpga.simulate" => Self::handle_simulate(params).await,
            "fpga.program" => Self::handle_program(params).await,
            "fpga.build.v1.start" => Self::handle_build_start(params).await,
            "fpga.build.v1.cancel" => Self::handle_build_cancel(params).await,
            "fpga.build.v1.status" => Self::handle_build_status(params).await,
            "fpga.status" => Self::handle_status().await,
            "system.health.v1" => Self::handle_health().await,
            "system.readiness" => Self::handle_readiness().await,
            "project_open" => Self::handle_project_open(params).await,
            "project_targets" => Self::handle_project_targets(params).await,
            "report_timing" => Self::handle_report_timing(params).await,
            "report_resource" => Self::handle_report_resource(params).await,
            "report_messages" => Self::handle_report_messages(params).await,
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

impl FpgaHandler {
    async fn handle_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "fpga.init",
            "project": project,
        }))
    }

    async fn handle_synthesize(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("xilinx");
        tracing::info!(target = %target, "fpga.synthesize");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "fpga.synthesize",
            "target": target,
            "step": "synthesize",
        }))
    }

    async fn handle_implement(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("xilinx");
        tracing::info!(target = %target, "fpga.implement");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "fpga.implement",
            "target": target,
            "step": "implement",
        }))
    }

    async fn handle_bitstream(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("xilinx");
        tracing::info!(target = %target, "fpga.bitstream");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "fpga.bitstream",
            "target": target,
            "step": "bitstream",
        }))
    }

    async fn handle_simulate(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let testbench = params.get("testbench").and_then(|v| v.as_str()).unwrap_or("tb_top");
        tracing::info!(testbench = %testbench, "fpga.simulate");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "fpga.simulate",
            "testbench": testbench,
        }))
    }

    async fn handle_program(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let device = params.get("device").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!(device = %device, "fpga.program");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "fpga.program",
            "device": device,
        }))
    }

    async fn handle_build_start(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("xilinx");
        let steps = params
            .get("steps")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["synthesize".into(), "implement".into(), "bitstream".into()]);
        tracing::info!(target = %target, steps = ?steps, "fpga.build.v1.start");
        Ok(serde_json::json!({
            "status": "started",
            "method": "fpga.build.v1.start",
            "target": target,
            "steps": steps,
            "build_id": format!("build_{}", uuid::Uuid::new_v4().simple()),
        }))
    }

    async fn handle_build_cancel(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "cancelled",
            "method": "fpga.build.v1.cancel",
        }))
    }

    async fn handle_build_status(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "idle",
            "method": "fpga.build.v1.status",
            "state": "Idle",
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "fpga.status",
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["vivado", "quartus", "efinity"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }

    async fn handle_project_open(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let path = params.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "project_open",
            "path": path,
        }))
    }

    async fn handle_project_targets(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "project_targets",
            "targets": ["xc7a35t", "xc7z020", "5CEFA5F23"],
        }))
    }

    async fn handle_report_timing(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "report_timing",
            "timing_met": true,
            "worst_negative_slack": 0.0,
        }))
    }

    async fn handle_report_resource(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "report_resource",
            "lut": 0,
            "ff": 0,
            "bram": 0,
            "dsp": 0,
        }))
    }

    async fn handle_report_messages(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "report_messages",
            "messages": [],
        }))
    }
}