//! AI orchestrator handler smoke tests.

use conductor_sdk::config::HestiaClientConfig;
use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_ai_conductor::handler::AiHandler;
use serde_json::json;

async fn invoke(method: &str, params: serde_json::Value) -> Response {
    let handler = AiHandler::new(HestiaClientConfig::default());
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
async fn readiness_returns_ready_flag() {
    let result = unwrap_ok(invoke("system.readiness", json!({})).await);
    assert_eq!(result["ready"], true,
        "system.readiness should report ready=true on a healthy ai conductor");
}

#[tokio::test]
async fn shutdown_returns_ok() {
    let result = unwrap_ok(invoke("system.shutdown", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["method"], "system.shutdown");
}

#[tokio::test]
async fn unknown_method_returns_error() {
    match invoke("ai.does_not_exist", json!({})).await {
        Response::Error(e) => assert_eq!(e.error.code, -32601),
        Response::Success(_) => panic!("expected Error for unknown method"),
    }
}
