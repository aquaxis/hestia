//! RTL Conductor メッセージハンドラ
//!
//! RTL ドメイン固有のメソッドをディスパッチする。

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

#[async_trait::async_trait]
impl MessageHandler for RtlHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "rtl.init" => Self::handle_init(params).await,
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

    async fn handle_lint(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let adapter = params.get("adapter").and_then(|v| v.as_str()).unwrap_or("verilator");
        let flags = params
            .get("flags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();

        // Phase 20: HDL sources go under <root>/rtl/, lint reports under <root>/sim/.
        // Phase 21: keep this handler generic. RTL sources come from
        //   1) params.sources, or
        //   2) existing <root>/rtl/*.{sv,v,vhd}, or
        //   3) project-side templates copied from <root>/.hestia/rtl/templates/
        // Hestia core does not embed any application-specific RTL.
        let rtl_dir = conductor_sdk::workspace::ensure_artifact_dir("rtl", None)?;
        let sim_dir = conductor_sdk::workspace::ensure_artifact_dir("sim", None)?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        // Resolve RTL sources (Phase 21 template resolution).
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
        if hdl_sources.is_empty() {
            // Try project-side templates: copy them into <root>/rtl/.
            let template_dir = conductor_sdk::workspace::resolve_project_root()
                .join(".hestia/rtl/templates");
            if let Ok(read_dir) = std::fs::read_dir(&template_dir) {
                for entry in read_dir.flatten() {
                    let src = entry.path();
                    let name = match src.file_name() { Some(n) => n.to_owned(), None => continue };
                    if src.extension().map(|e| matches!(e.to_string_lossy().as_ref(), "sv" | "v" | "vh" | "svh" | "vhd")).unwrap_or(false)
                        && !name.to_string_lossy().starts_with("tb_")
                    {
                        let dst = rtl_dir.join(&name);
                        let _ = std::fs::copy(&src, &dst);
                        hdl_sources.push(dst);
                    }
                }
            }
            if !hdl_sources.is_empty() { source_kind = "project_template"; }
        }

        let started = std::time::Instant::now();
        let (tool_invoked, tool_path_str, exit_code, diagnostics, lint_log_path, lint_status) = if hdl_sources.is_empty() {
            let log_path = sim_dir.join("lint.log");
            let _ = std::fs::write(&log_path, "[hestia rtl.lint.v1] no RTL sources resolved (params.sources empty, no <root>/rtl/*.sv, no <root>/.hestia/rtl/templates/)\n");
            (false, None, None, 0, Some(log_path.to_string_lossy().into_owned()), "skipped")
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

        // Resolve testbench file.
        let tb_filename = format!("{testbench}.sv");
        let mut tb_path: Option<std::path::PathBuf> = None;
        let mut dut_sources: Vec<std::path::PathBuf> = Vec::new();
        let mut source_kind = "empty";

        // 1. Existing <root>/rtl/<testbench>.sv?
        let candidate = rtl_dir.join(&tb_filename);
        if candidate.is_file() {
            tb_path = Some(candidate);
            source_kind = "project_existing";
        }
        // 2. Project template <root>/.hestia/rtl/templates/<testbench>.sv?
        if tb_path.is_none() {
            if let Some(tmpl) = conductor_sdk::workspace::find_project_template("rtl", &tb_filename) {
                let dst = rtl_dir.join(&tb_filename);
                let _ = std::fs::copy(&tmpl, &dst);
                tb_path = Some(dst);
                source_kind = "project_template";
            }
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
        // 3. If still no DUT sources, copy non-tb_*.sv templates from .hestia/rtl/templates/.
        if dut_sources.is_empty() {
            let template_dir = conductor_sdk::workspace::resolve_project_root()
                .join(".hestia/rtl/templates");
            if let Ok(read_dir) = std::fs::read_dir(&template_dir) {
                for entry in read_dir.flatten() {
                    let src = entry.path();
                    let name = match src.file_name() { Some(n) => n.to_owned(), None => continue };
                    if src.extension().map(|e| matches!(e.to_string_lossy().as_ref(), "sv" | "v")).unwrap_or(false)
                        && name.to_string_lossy() != tb_filename
                        && !name.to_string_lossy().starts_with("tb_")
                    {
                        let dst = rtl_dir.join(&name);
                        let _ = std::fs::copy(&src, &dst);
                        dut_sources.push(dst);
                    }
                }
            }
            if !dut_sources.is_empty() && source_kind == "empty" { source_kind = "project_template"; }
        }

        let started = std::time::Instant::now();
        let sim_log = sim_dir.join("sim.log");
        let waves_path = sim_dir.join("waves.vcd");
        let (tool_invoked, tool_path_str, exit_code, sim_status) = if tb_path.is_none() {
            let _ = std::fs::write(&sim_log, format!("[hestia rtl.simulate.v1] testbench '{testbench}' not found (no params, no <root>/rtl/{testbench}.sv, no template)\n"));
            (false, None, None, "skipped")
        } else if let Some(tool) = conductor_sdk::workspace::find_in_path(simulator) {
            let tb_use = tb_path.as_ref().unwrap();
            let result = if simulator == "verilator" {
                let mut cmd = tokio::process::Command::new(&tool);
                cmd.args(["--binary", "-Wall", "-o", "sim_bin", "--top-module", testbench]);
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
                    let status = if out.status.success() { "ok" } else { "sim_failed" };
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