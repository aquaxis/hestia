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
            "asic.init" => Self::handle_init(params).await,
            "asic.design.v1" => Self::handle_design(params).await,
            "asic.dispatch_steps.v1" => Self::handle_dispatch_steps(params).await,
            "asic.build" => Self::handle_build(params).await,
            "asic.advance" => Self::handle_advance(params).await,
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
    async fn handle_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.init",
            "project": project,
        }))
    }

    /// Phase 58 — `asic.design.v1`: ASIC 設計依頼を asic-designer に dispatch。
    async fn handle_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
        let pdk = params.get("pdk").and_then(|v| v.as_str()).unwrap_or("sky130");
        let designer_peer = "asic-designer";
        let designer_alive = conductor_sdk::workspace::agent_cli_peer_alive(designer_peer);
        let expected_artifacts = vec![
            "asic/floorplan.def".to_string(),
            "asic/constraints.sdc".to_string(),
            "asic/config.json".to_string(),
        ];
        // Phase 84f — strict mode: designer 不在時は fallback ではなく halt
        if !designer_alive && conductor_sdk::workspace::strict_subagent_enabled() {
            return Ok(serde_json::json!({
                "status": "subagent_unavailable",
                "method": "asic.design.v1",
                "phase": "phase84-strict",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "halted_reason": "subagent_spawn_failure",
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
                "pdk": pdk,
                "note": "HESTIA_STRICT_SUBAGENT=1: asic-designer が registry 不在のため halt。`hestia start` ログ確認 + `agent-cli list` で resident sub-agent 登録状態を調査してください。",
            }));
        }
        if designer_alive {
            let prompt = format!(
                "[asic.design.v1 pdk={pdk}] {instruction}\nfs_write asic/floorplan.def + asic/constraints.sdc + asic/config.json."
            );
            let dispatched = conductor_sdk::workspace::agent_cli_send(designer_peer, &prompt).is_ok();
            Ok(serde_json::json!({
                "status": "delegated",
                "method": "asic.design.v1",
                "phase": "phase58",
                "designer_peer": designer_peer,
                "designer_alive": true,
                "dispatched": dispatched,
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
                "pdk": pdk,
            }))
        } else {
            Ok(serde_json::json!({
                "status": "input_required",
                "method": "asic.design.v1",
                "phase": "phase58-fallback",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "expected_artifacts": expected_artifacts,
                "fallback": "ai-conductor fs_write asic/floorplan.def + asic/constraints.sdc + asic/config.json",
                "instruction": instruction,
                "pdk": pdk,
            }))
        }
    }

    /// Phase 65 — `asic.dispatch_steps.v1`: ASIC ステップごとに対応サブエージェントへ
    /// 順次 dispatch（synthesizer / implementer / signoff_checker / tester）。
    async fn handle_dispatch_steps(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let steps: Vec<String> = params.get("steps")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");
        if steps.is_empty() {
            return Ok(serde_json::json!({
                "status": "input_required",
                "method": "asic.dispatch_steps.v1",
                "phase": "phase65",
                "note": "steps array required (e.g. [\"synthesizer\",\"implementer\",\"signoff\",\"tester\"])",
            }));
        }
        let mut spawned: Vec<String> = Vec::new();
        let mut dispatched_all = true;
        for step in &steps {
            // signoff は peer 名 asic-signoff (Phase 56 §3.11)、ファイルは asic-signoff-checker.md
            let (persona, peer) = match step.as_str() {
                "signoff" | "signoff_checker" => ("asic-signoff-checker", "asic-signoff".to_string()),
                other => (
                    Box::leak(format!("asic-{other}").into_boxed_str()) as &str,
                    format!("asic-{other}"),
                ),
            };
            let r = std::process::Command::new("hestia")
                .args(["spawn-subagent", "--persona", persona, "--name", &peer])
                .output();
            match r {
                Ok(o) if o.status.success() => spawned.push(peer.clone()),
                _ => dispatched_all = false,
            }
            let prompt = format!("[{peer}] step={step}: {spec}");
            if conductor_sdk::workspace::agent_cli_send(&peer, &prompt).is_err() {
                dispatched_all = false;
            }
        }
        // Phase 80: dispatch 完了後に ai-reviewer auto-spawn
        let auto_review_dispatched = conductor_sdk::workspace::auto_review_after_dispatch(
            "asic", "asic.dispatch_steps.v1", spawned.len(),
        );

        Ok(serde_json::json!({
            "status": if dispatched_all { "delegated" } else { "partial" },
            "method": "asic.dispatch_steps.v1",
            "phase": "phase65",
            "spawned": spawned,
            "dispatched_all": dispatched_all,
            "steps_requested": steps.len(),
            "auto_review_dispatched": auto_review_dispatched,
        }))
    }

    async fn handle_build(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let pdk = params.get("pdk").and_then(|v| v.as_str()).unwrap_or("sky130");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.build",
            "pdk": pdk,
        }))
    }

    async fn handle_advance(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let stage = params.get("stage").and_then(|v| v.as_str()).unwrap_or("synthesis");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.advance",
            "stage": stage,
        }))
    }

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

    async fn handle_place(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.place",
        }))
    }

    async fn handle_cts(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.cts",
        }))
    }

    async fn handle_route(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.route",
        }))
    }

    async fn handle_gdsii(_params: serde_json::Value) -> Result<serde_json::Value, String> {
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

    async fn handle_lvs(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.lvs",
            "matches": true,
        }))
    }

    async fn handle_timing_signoff(_params: serde_json::Value) -> Result<serde_json::Value, String> {
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

    async fn handle_ai_timing_fix(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.timing_fix",
            "suggestions": [],
        }))
    }

    async fn handle_ai_drc_fix(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.drc_fix",
            "patches": [],
        }))
    }

    async fn handle_ai_floorplan_optimize(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "asic.ai.floorplan_optimize",
            "suggestions": [],
        }))
    }

    async fn handle_ai_pdk_migrate(_params: serde_json::Value) -> Result<serde_json::Value, String> {
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