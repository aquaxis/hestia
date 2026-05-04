//! PCB handler smoke tests.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_pcb_conductor::handler::PcbHandler;
use serde_json::json;

async fn invoke(method: &str, params: serde_json::Value) -> Response {
    let handler = PcbHandler;
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
async fn run_drc_returns_ok() {
    let result = unwrap_ok(invoke("pcb.run_drc", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["violations"], 0);
}

#[tokio::test]
async fn run_erc_returns_ok() {
    let result = unwrap_ok(invoke("pcb.run_erc", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["violations"], 0);
}

#[tokio::test]
async fn unknown_method_returns_error() {
    match invoke("pcb.does_not_exist", json!({})).await {
        Response::Error(e) => assert_eq!(e.error.code, -32601),
        Response::Success(_) => panic!("expected Error for unknown method"),
    }
}

#[tokio::test]
async fn dispatch_phases_v1_requires_phases() {
    let result = unwrap_ok(invoke("pcb.dispatch_phases.v1",
                                    json!({"phases": [], "spec": ""})).await);
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["phase"], "phase65");
}

#[tokio::test]
async fn dispatch_phases_v1_includes_auto_review_dispatched_field() {
    // Phase 80: dispatch path must include `auto_review_dispatched` boolean field.
    // PATH override prevents `hestia spawn-subagent` from actually launching.
    let prior_path = std::env::var("PATH").ok();
    std::env::set_var("HESTIA_DISABLE_AUTO_REVIEW", "1");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("PATH", "/nonexistent");
    let result = unwrap_ok(invoke("pcb.dispatch_phases.v1",
        json!({"phases": ["schematic"], "spec": "test"})).await);
    std::env::remove_var("HESTIA_DISABLE_AUTO_REVIEW");
    std::env::remove_var("HESTIA_PEER_SEND_NOOP");
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
    std::env::set_var("HESTIA_PEER_ALIVE_FORCE", "");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    let result = unwrap_ok(invoke("pcb.design.v1",
        json!({"instruction": "4-layer board with USB-C and JTAG"})).await);
    std::env::remove_var("HESTIA_PEER_ALIVE_FORCE");
    std::env::remove_var("HESTIA_PEER_SEND_NOOP");
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["designer_peer"], "pcb-designer");
    assert_eq!(result["phase"], "phase58-fallback");
}
