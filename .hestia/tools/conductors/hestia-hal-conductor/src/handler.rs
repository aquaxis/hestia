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
            "hal.design.v1" => Self::handle_design(params).await,
            "hal.dispatch_coders.v1" => Self::handle_dispatch_coders(params).await,
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

    /// Phase 55c — `hal.design.v1`: handler が直接 hal-designer へ送信し、
    /// `expected_artifacts` を ai-conductor に提示する fire-and-forget dispatch モデル。
    async fn handle_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
        let designer_peer = "hal-designer";
        let designer_alive = conductor_sdk::workspace::agent_cli_peer_alive(designer_peer);
        let expected_artifacts = vec!["hal/register_map.json"];
        if designer_alive {
            let prompt = format!(
                "[hal.design.v1] {instruction}\nfs_write hal/register_map.json with registers array (each: name/offset/fields)."
            );
            let dispatched = match conductor_sdk::workspace::agent_cli_send(designer_peer, &prompt) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!(error = %e, peer = designer_peer, "hal.design.v1: agent-cli send failed");
                    false
                }
            };
            Ok(serde_json::json!({
                "status": "delegated",
                "method": "hal.design.v1",
                "phase": "phase55c",
                "designer_peer": designer_peer,
                "designer_alive": true,
                "dispatched": dispatched,
                "expected_artifacts": expected_artifacts,
                "next_action": "ai-conductor は designer の fs_write 完了後に hal.parse.v1 を実行。",
                "instruction": instruction,
            }))
        } else {
            Ok(serde_json::json!({
                "status": "input_required",
                "method": "hal.design.v1",
                "phase": "phase55b-fallback",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "expected_artifacts": expected_artifacts,
                "fallback": "ai-conductor fs_write hal/register_map.json",
                "instruction": instruction,
                "note": "hal-designer が agent-cli registry に不在のため移行期間動作にフォールバック。ai-conductor が暫定で fs_write してください。",
            }))
        }
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

    /// Phase 60b — `hal.dispatch_coders.v1`: hal-designer の出力（言語一覧）を受けて
    /// `hal-coder-{lang}` (c/rust/python/svd 等) を動的並列起動。設計仕様書 §8.x の
    /// 「言語ごとに動的起動」を実装。
    async fn handle_dispatch_coders(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let langs: Vec<String> = params.get("languages")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");
        if langs.is_empty() {
            return Ok(serde_json::json!({
                "status": "input_required",
                "method": "hal.dispatch_coders.v1",
                "phase": "phase60b",
                "note": "languages array required (each entry becomes a `hal-coder-{lang}` peer)",
            }));
        }
        let mut spawned: Vec<String> = Vec::new();
        let mut dispatched_all = true;
        for lang in &langs {
            let peer = format!("hal-coder-{lang}");
            let r = std::process::Command::new("hestia")
                .args(["spawn-subagent", "--persona", "hal-coder", "--name", &peer])
                .output();
            match r {
                Ok(o) if o.status.success() => spawned.push(peer.clone()),
                _ => dispatched_all = false,
            }
            let prompt = format!("[hal-coder-{lang}] generate driver code: {spec}");
            if conductor_sdk::workspace::agent_cli_send(&peer, &prompt).is_err() {
                dispatched_all = false;
            }
        }
        // Phase 80: dispatch 完了後に ai-reviewer auto-spawn
        let auto_review_dispatched = conductor_sdk::workspace::auto_review_after_dispatch(
            "hal", "hal.dispatch_coders.v1", spawned.len(),
        );

        Ok(serde_json::json!({
            "status": if dispatched_all { "delegated" } else { "partial" },
            "method": "hal.dispatch_coders.v1",
            "phase": "phase60b",
            "spawned": spawned,
            "dispatched_all": dispatched_all,
            "languages_requested": langs.len(),
            "auto_review_dispatched": auto_review_dispatched,
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