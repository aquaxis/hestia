//! ASIC Conductor メッセージハンドラ
//!
//! ASIC ドメイン固有のメソッドをディスパッチする。

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// ASIC Conductor メッセージハンドラ
pub struct AsicHandler;

#[async_trait::async_trait]
impl MessageHandler for AsicHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "asic.synthesize" => Self::handle_synthesize(params).await,
            "asic.floorplan" => Self::handle_floorplan(params).await,
            "asic.place" => Self::handle_place(params).await,
            "asic.cts" => Self::handle_cts(params).await,
            "asic.route" => Self::handle_route(params).await,
            "asic.gdsii" => Self::handle_gdsii(params).await,
            "asic.drc" => Self::handle_drc(params).await,
            "asic.lvs" => Self::handle_lvs(params).await,
            "asic.timing_signoff" => Self::handle_timing_signoff(params).await,
            "asic.pdk.install" => Self::handle_pdk_install(params).await,
            "asic.pdk.list" => Self::handle_pdk_list().await,
            "asic.ai.timing_fix" => Self::handle_ai_timing_fix(params).await,
            "asic.ai.drc_fix" => Self::handle_ai_drc_fix(params).await,
            "asic.ai.floorplan_optimize" => Self::handle_ai_floorplan_optimize(params).await,
            "asic.ai.pdk_migrate" => Self::handle_ai_pdk_migrate(params).await,
            "asic.status" => Self::handle_status().await,
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

impl AsicHandler {
    async fn handle_synthesize(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let pdk = params.get("pdk").and_then(|v| v.as_str()).unwrap_or("sky130");
        let strategy = params.get("strategy").and_then(|v| v.as_str()).unwrap_or("area");
        tracing::info!(pdk = %pdk, strategy = %strategy, "asic.synthesize");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.synthesize",
            "pdk": pdk,
            "strategy": strategy,
        }))
    }

    async fn handle_floorplan(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let pdk = params.get("pdk").and_then(|v| v.as_str()).unwrap_or("sky130");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.floorplan",
            "pdk": pdk,
        }))
    }

    async fn handle_place(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.place",
        }))
    }

    async fn handle_cts(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.cts",
        }))
    }

    async fn handle_route(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.route",
        }))
    }

    async fn handle_gdsii(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.gdsii",
        }))
    }

    async fn handle_drc(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let tool = params.get("tool").and_then(|v| v.as_str()).unwrap_or("magic");
        let gds_path = params.get("gds_path").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.drc",
            "tool": tool,
            "gds_path": gds_path,
            "violations": 0,
        }))
    }

    async fn handle_lvs(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.lvs",
            "matches": true,
        }))
    }

    async fn handle_timing_signoff(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.timing_signoff",
            "timing_met": true,
        }))
    }

    async fn handle_pdk_install(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let pdk = params.get("pdk").and_then(|v| v.as_str()).unwrap_or("sky130");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.pdk.install",
            "pdk": pdk,
        }))
    }

    async fn handle_pdk_list() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.pdk.list",
            "pdks": ["sky130", "gf180mcu", "ihp-sg13g2"],
        }))
    }

    async fn handle_ai_timing_fix(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.timing_fix",
            "suggestions": [],
        }))
    }

    async fn handle_ai_drc_fix(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.drc_fix",
            "patches": [],
        }))
    }

    async fn handle_ai_floorplan_optimize(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.floorplan_optimize",
            "suggestions": [],
        }))
    }

    async fn handle_ai_pdk_migrate(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.pdk_migrate",
            "changes": [],
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "asic.status",
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["openlane", "yosys", "openroad", "magic"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}