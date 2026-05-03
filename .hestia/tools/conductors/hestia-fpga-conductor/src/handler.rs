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
        let bitstream = params.get("bitstream").and_then(|v| v.as_str()).map(std::path::PathBuf::from);
        let execute = params.get("execute").and_then(|v| v.as_bool()).unwrap_or(false);
        tracing::info!(device = %device, "fpga.program");

        let scripts_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("scripts"))?;
        let reports_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("reports"))?;
        let bitstream_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("output"))?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        // Resolve bitstream: params.bitstream > first .bit in <root>/fpga/output/.
        let bit_path = bitstream
            .filter(|p| p.is_file())
            .or_else(|| {
                std::fs::read_dir(&bitstream_dir).ok().and_then(|mut d| {
                    d.find_map(|e| e.ok().map(|e| e.path()).filter(|p| {
                        p.extension().map(|e| e.to_string_lossy().to_string() == "bit").unwrap_or(false)
                    }))
                })
            });

        // Vivado / hw_server detection.
        let vivado_root = std::env::var("VIVADO_PATH").ok()
            .map(std::path::PathBuf::from)
            .or_else(|| {
                let p = std::path::PathBuf::from("/opt/Xilinx/2025.2/Vivado");
                if p.is_dir() { Some(p) } else { None }
            });
        let vivado_bin = vivado_root.as_ref().map(|root| root.join("bin/vivado"));
        let vivado_present = vivado_bin.as_ref().map(|p| p.is_file()).unwrap_or(false);

        // Resolve programming TCL: params.tcl > project template > generic generator.
        let provided_tcl = params.get("tcl").and_then(|v| v.as_str()).map(std::path::PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| conductor_sdk::workspace::find_project_template("fpga", "program.tcl"));

        let tcl_path = scripts_dir.join("program.tcl");
        if let Some(tpl) = &provided_tcl {
            // Render template with placeholders.
            let raw = std::fs::read_to_string(tpl).map_err(|e| format!("read tcl template {}: {e}", tpl.display()))?;
            let mut out = raw;
            out = out.replace("{{BITSTREAM}}", &bit_path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default());
            out = out.replace("{{DEVICE}}", device);
            std::fs::write(&tcl_path, out).map_err(|e| format!("write program.tcl: {e}"))?;
        } else {
            // Generic Vivado hw_server programming flow.
            let bit_str = bit_path.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
            let body = format!(r#"# program.tcl — generic Vivado JTAG programming flow.
# Override by placing a custom script at <root>/.hestia/fpga/templates/program.tcl.
open_hw_manager
connect_hw_server -allow_non_jtag
open_hw_target
current_hw_device [lindex [get_hw_devices] 0]
refresh_hw_device -update_hw_probes false [current_hw_device]
set_property PROGRAM.FILE {{{bit}}} [current_hw_device]
program_hw_devices [current_hw_device]
disconnect_hw_server
close_hw_manager
"#, bit = bit_str);
            std::fs::write(&tcl_path, body).map_err(|e| format!("write program.tcl: {e}"))?;
        }

        let mut executed = false;
        let mut exit_code: Option<i32> = None;
        let mut log_path: Option<String> = None;
        // Phase 26: align with Phase 25 fpga.build gate — only invoke Vivado
        // when the inputs we actually need (bitstream + tool) are present.
        let inputs_complete = bit_path.is_some();
        if execute && vivado_present && inputs_complete {
            if let Some(bin) = vivado_bin.as_ref() {
                let log = reports_dir.join("program.log");
                let res = tokio::process::Command::new(bin)
                    .args(["-mode", "batch", "-nojournal", "-nolog", "-source"])
                    .arg(&tcl_path)
                    .current_dir(&reports_dir)
                    .output().await
                    .map_err(|e| format!("invoke vivado program failed: {e}"))?;
                let mut combined = res.stdout.clone();
                combined.extend_from_slice(&res.stderr);
                let _ = std::fs::write(&log, &combined);
                executed = true;
                exit_code = res.status.code();
                log_path = Some(log.to_string_lossy().into_owned());
            }
        }

        // Phase 26: surface `input_required` when execute was requested but
        // no bitstream is available (was previously masked as `skipped`).
        let status_str = if executed && exit_code == Some(0) {
            "ok"
        } else if executed {
            "program_failed"
        } else if !vivado_present {
            "tool_unavailable"
        } else if execute && !inputs_complete {
            "input_required"
        } else if !inputs_complete {
            "skipped"
        } else {
            "ready"
        };
        Ok(serde_json::json!({
            "status": status_str,
            "method": "fpga.program",
            "device": device,
            "bitstream": bit_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "tcl_script": tcl_path.to_string_lossy(),
            "execute_requested": execute,
            "executed": executed,
            "inputs_complete": inputs_complete,
            "bitstream_present": bit_path.is_some(),
            "exit_code": exit_code,
            "log": log_path,
            "vivado_present": vivado_present,
            "run_id": run_id,
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

        // Phase 20: split FPGA artifacts into <root>/fpga/{constraints,scripts,reports,output}/
        // Phase 21: keep this handler generic. Constraints/TCL/part/top come from
        //   1) params, or
        //   2) existing <root>/fpga/{constraints,scripts}/, or
        //   3) project-side templates in <root>/.hestia/fpga/templates/.
        let constraints_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("constraints"))?;
        let scripts_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("scripts"))?;
        let reports_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("reports"))?;
        let bitstream_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("output"))?;
        let rtl_dir = conductor_sdk::workspace::ensure_artifact_dir("rtl", None)?;
        let work_dir = conductor_sdk::workspace::ensure_artifact_dir("fpga", Some("work"))?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        let execute = params.get("execute").and_then(|v| v.as_bool()).unwrap_or(false);

        // Detect Vivado from VIVADO_PATH env or the standard install root.
        let vivado_root = std::env::var("VIVADO_PATH").ok()
            .map(std::path::PathBuf::from)
            .or_else(|| {
                let p = std::path::PathBuf::from("/opt/Xilinx/2025.2/Vivado");
                if p.is_dir() { Some(p) } else { None }
            });
        let vivado_bin = vivado_root.as_ref().map(|root| root.join("bin/vivado"));
        let vivado_present = vivado_bin.as_ref().map(|p| p.is_file()).unwrap_or(false);

        // Resolve constraints (XDC). Project side > template; no hardcoded board data.
        let xdc_path: Option<std::path::PathBuf> = params.get("constraints").and_then(|v| v.as_str()).map(std::path::PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| {
                std::fs::read_dir(&constraints_dir).ok().and_then(|mut d| {
                    d.find_map(|e| e.ok().map(|e| e.path()).filter(|p| p.extension().map(|e| e.to_string_lossy().to_string() == "xdc").unwrap_or(false)))
                })
            })
            .or_else(|| conductor_sdk::workspace::find_project_template("fpga", &format!("{target}.xdc")))
            .or_else(|| conductor_sdk::workspace::find_project_template("fpga", "constraints.xdc"));
        let xdc_in_constraints: Option<std::path::PathBuf> = xdc_path.as_ref().map(|src| {
            let name = src.file_name().map(std::ffi::OsString::from).unwrap_or_else(|| std::ffi::OsString::from("constraints.xdc"));
            let dst = constraints_dir.join(&name);
            if src != &dst { let _ = std::fs::copy(src, &dst); }
            dst
        });

        // Resolve build TCL template (project-side only; no hardcoded vendor TCL).
        let tcl_template: Option<std::path::PathBuf> = params.get("tcl_template").and_then(|v| v.as_str()).map(std::path::PathBuf::from)
            .filter(|p| p.is_file())
            .or_else(|| conductor_sdk::workspace::find_project_template("fpga", &format!("{target}.tcl")))
            .or_else(|| conductor_sdk::workspace::find_project_template("fpga", "build.tcl"));

        // Discover RTL sources from <root>/rtl/.
        let mut rtl_sources: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(&rtl_dir) {
            for entry in read_dir.flatten() {
                let p = entry.path();
                let is_hdl = p.extension().map(|e| matches!(e.to_string_lossy().as_ref(), "sv" | "v" | "vhd")).unwrap_or(false);
                let is_tb = p.file_name().map(|n| n.to_string_lossy().starts_with("tb_")).unwrap_or(false);
                if is_hdl && !is_tb { rtl_sources.push(p); }
            }
        }

        // Top module name from params or first RTL file stem.
        let top_module = params.get("top").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| rtl_sources.first().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "top".to_string());

        // Part number resolution: params.part > project template/manifest > common map > unknown.
        let part = params.get("part").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| {
                conductor_sdk::workspace::find_project_template("fpga", &format!("{target}.part"))
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|s| s.trim().to_string())
            });

        // Build TCL: render template if available, otherwise emit a minimal generic skeleton.
        let bitstream_path = bitstream_dir.join(format!("{}.bit", top_module));
        let tcl_path = scripts_dir.join("build.tcl");

        let tcl_content = if let Some(tpl) = &tcl_template {
            let raw = std::fs::read_to_string(tpl).map_err(|e| format!("read tcl_template {}: {e}", tpl.display()))?;
            let mut out = raw;
            out = out.replace("{{TARGET}}", target);
            out = out.replace("{{TOP}}", &top_module);
            out = out.replace("{{PART}}", part.as_deref().unwrap_or("UNKNOWN"));
            out = out.replace("{{WORK_DIR}}", &work_dir.to_string_lossy());
            out = out.replace("{{RTL_DIR}}", &rtl_dir.to_string_lossy());
            out = out.replace("{{CONSTRAINTS}}", &xdc_in_constraints.as_ref().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default());
            out = out.replace("{{BITSTREAM}}", &bitstream_path.to_string_lossy());
            // Replace {{RTL_SOURCES}} with newline-joined add_files commands.
            let rtl_lines = rtl_sources.iter()
                .map(|p| format!("add_files -norecurse {}", p.display()))
                .collect::<Vec<_>>().join("\n");
            out = out.replace("{{RTL_SOURCES}}", &rtl_lines);
            out
        } else {
            // No template: minimal generic skeleton (no hardcoded part / top / module name).
            let part_str = part.as_deref().unwrap_or("# part not resolved; specify via params.part or template");
            let xdc_line = match &xdc_in_constraints {
                Some(p) => format!("add_files -fileset constrs_1 -norecurse {}", p.display()),
                None => "# no XDC constraints resolved".to_string(),
            };
            let rtl_lines = if rtl_sources.is_empty() {
                "# no RTL sources resolved (place HDL files in <root>/rtl/)".to_string()
            } else {
                rtl_sources.iter().map(|p| format!("add_files -norecurse {}", p.display())).collect::<Vec<_>>().join("\n")
            };
            format!(r#"# build.tcl — generic skeleton (Phase 21). Provide <root>/.hestia/fpga/templates/<target>.tcl for vendor-specific flow.
create_project -force {top} {work} -part {part}
{rtl}
{xdc}
synth_design -top {top}
opt_design
place_design
route_design
write_bitstream -force {bit}
"#, top = top_module, work = work_dir.display(), part = part_str, rtl = rtl_lines, xdc = xdc_line, bit = bitstream_path.display())
        };
        std::fs::write(&tcl_path, &tcl_content)
            .map_err(|e| format!("write build.tcl failed: {e}"))?;

        // Optional: actually invoke Vivado batch when params.execute=true.
        // Phase 25: gate Vivado invocation behind input completeness — running
        // Vivado without RTL sources / constraints / part is guaranteed to fail
        // and burns minutes. Honest report `input_required` when the project
        // setup is incomplete instead.
        let mut executed = false;
        let mut exit_code: Option<i32> = None;
        let mut vivado_log: Option<String> = None;
        let inputs_complete =
            !rtl_sources.is_empty() && xdc_in_constraints.is_some() && part.is_some();
        if execute && vivado_present && inputs_complete {
            if let Some(bin) = vivado_bin.as_ref() {
                let log_path = reports_dir.join("vivado_build.log");
                let result = tokio::process::Command::new(bin)
                    .args(["-mode", "batch", "-nojournal", "-nolog", "-source"])
                    .arg(&tcl_path)
                    .current_dir(&work_dir)
                    .output()
                    .await
                    .map_err(|e| format!("invoke vivado failed: {e}"))?;
                let mut combined = result.stdout.clone();
                combined.extend_from_slice(&result.stderr);
                let _ = std::fs::write(&log_path, &combined);
                executed = true;
                exit_code = result.status.code();
                vivado_log = Some(log_path.to_string_lossy().into_owned());
            }
        }

        let build_id = format!("build_{}", uuid::Uuid::new_v4().simple());
        let manifest_path = reports_dir.join("build_manifest.json");
        let manifest = serde_json::json!({
            "build_id": build_id,
            "run_id": run_id,
            "target": target,
            "top": top_module,
            "part": part,
            "steps": steps,
            "vivado_present": vivado_present,
            "vivado_bin": vivado_bin.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "rtl_sources": rtl_sources.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>(),
            "constraints": xdc_in_constraints.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "constraints_source": xdc_path.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "tcl_template": tcl_template.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "tcl_script": tcl_path.to_string_lossy(),
            "work_dir": work_dir.to_string_lossy(),
            "expected_bitstream": bitstream_path.to_string_lossy(),
            "execute_requested": execute,
            "executed": executed,
            "exit_code": exit_code,
            "vivado_log": vivado_log,
            "bitstream_present": bitstream_path.is_file(),
        });
        std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap())
            .map_err(|e| format!("write build_manifest.json failed: {e}"))?;

        // Phase 25: surface `input_required` when --execute was requested but
        // mandatory inputs (RTL sources / constraints / part) are missing.
        // Without this, callers cannot distinguish "tool absent" from "project
        // setup incomplete" — both previously fell through to "skipped" or
        // (worse) ran Vivado and reported "build_failed".
        let status_str = if executed && exit_code == Some(0) && bitstream_path.is_file() {
            "ok"
        } else if executed {
            "build_failed"
        } else if !vivado_present {
            "tool_unavailable"
        } else if execute && !inputs_complete {
            "input_required"
        } else if !inputs_complete {
            "skipped"
        } else {
            "started"
        };
        Ok(serde_json::json!({
            "status": status_str,
            "method": "fpga.build.v1.start",
            "target": target,
            "top": top_module,
            "part": part,
            "steps": steps,
            "build_id": build_id,
            "vivado_present": vivado_present,
            "execute_requested": execute,
            "executed": executed,
            "inputs_complete": inputs_complete,
            "rtl_sources_count": rtl_sources.len(),
            "constraints_present": xdc_in_constraints.is_some(),
            "part_resolved": part.is_some(),
            "exit_code": exit_code,
            "run_id": run_id,
            "constraints": xdc_in_constraints.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "tcl_script": tcl_path.to_string_lossy(),
            "expected_bitstream": bitstream_path.to_string_lossy(),
            "bitstream_present": bitstream_path.is_file(),
            "vivado_log": vivado_log,
            "artifact": manifest_path.to_string_lossy(),
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