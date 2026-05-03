//! Debug Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// Debug Conductor メッセージハンドラ
pub struct DebugHandler;

#[async_trait::async_trait]
impl MessageHandler for DebugHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "debug.create" => Self::handle_create(params).await,
            "debug.connect" => Self::handle_connect(params).await,
            "debug.disconnect" => Self::handle_disconnect(params).await,
            "debug.reset" => Self::handle_reset(params).await,
            "debug.status" => Self::handle_status().await,
            "debug.setBreakpoint" => Self::handle_set_breakpoint(params).await,
            "debug.removeBreakpoint" => Self::handle_remove_breakpoint(params).await,
            "debug.run" => Self::handle_run(params).await,
            "debug.pause" => Self::handle_pause().await,
            "debug.stepOver" => Self::handle_step_over().await,
            "debug.stepInto" => Self::handle_step_into().await,
            "debug.readMemory" => Self::handle_read_memory(params).await,
            "debug.writeMemory" => Self::handle_write_memory(params).await,
            "debug.startCapture" => Self::handle_start_capture(params).await,
            "debug.stopCapture" => Self::handle_stop_capture().await,
            "debug.read_signals" => Self::handle_read_signals(params).await,
            "debug.set_trigger" => Self::handle_set_trigger(params).await,
            "debug.program" => Self::handle_program(params).await,
            "debug.uart_loopback" => Self::handle_uart_loopback(params).await,
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

