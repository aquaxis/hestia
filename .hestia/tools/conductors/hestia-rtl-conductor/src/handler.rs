//! RTL Conductor メッセージハンドラ
//!
//! RTL ドメイン固有のメソッドをディスパッチする。

use std::sync::OnceLock;

use conductor_sdk::concurrency::ConductorLimiter;
use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

use crate::adapter::RtlBuildContext;
use crate::formal::FormalRunner;
use crate::handoff::HandoffManager;
use crate::language::HdlLanguage;
use crate::lint::LintRunner;
use crate::simulation::SimRunner;

/// RTL Conductor メッセージハンドラ
pub struct RtlHandler;

/// Phase 126 — RTL conductor 配下サブエージェント並列度の上限。
/// `HESTIA_PER_CONDUCTOR_MAX` (既定 4) で設定。`HESTIA_ACQUIRE_TIMEOUT_SECS`
/// (既定 600) 経過で `dispatch_coders.v1` の当該 coder 起動を skip する。
static RTL_LIMITER: OnceLock<ConductorLimiter> = OnceLock::new();

fn rtl_limiter() -> &'static ConductorLimiter {
    RTL_LIMITER.get_or_init(ConductorLimiter::from_env)
}

#[async_trait::async_trait]
impl MessageHandler for RtlHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "rtl.init" => Self::handle_init(params).await,
            "rtl.design.v1" => Self::handle_design(params).await,
            "rtl.dispatch_coders.v1" => Self::handle_dispatch_coders(params).await,
            "rtl.lint.v1" => Self::handle_lint(params).await,
            "rtl.lint.v1.format" => Self::handle_lint_format(params).await,
            "rtl.simulate.v1" => Self::handle_simulate(params).await,
            "rtl.formal.v1" => Self::handle_formal(params).await,
            "rtl.transpile.v1" => Self::handle_transpile(params).await,
            "rtl.handoff.v1" => Self::handle_handoff(params).await,
            "rtl.status" => Self::handle_status().await,
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

