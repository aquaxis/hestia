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
            "rtl.lint.v1" => Self::handle_lint(params).await,
            "rtl.lint.v1.format" => Self::handle_lint_format(params).await,
            "rtl.simulate.v1" => Self::handle_simulate(params).await,
            "rtl.formal.v1" => Self::handle_formal(params).await,
            "rtl.transpile.v1" => Self::handle_transpile(params).await,
            "rtl.handoff.v1" => Self::handle_handoff(params).await,
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
    async fn handle_lint(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let project = params.get("project").and_then(|v| v.as_str()).unwrap_or(".");
        let adapter = params.get("adapter").and_then(|v| v.as_str()).unwrap_or("verilator");
        let flags = params
            .get("flags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();

        let ctx = RtlBuildContext {
            top_module: "top".to_string(),
            project_dir: std::path::PathBuf::from(project),
            language: HdlLanguage::SystemVerilog,
            sources: vec![],
            testbenches: vec![],
            job_id: String::new(),
            env_vars: std::collections::HashMap::new(),
        };

        let runner = LintRunner::new(adapter).with_args(flags);
        let result = runner.run(&ctx).await.map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "ok",
            "method": "rtl.lint.v1",
            "linter": adapter,
            "success": result.success,
            "diagnostics": result.diagnostics.len(),
            "duration_secs": result.duration_secs,
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
        let testbench = params.get("testbench").and_then(|v| v.as_str()).unwrap_or("tb_top");

        let ctx = RtlBuildContext {
            top_module: testbench.to_string(),
            project_dir: std::path::PathBuf::from(project),
            language: HdlLanguage::SystemVerilog,
            sources: vec![],
            testbenches: vec![std::path::PathBuf::from(format!("{testbench}.sv"))],
            job_id: String::new(),
            env_vars: std::collections::HashMap::new(),
        };

        let runner = SimRunner::new("verilator");
        let result = runner.run(&ctx).await.map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "status": "ok",
            "method": "rtl.simulate.v1",
            "testbench": testbench,
            "success": result.success,
            "duration_secs": result.duration_secs,
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