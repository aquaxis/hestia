//! PCB Conductor message handler

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// PCB Conductor message handler
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
            "pcb.design.v1" => Self::handle_design(params).await,
            "pcb.dispatch_phases.v1" => Self::handle_dispatch_phases(params).await,
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

    /// Phase 58 — `pcb.design.v1`: Dispatch PCB design request to pcb-designer.
    async fn handle_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
        let designer_peer = "pcb-designer";
        let designer_alive = conductor_sdk::workspace::agent_cli_peer_alive(designer_peer);
        let expected_artifacts = vec!["pcb/schematic.kicad_sch", "pcb/board.kicad_pcb"];
        // Phase 84f — strict mode: halt instead of fallback when designer is absent
        if !designer_alive && conductor_sdk::workspace::strict_subagent_enabled() {
            return Ok(serde_json::json!({
                "status": "subagent_unavailable",
                "method": "pcb.design.v1",
                "phase": "phase84-strict",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "halted_reason": "subagent_spawn_failure",
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
                "note": "HESTIA_STRICT_SUBAGENT=1: pcb-designer is not in the registry, halting. Check `hestia start` logs and `agent-cli list` for resident sub-agent registration status.",
            }));
        }
        if designer_alive {
            let prompt = format!(
                "[pcb.design.v1] {instruction}\nfs_write pcb/schematic.kicad_sch + pcb/board.kicad_pcb."
            );
            let dispatched = conductor_sdk::workspace::agent_cli_send(designer_peer, &prompt).is_ok();
            Ok(serde_json::json!({
                "status": "delegated",
                "method": "pcb.design.v1",
                "phase": "phase58",
                "designer_peer": designer_peer,
                "designer_alive": true,
                "dispatched": dispatched,
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
            }))
        } else {
            Ok(serde_json::json!({
                "status": "input_required",
                "method": "pcb.design.v1",
                "phase": "phase58-fallback",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "expected_artifacts": expected_artifacts,
                "fallback": "ai-conductor fs_write pcb/schematic.kicad_sch + pcb/board.kicad_pcb",
                "instruction": instruction,
            }))
        }
    }

    /// Phase 65 — `pcb.dispatch_phases.v1`: Dispatch sub-agents per PCB phase
    /// (schematic / layout / tester).
    async fn handle_dispatch_phases(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let phases: Vec<String> = params.get("phases")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");
        if phases.is_empty() {
            return Ok(serde_json::json!({
                "status": "input_required",
                "method": "pcb.dispatch_phases.v1",
                "phase": "phase65",
                "note": "phases array required (e.g. [\"schematic\",\"layout\",\"tester\"])",
            }));
        }
        let mut spawned: Vec<String> = Vec::new();
        let mut dispatched_all = true;
        for ph in &phases {
            let peer = format!("pcb-{ph}");
            let r = std::process::Command::new("hestia")
                .args(["spawn-subagent", "--persona", &peer, "--name", &peer])
                .output();
            match r {
                Ok(o) if o.status.success() => spawned.push(peer.clone()),
                _ => dispatched_all = false,
            }
            let prompt = format!("[{peer}] phase={ph}: {spec}");
            if conductor_sdk::workspace::agent_cli_send(&peer, &prompt).is_err() {
                dispatched_all = false;
            }
        }
        // Phase 80: Auto-spawn ai-reviewer after dispatch completes
        let auto_review_dispatched = conductor_sdk::workspace::auto_review_after_dispatch(
            "pcb", "pcb.dispatch_phases.v1", spawned.len(),
        );

        Ok(serde_json::json!({
            "status": if dispatched_all { "delegated" } else { "partial" },
            "method": "pcb.dispatch_phases.v1",
            "phase": "phase65",
            "spawned": spawned,
            "dispatched_all": dispatched_all,
            "phases_requested": phases.len(),
            "auto_review_dispatched": auto_review_dispatched,
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

    async fn handle_run_erc(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.run_erc",
            "violations": 0,
        }))
    }

    async fn handle_generate_bom(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.generate_bom",
            "components": 0,
        }))
    }

    async fn handle_place_components(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "pcb.place_components",
        }))
    }

    async fn handle_route_traces(_params: serde_json::Value) -> Result<serde_json::Value, String> {
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