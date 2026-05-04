//! Unit tests for Debug handler — Phase 26 status normalization semantics.
//!
//! Verifies that env-side failures (no device, no permissions, etc.) all
//! surface as `tool_unavailable` rather than diverging into per-failure-mode
//! status strings, so the ai persona's status-vocabulary table works
//! uniformly across handlers.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_debug_conductor::handler::DebugHandler;
use serde_json::json;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

async fn invoke(method: &str, params: serde_json::Value) -> serde_json::Value {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".hestia")).expect("mkdir .hestia");
    let prior_root = std::env::var("HESTIA_PROJECT_ROOT").ok();
    std::env::set_var("HESTIA_PROJECT_ROOT", tmp.path());

    let handler = DebugHandler;
    let request = Request {
        kind: "prompt".to_string(),
        from: "test".to_string(),
        method: method.to_string(),
        params,
        id: MessageId::new(),
        trace_id: None,
    };
    let response = handler.handle_request(request).await;

    match prior_root {
        Some(v) => std::env::set_var("HESTIA_PROJECT_ROOT", v),
        None => std::env::remove_var("HESTIA_PROJECT_ROOT"),
    }

    match response {
        Response::Success(s) => s.result,
        Response::Error(e) => panic!("expected Success, got Error: {:?}", e.error),
    }
}

#[tokio::test]
async fn uart_loopback_execute_false_returns_skipped() {
    let result = invoke("debug.uart_loopback", json!({"execute": false})).await;
    assert_eq!(result["status"], "skipped",
        "execute=false should yield `skipped` (dry-run), got {result:?}");
    assert_eq!(result["executed"], false);
    assert_eq!(result["write_ok"], false);
}

#[tokio::test]
async fn uart_loopback_missing_device_returns_tool_unavailable() {
    // Phase 26: any env-side failure (device gone, permissions, stty broken)
    // must surface as `tool_unavailable`, never as device_unavailable / etc.
    let result = invoke("debug.uart_loopback", json!({
        "execute": true,
        "device": "/dev/definitely-not-a-real-tty-xyzzy",
    })).await;
    assert_eq!(result["status"], "tool_unavailable",
        "missing device should normalize to `tool_unavailable`, got {result:?}");
    assert_eq!(result["executed"], false);
    assert!(result["error"].is_string(),
        "error message should be present for diagnostic");
}

#[tokio::test]
async fn connect_returns_tool_unavailable_when_no_probe() {
    // probe-rs / openocd typically not installed in test envs.
    let result = invoke("debug.connect", json!({})).await;
    // Either "ok" (if probe present) or "tool_unavailable" (typical CI).
    let status = result["status"].as_str().unwrap_or("");
    assert!(status == "ok" || status == "tool_unavailable",
        "debug.connect should return `ok` or `tool_unavailable`, got {status:?}");
    assert_eq!(result["method"], "debug.connect");
}

#[tokio::test]
async fn dispatch_sessions_v1_requires_targets() {
    let result = invoke("debug.dispatch_sessions.v1",
                          json!({"targets": [], "spec": ""})).await;
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["phase"], "phase65");
}

#[tokio::test]
async fn dispatch_sessions_v1_includes_auto_review_dispatched_field() {
    // Phase 80: dispatch path must include `auto_review_dispatched` boolean field.
    // PATH override prevents `hestia spawn-subagent` from actually launching.
    let prior_path = std::env::var("PATH").ok();
    std::env::set_var("HESTIA_DISABLE_AUTO_REVIEW", "1");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("HESTIA_STRICT_SUBAGENT", "0");
    std::env::set_var("PATH", "/nonexistent");
    let result = invoke("debug.dispatch_sessions.v1",
        json!({"targets": ["jtag"], "spec": "test"})).await;
    std::env::remove_var("HESTIA_DISABLE_AUTO_REVIEW");
    std::env::remove_var("HESTIA_PEER_SEND_NOOP");
    std::env::remove_var("HESTIA_STRICT_SUBAGENT");
    match prior_path {
        Some(v) => std::env::set_var("PATH", v),
        None => std::env::remove_var("PATH"),
    }
    assert!(result["auto_review_dispatched"].is_boolean(),
        "auto_review_dispatched field must be present");
    assert_eq!(result["auto_review_dispatched"], false);
}

#[tokio::test]
async fn design_v1_falls_back_when_designer_offline() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    std::env::set_var("HESTIA_PEER_ALIVE_FORCE", "");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("HESTIA_STRICT_SUBAGENT", "0");
    let handler = DebugHandler;
    let request = Request {
        kind: "prompt".to_string(),
        from: "test".to_string(),
        method: "debug.design.v1".to_string(),
        params: json!({"instruction": "JTAG SWD configuration for STM32"}),
        id: MessageId::new(),
        trace_id: None,
    };
    let response = handler.handle_request(request).await;
    std::env::remove_var("HESTIA_PEER_ALIVE_FORCE");
    std::env::remove_var("HESTIA_PEER_SEND_NOOP");
    std::env::remove_var("HESTIA_STRICT_SUBAGENT");
    let result = match response {
        Response::Success(s) => s.result,
        Response::Error(e) => panic!("expected Success, got: {:?}", e.error),
    };
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["designer_peer"], "debug-designer");
    assert_eq!(result["phase"], "phase58-fallback");
}
