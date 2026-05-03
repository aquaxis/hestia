//! RAG handler smoke tests.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_rag_conductor::handler::RagHandler;
use serde_json::json;

async fn invoke(method: &str, params: serde_json::Value) -> Response {
    let handler = RagHandler;
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
async fn search_returns_empty_chunks() {
    let result = unwrap_ok(invoke("rag.search", json!({"query": "uart"})).await);
    assert_eq!(result["status"], "ok");
    assert!(result["chunks"].is_array());
    assert_eq!(result["query"], "uart");
}

#[tokio::test]
async fn cleanup_returns_ok() {
    let result = unwrap_ok(invoke("rag.cleanup", json!({})).await);
    assert_eq!(result["status"], "ok");
}

#[tokio::test]
async fn search_design_returns_empty_results() {
    let result = unwrap_ok(invoke("rag.search_design.v1", json!({})).await);
    assert_eq!(result["status"], "ok");
    assert!(result["results"].is_array());
}

#[tokio::test]
async fn unknown_method_returns_error() {
    match invoke("rag.does_not_exist", json!({})).await {
        Response::Error(e) => assert_eq!(e.error.code, -32601),
        Response::Success(_) => panic!("expected Error for unknown method"),
    }
}