impl DebugHandler {
    async fn handle_create(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let protocol = params.get("protocol").and_then(|v| v.as_str()).unwrap_or("jtag");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.create",
            "protocol": protocol,
            "session_id": format!("dbg_{}", uuid::Uuid::new_v4().simple()),
        }))
    }

    async fn handle_connect(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let protocol = params.get("protocol").and_then(|v| v.as_str()).unwrap_or("jtag");
        let device = params.get("device").and_then(|v| v.as_str()).unwrap_or("");

        // Phase 20: project-facing debug artifacts under <root>/debug/.
        let debug_dir = conductor_sdk::workspace::ensure_artifact_dir("debug", None)?;
        let run_id = conductor_sdk::workspace::resolve_run_id();

        // Probe-rs / openocd availability check (no actual connection attempted here).
        let probe_rs = conductor_sdk::workspace::find_in_path("probe-rs");
        let openocd = conductor_sdk::workspace::find_in_path("openocd");
        let tool_present = probe_rs.is_some() || openocd.is_some();

        let session_id = format!("dbg_{}", uuid::Uuid::new_v4().simple());
        let session_path = debug_dir.join("debug_session.json");
        let session = serde_json::json!({
            "session_id": session_id,
            "run_id": run_id,
            "protocol": protocol,
            "device": device,
            "probe_rs_path": probe_rs.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "openocd_path": openocd.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "tool_present": tool_present,
            "connected": false,
            "note": "Connection attempt deferred to operator (no JTAG/SWD probe available in this environment). Session manifest emitted."
        });
        std::fs::write(&session_path, serde_json::to_string_pretty(&session).unwrap())
            .map_err(|e| format!("write debug_session.json failed: {e}"))?;

        Ok(serde_json::json!({
            "status": if tool_present { "ok" } else { "tool_unavailable" },
            "method": "debug.connect",
            "protocol": protocol,
            "device": device,
            "session_id": session_id,
            "tool_present": tool_present,
            "run_id": run_id,
            "artifact": session_path.to_string_lossy(),
            "debug_dir": debug_dir.to_string_lossy(),
        }))
    }

    async fn handle_disconnect(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let session_id = params.get("session_id").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.disconnect",
            "session_id": session_id,
        }))
    }

    async fn handle_reset(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("hardware");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.reset",
            "mode": mode,
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "debug.status",
        }))
    }

    async fn handle_set_breakpoint(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let bp_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("source");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.setBreakpoint",
            "type": bp_type,
            "bp_id": format!("bp_{}", uuid::Uuid::new_v4().simple()),
        }))
    }

    async fn handle_remove_breakpoint(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.removeBreakpoint",
        }))
    }

    async fn handle_run(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.run",
        }))
    }

    async fn handle_pause() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.pause",
        }))
    }

    async fn handle_step_over() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.stepOver",
        }))
    }

    async fn handle_step_into() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.stepInto",
        }))
    }

    async fn handle_read_memory(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("0x00000000");
        let size = params.get("size").and_then(|v| v.as_u64()).unwrap_or(4);
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.readMemory",
            "address": address,
            "size": size,
            "data": [],
        }))
    }

    async fn handle_write_memory(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let address = params.get("address").and_then(|v| v.as_str()).unwrap_or("0x00000000");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.writeMemory",
            "address": address,
        }))
    }

    async fn handle_start_capture(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let signals = params.get("signals").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
            .unwrap_or_default();
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.startCapture",
            "capture_id": format!("cap_{}", uuid::Uuid::new_v4().simple()),
            "signals": signals,
        }))
    }

    async fn handle_stop_capture() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.stopCapture",
        }))
    }

    async fn handle_read_signals(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.read_signals",
            "signals": {},
        }))
    }

    async fn handle_set_trigger(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.set_trigger",
        }))
    }

    async fn handle_program(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let method = params.get("method").and_then(|v| v.as_str()).unwrap_or("probe-rs");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "debug.program",
            "flash_method": method,
        }))
    }

    /// Generic UART loopback / send-and-receive test.
    ///
    /// Phase 21: keeps the handler app-agnostic. The caller specifies the serial
    /// device, baud rate and an arbitrary byte pattern to write. The handler
    /// opens the device, writes the pattern, optionally reads back, and writes
    /// a report under `<root>/debug/`.
    ///
    /// Params:
    /// - `device` (string): serial device path. Defaults to "/dev/ttyUSB1".
    /// - `baud` (integer): baud rate. Default 115200.
    /// - `pattern` (string|array<int>): bytes to send. Defaults to "ABCD".
    /// - `read_back` (bool): if true, also read up to `pattern.len()` bytes for
    ///   a true loopback test. Default false (one-shot send).
    /// - `read_timeout_ms` (integer): per-read timeout. Default 500.
    /// - `execute` (bool): default true. If false, only emits a plan/report.
    async fn handle_uart_loopback(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let device = params.get("device").and_then(|v| v.as_str()).unwrap_or("/dev/ttyUSB1").to_string();
        let baud = params.get("baud").and_then(|v| v.as_u64()).unwrap_or(115_200) as u32;
        let read_back = params.get("read_back").and_then(|v| v.as_bool()).unwrap_or(false);
        let read_timeout_ms = params.get("read_timeout_ms").and_then(|v| v.as_u64()).unwrap_or(500);
        let execute = params.get("execute").and_then(|v| v.as_bool()).unwrap_or(true);

        let pattern_bytes: Vec<u8> = match params.get("pattern") {
            Some(serde_json::Value::String(s)) => s.as_bytes().to_vec(),
            Some(serde_json::Value::Array(arr)) => arr.iter()
                .filter_map(|v| v.as_u64()).map(|n| (n & 0xff) as u8).collect(),
            _ => b"ABCD".to_vec(),
        };

        let debug_dir = conductor_sdk::workspace::ensure_artifact_dir("debug", None)?;
        let run_id = conductor_sdk::workspace::resolve_run_id();
        let report_path = debug_dir.join("uart_loopback.json");
        let log_path = debug_dir.join("uart_loopback.log");

        let device_present = std::path::Path::new(&device).exists();

        // Phase 26: align status names with the canonical Phase 25 vocabulary
        // (see ai persona "handler の status 値とアグリゲート status の対応" table).
        // Device absence and stty/open failures are environment-side issues,
        // not Hestia errors → all map to `tool_unavailable` (exit 0).
        let (executed, write_ok, bytes_received, received_hex, status_str, error_msg) = if !execute {
            (false, false, 0usize, String::new(), "skipped".to_string(), None)
        } else if !device_present {
            let _ = std::fs::write(&log_path, format!("[debug.uart_loopback] device '{device}' not present\n"));
            (false, false, 0usize, String::new(), "tool_unavailable".to_string(), Some(format!("device {device} not found")))
        } else {
            // Configure the device with stty (no extra dep).
            let stty_args = ["-F", &device, &baud.to_string(), "raw", "-echo", "-echoe", "-echok", "cs8", "-cstopb", "-parenb", "min", "0", "time", "0"];
            let stty_res = tokio::process::Command::new("stty").args(stty_args).output().await;
            if let Err(e) = stty_res {
                let _ = std::fs::write(&log_path, format!("[debug.uart_loopback] stty failed: {e}\n"));
                let none_msg: Option<String> = Some(format!("stty failed: {e}"));
                (false, false, 0usize, String::new(), "tool_unavailable".to_string(), none_msg)
            } else {
                use std::io::{Read, Write};
                use std::os::unix::fs::OpenOptionsExt;
                use std::time::{Duration, Instant};

                let open_result = std::fs::OpenOptions::new()
                    .read(true).write(true)
                    .custom_flags(libc::O_NOCTTY | libc::O_NONBLOCK)
                    .open(&device);

                match open_result {
                    Err(e) => {
                        let _ = std::fs::write(&log_path, format!("[debug.uart_loopback] open {device} failed: {e}\n"));
                        // Phase 26: open failure is env-side (permissions / udev / device gone)
                        (false, false, 0usize, String::new(), "tool_unavailable".to_string(), Some(format!("open failed: {e}")))
                    }
                    Ok(mut port) => {
                        let written = port.write_all(&pattern_bytes).is_ok();
                        let _ = port.flush();

                        let mut received: Vec<u8> = Vec::new();
                        if written && read_back {
                            let deadline = Instant::now() + Duration::from_millis(read_timeout_ms.saturating_mul(2).max(read_timeout_ms));
                            let want = pattern_bytes.len();
                            let mut buf = [0u8; 256];
                            while received.len() < want && Instant::now() < deadline {
                                match port.read(&mut buf) {
                                    Ok(0) => tokio::time::sleep(Duration::from_millis(20)).await,
                                    Ok(n) => received.extend_from_slice(&buf[..n]),
                                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                                                || e.raw_os_error() == Some(libc::EAGAIN) => {
                                        tokio::time::sleep(Duration::from_millis(20)).await;
                                    }
                                    Err(e) => {
                                        let _ = std::fs::write(&log_path, format!("[debug.uart_loopback] read failed: {e}\n"));
                                        break;
                                    }
                                }
                            }
                        }

                        let hex_dump = received.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                        let status_str = if !written {
                            "write_failed".to_string()
                        } else if read_back {
                            if received == pattern_bytes { "ok".to_string() }
                            else if received.is_empty() { "no_response".to_string() }
                            else { "mismatch".to_string() }
                        } else {
                            "sent".to_string()
                        };

                        let _ = std::fs::write(&log_path, format!(
                            "[debug.uart_loopback] device={device} baud={baud} wrote={} bytes read={} bytes\nsent: {:02x?}\nrecv: {:02x?}\n",
                            pattern_bytes.len(), received.len(), pattern_bytes.as_slice(), received.as_slice()
                        ));

                        (true, written, received.len(), hex_dump, status_str, None)
                    }
                }
            }
        };

        let report = serde_json::json!({
            "run_id": run_id,
            "method": "debug.uart_loopback",
            "device": device,
            "baud": baud,
            "pattern_len": pattern_bytes.len(),
            "pattern_hex": pattern_bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" "),
            "executed": executed,
            "write_ok": write_ok,
            "read_back": read_back,
            "bytes_received": bytes_received,
            "received_hex": received_hex,
            "log": log_path.to_string_lossy(),
            "status": status_str,
            "error": error_msg,
        });
        std::fs::write(&report_path, serde_json::to_string_pretty(&report).unwrap())
            .map_err(|e| format!("write uart_loopback.json failed: {e}"))?;

        Ok(serde_json::json!({
            "status": status_str,
            "method": "debug.uart_loopback",
            "device": device,
            "baud": baud,
            "pattern_len": pattern_bytes.len(),
            "executed": executed,
            "write_ok": write_ok,
            "bytes_received": bytes_received,
            "received_hex": received_hex,
            "artifact": report_path.to_string_lossy(),
            "log": log_path.to_string_lossy(),
            "run_id": run_id,
            "error": error_msg,
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": ["openocd", "probe-rs", "sigrok"],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }
}