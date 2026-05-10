//! Apps Conductor メッセージハンドラ

use std::sync::OnceLock;

use conductor_sdk::concurrency::ConductorLimiter;
use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// Apps Conductor メッセージハンドラ
pub struct AppsHandler;

/// Phase 126 — Apps conductor 配下サブエージェント並列度の上限。
/// `HESTIA_PER_CONDUCTOR_MAX` (既定 4) で設定。`HESTIA_ACQUIRE_TIMEOUT_SECS`
/// (既定 600) 経過で当該 coder 起動を skip する（デッドロック検知）。
static APPS_LIMITER: OnceLock<ConductorLimiter> = OnceLock::new();

fn apps_limiter() -> &'static ConductorLimiter {
    APPS_LIMITER.get_or_init(ConductorLimiter::from_env)
}

#[async_trait::async_trait]
impl MessageHandler for AppsHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "apps.init" => Self::handle_init(params).await,
            "apps.design.v1" => Self::handle_design(params).await,
            "apps.dispatch_coders.v1" => Self::handle_dispatch_coders(params).await,
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

    /// Phase 58 — `apps.design.v1`: Apps 設計依頼を apps-designer に dispatch。
    async fn handle_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("cortex-m");
        let designer_peer = "apps-designer";
        let designer_alive = conductor_sdk::workspace::agent_cli_peer_alive(designer_peer);
        let expected_artifacts = vec![
            "apps/main.c".to_string(),
            "apps/Cargo.toml".to_string(),
            "apps/linker.ld".to_string(),
        ];
        // Phase 84f — strict mode: designer 不在時は fallback ではなく halt
        if !designer_alive && conductor_sdk::workspace::strict_subagent_enabled() {
            return Ok(serde_json::json!({
                "status": "subagent_unavailable",
                "method": "apps.design.v1",
                "phase": "phase84-strict",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "halted_reason": "subagent_spawn_failure",
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
                "target": target,
                "note": "HESTIA_STRICT_SUBAGENT=1: apps-designer が registry 不在のため halt。`hestia start` ログ確認 + `agent-cli list` で resident sub-agent 登録状態を調査してください。",
            }));
        }
        if designer_alive {
            let prompt = format!(
                "[apps.design.v1 target={target}] {instruction}\nfs_write apps/main.c + apps/Cargo.toml + apps/linker.ld."
            );
            let dispatched = conductor_sdk::workspace::agent_cli_send(designer_peer, &prompt).is_ok();
            Ok(serde_json::json!({
                "status": "delegated",
                "method": "apps.design.v1",
                "phase": "phase58",
                "designer_peer": designer_peer,
                "designer_alive": true,
                "dispatched": dispatched,
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
                "target": target,
            }))
        } else {
            Ok(serde_json::json!({
                "status": "input_required",
                "method": "apps.design.v1",
                "phase": "phase58-fallback",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "expected_artifacts": expected_artifacts,
                "fallback": "ai-conductor fs_write apps/main.c + apps/Cargo.toml + apps/linker.ld",
                "instruction": instruction,
                "target": target,
            }))
        }
    }

    /// Phase 60b — `apps.dispatch_coders.v1`: apps-coder-{module} 動的並列起動。
    async fn handle_dispatch_coders(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let modules: Vec<String> = params.get("modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");
        if modules.is_empty() {
            return Ok(serde_json::json!({
                "status": "input_required",
                "method": "apps.dispatch_coders.v1",
                "phase": "phase60b",
                "note": "modules array required",
            }));
        }
        // Phase 126 — hardcode 16 を ConductorLimiter (env 駆動) に置換。
        // Phase 129 — per_conductor_max を **alive cap** として強制（rtl と同等、prefix=apps-coder-）。
        let limiter = apps_limiter();
        let cap = limiter.capacity();
        let alive = conductor_sdk::workspace::count_alive_peers_with_prefix("apps-coder-");
        let available_slots = cap.saturating_sub(alive);
        let max_parallel = std::cmp::min(modules.len(), available_slots);

        if max_parallel == 0 {
            tracing::warn!(
                cap = cap,
                alive = alive,
                modules_requested = modules.len(),
                "apps.dispatch_coders.v1: alive cap exhausted — skipping all spawn (Phase 129)"
            );
            return Ok(serde_json::json!({
                "status": "cap_exhausted",
                "method": "apps.dispatch_coders.v1",
                "phase": "phase129",
                "alive_coders": alive,
                "per_conductor_max": cap,
                "modules_requested": modules.len(),
                "spawned": serde_json::Value::Array(Vec::new()),
                "dispatched_all": false,
                "max_parallel": 0,
                "auto_review_dispatched": false,
                "note": "per_conductor_max に到達済の apps-coder-* が alive のため新規 spawn を skip。\
                         既存 coder の完了を待つか hestia kill で集約してください。",
            }));
        }

        tracing::info!(
            cap = cap,
            alive = alive,
            available = available_slots,
            requested = modules.len(),
            will_spawn = max_parallel,
            "apps.dispatch_coders.v1: alive cap check (Phase 129)"
        );

        let mut spawned: Vec<String> = Vec::new();
        let mut dispatched_all = true;
        for m in modules.iter().take(max_parallel) {
            let peer = format!("apps-coder-{m}");
            let _permit = match limiter.acquire().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(peer = %peer, error = %e,
                        "apps.dispatch_coders.v1: limiter acquire timeout — skipping coder");
                    dispatched_all = false;
                    continue;
                }
            };
            let r = std::process::Command::new("hestia")
                .args(["spawn-subagent", "--persona", "apps-coder", "--name", &peer])
                .output();
            match r {
                Ok(o) if o.status.success() => spawned.push(peer.clone()),
                _ => dispatched_all = false,
            }
            let prompt = format!("[apps-coder-{m}] implement module: {spec}");
            if conductor_sdk::workspace::agent_cli_send(&peer, &prompt).is_err() {
                dispatched_all = false;
            }
        }
        // Phase 80: dispatch 完了後に ai-reviewer auto-spawn
        let auto_review_dispatched = conductor_sdk::workspace::auto_review_after_dispatch(
            "apps", "apps.dispatch_coders.v1", spawned.len(),
        );

        Ok(serde_json::json!({
            "status": if dispatched_all { "delegated" } else { "partial" },
            "method": "apps.dispatch_coders.v1",
            "phase": "phase60b",
            "spawned": spawned,
            "dispatched_all": dispatched_all,
            "max_parallel": max_parallel,
            "modules_requested": modules.len(),
            "alive_coders": alive,
            "per_conductor_max": cap,
            "auto_review_dispatched": auto_review_dispatched,
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