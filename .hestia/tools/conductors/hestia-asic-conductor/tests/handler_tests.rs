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
