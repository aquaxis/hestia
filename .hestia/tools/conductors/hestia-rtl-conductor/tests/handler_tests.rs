//! Unit tests for RTL handler — Phase 54 design.v1 delegation semantics.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_rtl_conductor::handler::RtlHandler;
use serde_json::json;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Phase 55b — invoke variant that holds ENV_LOCK across both the
/// HESTIA_PEER_ALIVE_FORCE override and the handler call.
async fn invoke_with_peers(method: &str, params: serde_json::Value, alive_peers: &str)
    -> serde_json::Value
{
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".hestia")).expect("mkdir .hestia");
    let prior_root = std::env::var("HESTIA_PROJECT_ROOT").ok();
    let prior_peers = std::env::var("HESTIA_PEER_ALIVE_FORCE").ok();
    let prior_noop = std::env::var("HESTIA_PEER_SEND_NOOP").ok();
    std::env::set_var("HESTIA_PROJECT_ROOT", tmp.path());
    std::env::set_var("HESTIA_PEER_ALIVE_FORCE", alive_peers);
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");

    let handler = RtlHandler;
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
    match prior_peers {
        Some(v) => std::env::set_var("HESTIA_PEER_ALIVE_FORCE", v),
        None => std::env::remove_var("HESTIA_PEER_ALIVE_FORCE"),
    }
    match prior_noop {
        Some(v) => std::env::set_var("HESTIA_PEER_SEND_NOOP", v),
        None => std::env::remove_var("HESTIA_PEER_SEND_NOOP"),
    }

    match response {
        Response::Success(s) => s.result,
        Response::Error(e) => panic!("expected Success, got Error: {:?}", e.error),
    }
}

#[tokio::test]
async fn design_v1_falls_back_to_input_required_when_designer_offline() {
    let result = invoke_with_peers(
        "rtl.design.v1",
        json!({"instruction": "design UART RX FSM with 16x oversampling"}),
        "",
    ).await;
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["method"], "rtl.design.v1");
    assert_eq!(result["designer_peer"], "rtl-designer");
    assert_eq!(result["designer_alive"], false);
    assert_eq!(result["phase"], "phase55b-fallback");
    assert_eq!(
        result["instruction"],
        "design UART RX FSM with 16x oversampling"
    );
}

#[tokio::test]
async fn design_v1_delegates_to_designer_when_alive() {
    let result = invoke_with_peers(
        "rtl.design.v1",
        json!({"instruction": "design 8b/10b encoder"}),
        "rtl-designer",
    ).await;
    assert_eq!(result["status"], "delegated");
    assert_eq!(result["method"], "rtl.design.v1");
    assert_eq!(result["designer_peer"], "rtl-designer");
    assert_eq!(result["designer_alive"], true);
    assert_eq!(result["phase"], "phase55c");
    assert_eq!(result["dispatched"], true);
    assert!(result["expected_artifacts"].is_array());
}

#[tokio::test]
async fn dispatch_coders_v1_requires_modules() {
    // Phase 60: empty modules list returns input_required.
    let result = invoke_with_peers(
        "rtl.dispatch_coders.v1",
        json!({"modules": [], "spec": ""}),
        "",
    ).await;
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["method"], "rtl.dispatch_coders.v1");
    assert_eq!(result["phase"], "phase60");
}

#[tokio::test]
async fn dispatch_coders_v1_includes_auto_review_dispatched_field() {
    // Phase 80: dispatch path must include `auto_review_dispatched` boolean field.
    // PATH override prevents `hestia spawn-subagent` from actually launching.
    let prior_review = std::env::var("HESTIA_DISABLE_AUTO_REVIEW").ok();
    let prior_path = std::env::var("PATH").ok();
    std::env::set_var("HESTIA_DISABLE_AUTO_REVIEW", "1");
    std::env::set_var("PATH", "/nonexistent");
    let result = invoke_with_peers(
        "rtl.dispatch_coders.v1",
        json!({"modules": ["uart_rx"], "spec": "test"}),
        "",
    ).await;
    match prior_review {
        Some(v) => std::env::set_var("HESTIA_DISABLE_AUTO_REVIEW", v),
        None => std::env::remove_var("HESTIA_DISABLE_AUTO_REVIEW"),
    }
    match prior_path {
        Some(v) => std::env::set_var("PATH", v),
        None => std::env::remove_var("PATH"),
    }
    assert!(result["auto_review_dispatched"].is_boolean(),
        "auto_review_dispatched field must be present");
    assert_eq!(result["auto_review_dispatched"], false,
        "with HESTIA_DISABLE_AUTO_REVIEW=1 auto_review must be skipped");
}

#[tokio::test]
async fn unknown_method_returns_error() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let handler = RtlHandler;
    let request = Request {
        kind: "prompt".to_string(),
        from: "test".to_string(),
        method: "rtl.bogus.v9".to_string(),
        params: json!({}),
        id: MessageId::new(),
        trace_id: None,
    };
    let response = handler.handle_request(request).await;
    match response {
        Response::Error(e) => assert_eq!(e.error.code, -32601),
        Response::Success(_) => panic!("expected Error for unknown method"),
    }
}