impl RtlHandler {
    async fn handle_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rtl.init",
            "project": project,
        }))
    }

    /// Phase 55c — `rtl.design.v1`: handler が直接 rtl-designer へ送信し、
    /// `expected_artifacts` を ai-conductor に提示する fire-and-forget dispatch モデル。
    /// designer 不在時は `phase55b-fallback` で ai-conductor 暫定 fs_write へフォールバック。
    async fn handle_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
        let designer_peer = "rtl-designer";
        let designer_alive = conductor_sdk::workspace::agent_cli_peer_alive(designer_peer);
        let expected_artifacts = vec!["rtl/<top>.sv", "rtl/tb_<top>.sv"];
        // Phase 84f — strict mode: designer 不在時は fallback ではなく halt
        if !designer_alive && conductor_sdk::workspace::strict_subagent_enabled() {
            return Ok(serde_json::json!({
                "status": "subagent_unavailable",
                "method": "rtl.design.v1",
                "phase": "phase84-strict",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "halted_reason": "subagent_spawn_failure",
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
                "note": "HESTIA_STRICT_SUBAGENT=1: rtl-designer が registry 不在のため halt。`hestia start` ログ確認 + `agent-cli list` で resident sub-agent 登録状態を調査してください。",
            }));
        }
        if designer_alive {
            let prompt = format!(
                "[rtl.design.v1] {instruction}\nfs_write rtl/<top>.sv (SystemVerilog top module) and rtl/tb_<top>.sv (testbench)."
            );
            let dispatched = match conductor_sdk::workspace::agent_cli_send(designer_peer, &prompt) {
                Ok(()) => true,
                Err(e) => {
                    tracing::warn!(error = %e, peer = designer_peer, "rtl.design.v1: agent-cli send failed");
                    false
                }
            };
            Ok(serde_json::json!({
                "status": "delegated",
                "method": "rtl.design.v1",
                "phase": "phase55c",
                "designer_peer": designer_peer,
                "designer_alive": true,
                "dispatched": dispatched,
                "expected_artifacts": expected_artifacts,
                "next_action": "ai-conductor は designer の fs_write 完了後に rtl.lint.v1 / rtl.simulate.v1 を実行。expected_artifacts ファイルの存在を fs_read で確認可。",
                "instruction": instruction,
            }))
        } else {
            Ok(serde_json::json!({
                "status": "input_required",
                "method": "rtl.design.v1",
                "phase": "phase55b-fallback",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "expected_artifacts": expected_artifacts,
                "fallback": "ai-conductor fs_write rtl/<top>.sv + rtl/tb_<top>.sv",
                "instruction": instruction,
                "note": "rtl-designer が agent-cli registry に不在のため移行期間動作にフォールバック。ai-conductor が暫定で fs_write してください。",
            }))
        }
    }

    /// Phase 60 — `rtl.dispatch_coders.v1`: rtl-designer の出力（モジュール一覧）を
    /// 受けて `rtl-coder-{module}` を動的並列起動する。設計仕様書 §4.8 の
    /// 「N 個の coder を並列起動・割当」を Hestia ランタイムで実装する経路。
    ///
    /// params:
    ///   modules: ["uart_rx", "uart_tx", "led_ctrl", ...]
    ///   spec: 各 coder に渡す設計仕様（natural language または JSON）
    /// returns:
    ///   spawned: ["rtl-coder-uart_rx", ...]
    ///   dispatched: bool (全 coder への送信成否の AND)
    async fn handle_dispatch_coders(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let modules: Vec<String> = params.get("modules")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");

        if modules.is_empty() {
            return Ok(serde_json::json!({
                "status": "input_required",
                "method": "rtl.dispatch_coders.v1",
                "phase": "phase60",
                "note": "modules array required (each entry becomes a `rtl-coder-{module}` peer)",
            }));
        }

        // Phase 126 — 設計 §4.8 の hardcode 16 を ConductorLimiter (env 駆動) に置換。
        // Phase 129 — per_conductor_max を **alive cap** として強制する。
        // engine registry を query して既存 alive な rtl-coder-* 数を取得し、
        // 残 slot だけ spawn する（複数 dispatch 呼出を跨いだ累積 alive 抑制）。
        let limiter = rtl_limiter();
        let cap = limiter.capacity();
        let alive = conductor_sdk::workspace::count_alive_peers_with_prefix("rtl-coder-");
        let available_slots = cap.saturating_sub(alive);
        let max_parallel = std::cmp::min(modules.len(), available_slots);

        if max_parallel == 0 {
            tracing::warn!(
                cap = cap,
                alive = alive,
                modules_requested = modules.len(),
                "rtl.dispatch_coders.v1: alive cap exhausted — skipping all spawn (Phase 129)"
            );
            return Ok(serde_json::json!({
                "status": "cap_exhausted",
                "method": "rtl.dispatch_coders.v1",
                "phase": "phase129",
                "alive_coders": alive,
                "per_conductor_max": cap,
                "modules_requested": modules.len(),
                "spawned": serde_json::Value::Array(Vec::new()),
                "dispatched_all": false,
                "max_parallel": 0,
                "auto_review_dispatched": false,
                "note": "per_conductor_max に到達済の rtl-coder-* が alive のため新規 spawn を skip。\
                         既存 coder の完了を待つか hestia kill で集約してください。",
            }));
        }

        tracing::info!(
            cap = cap,
            alive = alive,
            available = available_slots,
            requested = modules.len(),
            will_spawn = max_parallel,
            "rtl.dispatch_coders.v1: alive cap check (Phase 129)"
        );

        let mut spawned: Vec<String> = Vec::new();
        let mut dispatched_all = true;

        for module in modules.iter().take(max_parallel) {
            let peer = format!("rtl-coder-{module}");

            let _permit = match limiter.acquire().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(peer = %peer, error = %e,
                        "rtl.dispatch_coders.v1: limiter acquire timeout — skipping coder");
                    dispatched_all = false;
                    continue;
                }
            };

            // hestia spawn-subagent --persona rtl-coder --name rtl-coder-{module}
            let spawn_result = std::process::Command::new("hestia")
                .args(["spawn-subagent", "--persona", "rtl-coder", "--name", &peer])
                .output();
            match spawn_result {
                Ok(out) if out.status.success() => spawned.push(peer.clone()),
                Ok(out) => {
                    tracing::warn!(peer = %peer, stderr = %String::from_utf8_lossy(&out.stderr),
                        "rtl.dispatch_coders.v1: spawn failed");
                    dispatched_all = false;
                }
                Err(e) => {
                    tracing::warn!(peer = %peer, error = %e,
                        "rtl.dispatch_coders.v1: hestia binary not found in PATH");
                    dispatched_all = false;
                }
            }

            // 各 coder に spec を送信（設計 §4.8 並列開発フロー Step 3）
            let prompt = format!("[rtl-coder-{module}] implement module: {spec}");
            if conductor_sdk::workspace::agent_cli_send(&peer, &prompt).is_err() {
                dispatched_all = false;
            }
        }

        // Phase 80: dispatch 完了後に ai-reviewer auto-spawn
        let auto_review_dispatched = conductor_sdk::workspace::auto_review_after_dispatch(
            "rtl", "rtl.dispatch_coders.v1", spawned.len(),
        );

        Ok(serde_json::json!({
            "status": if dispatched_all { "delegated" } else { "partial" },
            "method": "rtl.dispatch_coders.v1",
            "phase": "phase60",
            "spawned": spawned,
            "dispatched_all": dispatched_all,
            "max_parallel": max_parallel,
            "modules_requested": modules.len(),
            "alive_coders": alive,
            "per_conductor_max": cap,
            "auto_review_dispatched": auto_review_dispatched,
        }))
    }

    async fn handle_lint(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let adapter = params.get("adapter").and_then(|v| v.as_str()).unwrap_or("verilator");
        let flags = params
            .get("flags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();

        // Phase 20: HDL sources go under <root>/rtl/, lint reports under <root>/sim/.
        // Phase 42: agent-driven generation. RTL sources come from
        //   1) params.sources, or
        //   2) existing <root>/rtl/*.{sv,v,vhd}
        // The AI orchestrator must fs_write the design before lint. Template
        // fallback was removed — Hestia is not a template-substitution engine.
        let rtl_dir = conductor_sdk::workspace::ensure_artifact_dir("rtl", None)?;
        let sim_dir = conductor_sdk::workspace::ensure_artifact_dir("sim", None)?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        let mut hdl_sources: Vec<std::path::PathBuf> = params
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(std::path::PathBuf::from)).collect())
            .unwrap_or_default();

        let mut source_kind = "empty";
        if hdl_sources.is_empty() {
            // Try existing project HDL files.
            for ext in &["sv", "v", "vhd"] {
                if let Ok(read_dir) = std::fs::read_dir(&rtl_dir) {
                    for entry in read_dir.flatten() {
                        let p = entry.path();
                        if p.extension().map(|e| e.to_string_lossy().to_string() == *ext).unwrap_or(false)
                            && !p.file_name().map(|n| n.to_string_lossy().starts_with("tb_")).unwrap_or(false)
                        {
                            hdl_sources.push(p);
                        }
                    }
                }
            }
            if !hdl_sources.is_empty() { source_kind = "project_existing"; }
        }

        let started = std::time::Instant::now();
        let (tool_invoked, tool_path_str, exit_code, diagnostics, lint_log_path, lint_status) = if hdl_sources.is_empty() {
            let log_path = sim_dir.join("lint.log");
            let _ = std::fs::write(&log_path, "[hestia rtl.lint.v1] no RTL sources resolved (params.sources empty, no <root>/rtl/*.{sv,v,vhd}). The AI orchestrator must fs_write the RTL design before invoking rtl.lint.\n");
            (false, None, None, 0, Some(log_path.to_string_lossy().into_owned()), "input_required")
        } else if let Some(tool) = conductor_sdk::workspace::find_in_path(adapter) {
            let mut cmd = tokio::process::Command::new(&tool);
            if adapter == "verilator" {
                cmd.arg("--lint-only").arg("-Wall");
            }
            for f in &flags { cmd.arg(f); }
            for s in &hdl_sources { cmd.arg(s); }
            let result = cmd.output().await
                .map_err(|e| format!("invoke {adapter} failed: {e}"))?;
            let log_path = sim_dir.join("lint.log");
            let _ = std::fs::write(&log_path, &result.stderr);
            let stderr_text = String::from_utf8_lossy(&result.stderr);
            let diag_count = stderr_text.matches("%Warning").count()
                + stderr_text.matches("%Error").count();
            let status_str = if result.status.success() { "ok" } else { "lint_failed" };
            (true, Some(tool.to_string_lossy().into_owned()), result.status.code(), diag_count, Some(log_path.to_string_lossy().into_owned()), status_str)
        } else {
            let log_path = sim_dir.join("lint.log");
            let _ = std::fs::write(&log_path, format!("[hestia rtl.lint.v1] linter '{adapter}' not found in PATH\n"));
            (false, None, None, 0, Some(log_path.to_string_lossy().into_owned()), "tool_unavailable")
        };
        let duration_secs = started.elapsed().as_secs_f64();

        // Drive the legacy LintRunner for fsm-state continuity (no-op stub today).
        let ctx = RtlBuildContext {
            top_module: "top".to_string(),
            project_dir: std::path::PathBuf::from(project),
            language: HdlLanguage::SystemVerilog,
            sources: hdl_sources.clone(),
            testbenches: vec![],
            job_id: run_id.clone(),
            env_vars: std::collections::HashMap::new(),
        };
        let _ = LintRunner::new(adapter).with_args(flags.clone()).run(&ctx).await;

        let report_path = sim_dir.join("lint_report.json");
        let report = serde_json::json!({
            "run_id": run_id,
            "method": "rtl.lint.v1",
            "project": project,
            "adapter": adapter,
            "flags": flags,
            "source_kind": source_kind,
            "sources": hdl_sources.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>(),
            "tool_invoked": tool_invoked,
            "tool_path": tool_path_str,
            "exit_code": exit_code,
            "diagnostics": diagnostics,
            "log": lint_log_path,
            "duration_secs": duration_secs,
            "lint_status": lint_status,
        });
        std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("write lint_report.json failed: {e}"))?;

        Ok(serde_json::json!({
            "status": lint_status,
            "method": "rtl.lint.v1",
            "linter": adapter,
            "tool_invoked": tool_invoked,
            "success": tool_invoked && exit_code == Some(0),
            "diagnostics": diagnostics,
            "duration_secs": duration_secs,
            "run_id": run_id,
            "source_kind": source_kind,
            "sources": hdl_sources.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>(),
            "artifact": report_path.to_string_lossy(),
            "rtl_dir": rtl_dir.to_string_lossy(),
            "sim_dir": sim_dir.to_string_lossy(),
        }))
    }

    async fn handle_lint_format(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rtl.lint.v1.format",
            "project": project,
            "formatted": true,
        }))
    }

    async fn handle_simulate(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let testbench = params.get("testbench").and_then(|v| v.as_str()).unwrap_or("tb_uart_led");
        let simulator = params.get("simulator").and_then(|v| v.as_str()).unwrap_or("verilator");

        // Phase 20: testbench under <root>/rtl/, sim outputs under <root>/sim/.
        // Phase 21: keep this handler generic. Testbench/DUT come from
        //   1) params.testbench / params.sources, or
        //   2) existing <root>/rtl/{tb_*.sv, *.sv}, or
        //   3) project-side templates copied from <root>/.hestia/rtl/templates/.
        let rtl_dir = conductor_sdk::workspace::ensure_artifact_dir("rtl", None)?;
        let sim_dir = conductor_sdk::workspace::ensure_artifact_dir("sim", None)?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        // Phase 42: agent-driven generation. Testbench and DUT sources come
        // only from params or existing project files — no template fallback.
        let tb_filename = format!("{testbench}.sv");
        let mut tb_path: Option<std::path::PathBuf> = None;
        let mut dut_sources: Vec<std::path::PathBuf> = Vec::new();
        let mut source_kind = "empty";

        // Existing <root>/rtl/<testbench>.sv?
        let candidate = rtl_dir.join(&tb_filename);
        if candidate.is_file() {
            tb_path = Some(candidate);
            source_kind = "project_existing";
        }

        // Discover DUT sources from <root>/rtl/*.{sv,v} excluding the testbench file.
        if let Ok(read_dir) = std::fs::read_dir(&rtl_dir) {
            for entry in read_dir.flatten() {
                let p = entry.path();
                let is_hdl = p.extension().map(|e| matches!(e.to_string_lossy().as_ref(), "sv" | "v")).unwrap_or(false);
                let is_tb = p.file_name().map(|n| n.to_string_lossy().to_string() == tb_filename).unwrap_or(false);
                if is_hdl && !is_tb { dut_sources.push(p); }
            }
        }

        let started = std::time::Instant::now();
        let sim_log = sim_dir.join("sim.log");
        let waves_path = sim_dir.join("waves.vcd");
        // warnings_count is captured for the verilator branch so we can return
        // sim_warnings (Phase 50) instead of conflating it with sim_failed.
        let mut warnings_count: usize = 0;
        let (tool_invoked, tool_path_str, exit_code, sim_status) = if tb_path.is_none() {
            let _ = std::fs::write(&sim_log, format!("[hestia rtl.simulate.v1] testbench '{testbench}' not found at <root>/rtl/{testbench}.sv. The AI orchestrator must fs_write the testbench (and DUT modules) before invoking rtl.simulate — Hestia does not load templates.\n"));
            (false, None, None, "input_required")
        } else if let Some(tool) = conductor_sdk::workspace::find_in_path(simulator) {
            let tb_use = tb_path.as_ref().unwrap();
            let result = if simulator == "verilator" {
                // Phase 50: pass `--Wno-fatal` so cosmetic warnings (EOFNEWLINE,
                // WIDTHTRUNC, WIDTHEXPAND, UNUSEDSIGNAL etc.) are reported but
                // do not exit non-zero. Real syntax/elaboration errors still
                // fail the run. The rtl.lint pass (separate handler invocation)
                // is the canonical place for strict warning enforcement.
                let mut cmd = tokio::process::Command::new(&tool);
                cmd.args([
                    "--binary",
                    "-Wall",
                    "--Wno-fatal",
                    "-o",
                    "sim_bin",
                    "--top-module",
                    testbench,
                ]);
                cmd.arg(tb_use);
                for s in &dut_sources { cmd.arg(s); }
                cmd.current_dir(&sim_dir);
                cmd.output().await
            } else if simulator == "iverilog" {
                let mut cmd = tokio::process::Command::new(&tool);
                cmd.args(["-o", "sim.out", "-s", testbench]);
                cmd.arg(tb_use);
                for s in &dut_sources { cmd.arg(s); }
                cmd.current_dir(&sim_dir);
                cmd.output().await
            } else {
                tokio::process::Command::new(&tool).arg("--version").output().await
            };
            match result {
                Ok(out) => {
                    let _ = std::fs::write(&sim_log, &out.stderr);
                    if simulator == "verilator" {
                        let stderr_text = String::from_utf8_lossy(&out.stderr);
                        warnings_count = stderr_text.matches("%Warning").count();
                    }
                    // Phase 50 status logic:
                    //  - exit != 0  → sim_failed (real error)
                    //  - exit == 0 + warnings_count > 0 → sim_warnings (compiled but flagged)
                    //  - exit == 0 + no warnings → ok
                    let status = if !out.status.success() {
                        "sim_failed"
                    } else if warnings_count > 0 {
                        "sim_warnings"
                    } else {
                        "ok"
                    };
                    (true, Some(tool.to_string_lossy().into_owned()), out.status.code(), status)
                }
                Err(e) => {
                    let _ = std::fs::write(&sim_log, format!("invoke {simulator} failed: {e}\n"));
                    (false, None, None, "sim_failed")
                }
            }
        } else {
            let _ = std::fs::write(&sim_log, format!("[hestia rtl.simulate.v1] simulator '{simulator}' not found in PATH\n"));
            (false, None, None, "tool_unavailable")
        };
        let duration_secs = started.elapsed().as_secs_f64();

        // Drive the legacy SimRunner for fsm-state continuity (no-op stub today).
        let ctx = RtlBuildContext {
            top_module: testbench.to_string(),
            project_dir: std::path::PathBuf::from(project),
            language: HdlLanguage::SystemVerilog,
            sources: dut_sources.clone(),
            testbenches: tb_path.clone().map(|p| vec![p]).unwrap_or_default(),
            job_id: run_id.clone(),
            env_vars: std::collections::HashMap::new(),
        };
        let _ = SimRunner::new(simulator).run(&ctx).await;

        let report_path = sim_dir.join("sim_report.json");
        let report = serde_json::json!({
            "run_id": run_id,
            "method": "rtl.simulate.v1",
            "project": project,
            "testbench": testbench,
            "simulator": simulator,
            "source_kind": source_kind,
            "testbench_path": tb_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "dut_sources": dut_sources.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>(),
            "tool_invoked": tool_invoked,
            "tool_path": tool_path_str,
            "exit_code": exit_code,
            "log": sim_log.to_string_lossy(),
            "waves_path": waves_path.to_string_lossy(),
            "waves_present": waves_path.exists(),
            "duration_secs": duration_secs,
            "sim_status": sim_status,
        });
        std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("write sim_report.json failed: {e}"))?;

        Ok(serde_json::json!({
            "status": sim_status,
            "method": "rtl.simulate.v1",
            "testbench": testbench,
            "simulator": simulator,
            "tool_invoked": tool_invoked,
            "success": tool_invoked && exit_code == Some(0),
            "warnings": warnings_count,
            "duration_secs": duration_secs,
            "run_id": run_id,
            "source_kind": source_kind,
            "testbench_path": tb_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "artifact": report_path.to_string_lossy(),
            "rtl_dir": rtl_dir.to_string_lossy(),
            "sim_dir": sim_dir.to_string_lossy(),
        }))
    }

    async fn handle_formal(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let tool = params.get("tool").and_then(|v| v.as_str()).unwrap_or("symbiyosys");

        let ctx = RtlBuildContext {
            top_module: "top".to_string(),
            project_dir: std::path::PathBuf::from(project),
            language: HdlLanguage::SystemVerilog,
            sources: vec![],
            testbenches: vec![],
            job_id: String::new(),
            env_vars: std::collections::HashMap::new(),
        };

        let runner = FormalRunner::new(tool);
        let result = runner.run(&ctx).await.map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "ok",
            "method": "rtl.formal.v1",
            "tool": tool,
            "success": result.success,
            "duration_secs": result.duration_secs,
        }))
    }

    async fn handle_transpile(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let source_lang = params.get("source_lang").and_then(|v| v.as_str()).unwrap_or("chisel");
        let target_lang = params.get("target_lang").and_then(|v| v.as_str()).unwrap_or("verilog");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rtl.transpile.v1",
            "source_lang": source_lang,
            "target_lang": target_lang,
        }))
    }

    async fn handle_handoff(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let target = params.get("target").and_then(|v| v.as_str()).unwrap_or("fpga");

        let ctx = RtlBuildContext {
            top_module: "top".to_string(),
            project_dir: std::path::PathBuf::from(project),
            language: HdlLanguage::SystemVerilog,
            sources: vec![],
            testbenches: vec![],
            job_id: String::new(),
            env_vars: std::collections::HashMap::new(),
        };

        let manager = HandoffManager::new(std::path::PathBuf::from(project));
        match target {
            "fpga" => {
                let result = manager.handoff_to_fpga(&ctx).await.map_err(|e| e.to_string())?;
                Ok(serde_json::json!({
                    "status": "ok",
                    "method": "rtl.handoff.v1",
                    "target": "fpga",
                    "artifact_dir": result.artifact_dir.to_string_lossy(),
                }))
            }
            "asic" => {
                let result = manager.handoff_to_asic(&ctx).await.map_err(|e| e.to_string())?;
                Ok(serde_json::json!({
                    "status": "ok",
                    "method": "rtl.handoff.v1",
                    "target": "asic",
                    "artifact_dir": result.artifact_dir.to_string_lossy(),
                }))
            }
            _ => Err(format!("Unknown handoff target: {target}")),
        }
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "rtl.status",
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["verilator", "svlint", "symbiyosys"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}