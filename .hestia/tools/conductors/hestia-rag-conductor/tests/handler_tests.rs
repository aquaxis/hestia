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

#[tokio::test]
async fn dispatch_ingest_v1_requires_sources() {
    let result = unwrap_ok(invoke("rag.dispatch_ingest.v1",
                                    json!({"sources": [], "spec": ""})).await);
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["phase"], "phase60b");
}

#[tokio::test]
async fn dispatch_ingest_v1_includes_auto_review_dispatched_field() {
    // Phase 80: dispatch path must include `auto_review_dispatched` boolean field.
    // PATH override prevents `hestia spawn-subagent` from actually launching.
    let prior_path = std::env::var("PATH").ok();
    std::env::set_var("HESTIA_DISABLE_AUTO_REVIEW", "1");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("HESTIA_STRICT_SUBAGENT", "0");
    std::env::set_var("PATH", "/nonexistent");
    let result = unwrap_ok(invoke("rag.dispatch_ingest.v1",
        json!({"sources": ["docs/spec.md"], "spec": "test"})).await);
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
    std::env::set_var("HESTIA_PEER_ALIVE_FORCE", "");
    std::env::set_var("HESTIA_PEER_SEND_NOOP", "1");
    std::env::set_var("HESTIA_STRICT_SUBAGENT", "0");
    let result = unwrap_ok(invoke("rag.design.v1",
        json!({"instruction": "ingest pdf docs and build vector index"})).await);
    std::env::remove_var("HESTIA_PEER_ALIVE_FORCE");
    std::env::remove_var("HESTIA_PEER_SEND_NOOP");
    std::env::remove_var("HESTIA_STRICT_SUBAGENT");
    assert_eq!(result["status"], "input_required");
    assert_eq!(result["designer_peer"], "rag-designer");
    assert_eq!(result["phase"], "phase58-fallback");
}
