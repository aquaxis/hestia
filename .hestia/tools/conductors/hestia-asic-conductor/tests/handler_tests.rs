//! ASIC handler smoke tests — exercise the public dispatch surface.
//!
//! ASIC handlers don't currently use Phase 25/26 input_required gating, so
//! these tests just verify each method dispatches to a non-error response.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_asic_conductor::handler::AsicHandler;
use serde_json::json;

async fn invoke(method: &str, params: serde_json::Value) -> Response {
    let handler = AsicHandler;
    let request = Request {
        kind: "prompt".to_string(),
        from: "test".to_string(),
        method: method.to_string(),
        params,
        id: MessageId::new(),
        trace_id: None,
    };
    handler.handle_request(request).await
}

fn unwrap_ok(response: Response) -> serde_json::Value {
    match response {
        Response::Success(s) => s.result,
        Response::Error(e) => panic!("expected Success, got Error: {:?}", e.error),
    }
}

#[tokio::test]
async fn synthesize_returns_ok() {
    let result = unwrap_ok(invoke("asic.synthesize", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["method"], "asic.synthesize");
}

#[tokio::test]
async fn drc_returns_ok() {
    let result = unwrap_ok(invoke("asic.drc", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["violations"], 0);
}

#[tokio::test]
async fn pdk_list_returns_pdks() {
    let result = unwrap_ok(invoke("asic.pdk.list", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert!(result["pdks"].is_array());
}

#[tokio::test]
async fn unknown_method_returns_error() {
    match invoke("asic.does_not_exist", json!({})).await {
        Response::Error(e) => assert_eq!(e.error.code, -32601),
        Response::Success(_) => panic!("expected Error for unknown method"),
    }
}

#[tokio::test]
async fn dispatch_steps_v1_requires_steps() {
    let result = unwrap_ok(invoke("asic.dispatch_steps.v1",
                                    json!({"steps": [], "spec": ""})).await);
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["phase"], "phase65");
}

#[tokio::test]
async fn dispatch_steps_v1_includes_auto_review_dispatched_field() {
    // Phase 80: dispatch path must include `auto_review_dispatched` boolean field.
    // PATH override prevents `hestia spawn-subagent` from actually launching.
    let prior_path = std::env::var("PATH").ok();
    std::env::set_var("HESTIA_DISABLE_AUTO_REVIEW", "1");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("HESTIA_STRICT_SUBAGENT", "0");
    std::env::set_var("PATH", "/nonexistent");
    let result = unwrap_ok(invoke("asic.dispatch_steps.v1",
        json!({"steps": ["synthesize"], "spec": "test"})).await);
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
    // Phase 58: asic.design.v1 falls back to input_required when asic-designer offline.
    std::env::set_var("HESTIA_PEER_ALIVE_FORCE", "");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("HESTIA_STRICT_SUBAGENT", "0");
    let result = unwrap_ok(invoke("asic.design.v1",
        json!({"instruction": "design 8-bit MCU on sky130"})).await);
    std::env::remove_var("HESTIA_PEER_ALIVE_FORCE");
    std::env::remove_var("HESTIA_PEER_SEND_NOOP");
    std::env::remove_var("HESTIA_STRICT_SUBAGENT");
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["designer_peer"], "asic-designer");
    assert_eq!(result["phase"], "phase58-fallback");
}
