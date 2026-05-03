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
