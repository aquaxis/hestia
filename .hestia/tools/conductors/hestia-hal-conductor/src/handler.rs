//! HAL Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

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
            "hal.init" => Self::handle_init(params).await,
            "hal.parse.v1" => Self::handle_parse(params).await,
            "hal.validate.v1" => Self::handle_validate(params).await,
            "hal.generate.v1" => Self::handle_generate(params).await,
            "hal.export.v1" => Self::handle_export(params).await,
            "hal.diff.v1" => Self::handle_diff(params).await,
            "hal.status" => Self::handle_status().await,
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
    async fn handle_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "hal.init",
            "project": project,
        }))
    }

    async fn handle_parse(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let input_format = params.get("input_format").and_then(|v| v.as_str()).unwrap_or("systemrdl");
        let sources = params
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();

        // Project-facing artifacts go under <root>/hal/ (Phase 20).
        let artifact_dir = conductor_sdk::workspace::ensure_artifact_dir("hal", None)?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        // Phase 42: agent-driven generation. Hestia core never falls back to a
        // template. The AI orchestrator must fs_write the register map (or
        // pass it via params.sources) before invoking this handler. If
        // neither input is available, the handler reports `input_required`
        // so the operator knows the orchestrator skipped the design step.
        // Resolution order: params.sources > existing <root>/hal/*.json > input_required
        let artifact_path = artifact_dir.join("register_map.json");
        let mut source_kind = "empty";
        let mut source_path: Option<String> = None;

        let payload: serde_json::Value = if let Some(first_source) = sources.first() {
            source_kind = "params.sources";
            source_path = Some(first_source.clone());
            match std::fs::read_to_string(first_source) {
                Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({
                    "raw": text,
                    "input_format": input_format,
                    "note": format!("Loaded from {} (parser stub, raw text retained)", first_source)
                })),
                Err(e) => return Err(format!("read {first_source} failed: {e}")),
            }
        } else if let Some(existing) = conductor_sdk::workspace::find_project_file("hal", None, "register_map.json") {
            source_kind = "project_existing";
            source_path = Some(existing.to_string_lossy().into_owned());
            let text = std::fs::read_to_string(&existing).map_err(|e| format!("read {}: {e}", existing.display()))?;
            serde_json::from_str(&text).unwrap_or(serde_json::json!({"raw": text}))
        } else {
            serde_json::json!({
                "registers": [],
                "note": "No params.sources and no <root>/hal/register_map.json. The AI orchestrator must design and fs_write the register map before invoking hal.parse — Hestia does not load templates."
            })
        };

        std::fs::write(&artifact_path, serde_json::to_string_pretty(&payload).unwrap())
            .map_err(|e| format!("write register_map.json failed: {e}"))?;

        let registers_count = payload["registers"].as_array().map(|a| a.len()).unwrap_or(0);
        // Phase 42: distinguish "agent did not generate" from generic skip.
        // input_required signals the AI should design and fs_write first.
        let status = if source_kind == "empty" { "input_required" } else { "ok" };
        Ok(serde_json::json!({
            "status": status,
            "method": "hal.parse.v1",
            "input_format": input_format,
            "sources": sources,
            "registers_parsed": registers_count,
            "run_id": run_id,
            "source_kind": source_kind,
            "source_path": source_path,
            "artifact": artifact_path.to_string_lossy(),
            "artifact_dir": artifact_dir.to_string_lossy(),
        }))
    }

    async fn handle_validate(_params: serde_json::Value) -> Result<serde_json::Value, String> {
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

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "hal.status",
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