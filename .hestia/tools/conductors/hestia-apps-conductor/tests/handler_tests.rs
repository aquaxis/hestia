//! Apps handler smoke tests.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_apps_conductor::handler::AppsHandler;
use serde_json::json;

async fn invoke(method: &str, params: serde_json::Value) -> Response {
    let handler = AppsHandler;
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
async fn build_returns_ok() {
    let result = unwrap_ok(invoke("apps.build.v1", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["method"], "apps.build.v1");
}

#[tokio::test]
async fn size_returns_zeroed_report() {
    let result = unwrap_ok(invoke("apps.size.v1", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert_eq!(result["text"], 0);
    assert_eq!(result["data"], 0);
    assert_eq!(result["bss"], 0);
}

#[tokio::test]
async fn unknown_method_returns_error() {
    match invoke("apps.does_not_exist", json!({})).await {
        Response::Error(e) => assert_eq!(e.error.code, -32601),
        Response::Success(_) => panic!("expected Error for unknown method"),
    }
}
