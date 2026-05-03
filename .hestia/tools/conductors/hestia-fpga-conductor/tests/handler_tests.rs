//! Unit tests for FPGA handler — Phase 25/26 input_required gating semantics.
//!
//! These tests exercise the public `MessageHandler::handle_request` interface
//! and verify the status taxonomy established in Phase 25 (`input_required`
//! distinct from `skipped` / `tool_unavailable` / `build_failed`) without
//! requiring Vivado on the test host.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_fpga_conductor::handler::FpgaHandler;
use serde_json::json;
use std::sync::Mutex;

// Serialize env-var mutation across tests — HESTIA_PROJECT_ROOT is process-global.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Run the handler in a project-root temp dir so artifact writes don't
/// pollute the workspace and don't see our prior-run files.
async fn invoke(method: &str, params: serde_json::Value) -> serde_json::Value {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let tmp = tempfile::tempdir().expect("tempdir");
    // resolve_project_root walks up looking for .hestia/, so create one in tmp.
    std::fs::create_dir_all(tmp.path().join(".hestia")).expect("mkdir .hestia");
    let prior_root = std::env::var("HESTIA_PROJECT_ROOT").ok();
    std::env::set_var("HESTIA_PROJECT_ROOT", tmp.path());

    let handler = FpgaHandler;
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
async fn build_v1_start_no_inputs_no_execute_returns_skipped() {
    let result = invoke("fpga.build.v1.start", json!({"target": "artix7"})).await;
    assert_eq!(result["status"], "skipped",
        "no inputs and no --execute should yield `skipped`, got {result:?}");
    assert_eq!(result["execute_requested"], false);
    assert_eq!(result["inputs_complete"], false);
    assert_eq!(result["rtl_sources_count"], 0);
    assert_eq!(result["constraints_present"], false);
    assert_eq!(result["part_resolved"], false);
    assert_eq!(result["executed"], false);
}

#[tokio::test]
async fn build_v1_start_no_inputs_with_execute_returns_input_required() {
    // Phase 25 contract: --execute + missing inputs → input_required (not build_failed).
    let result = invoke("fpga.build.v1.start", json!({"target": "artix7", "execute": true})).await;
    assert_eq!(result["status"], "input_required",
        "execute=true with no inputs should yield `input_required`, got {result:?}");
    assert_eq!(result["execute_requested"], true);
    assert_eq!(result["executed"], false,
        "Vivado must NOT be invoked when inputs are incomplete");
    assert_eq!(result["inputs_complete"], false);
}

#[tokio::test]
async fn program_no_bitstream_no_execute_returns_skipped() {
    let result = invoke("fpga.program", json!({})).await;
    assert_eq!(result["status"], "skipped",
        "no bitstream and no --execute should yield `skipped`, got {result:?}");
    assert_eq!(result["bitstream_present"], false);
    assert_eq!(result["inputs_complete"], false);
    assert_eq!(result["executed"], false);
}

#[tokio::test]
async fn program_no_bitstream_with_execute_returns_input_required() {
    // Phase 26 contract: same gating pattern as fpga.build.
    let result = invoke("fpga.program", json!({"execute": true})).await;
    assert_eq!(result["status"], "input_required",
        "execute=true with no bitstream should yield `input_required`, got {result:?}");
    assert_eq!(result["execute_requested"], true);
    assert_eq!(result["executed"], false);
    assert_eq!(result["bitstream_present"], false);
}

#[tokio::test]
async fn build_v1_start_diagnostic_fields_present() {
    // Phase 25 added diagnostic fields — ensure they're always populated so
    // callers can introspect *why* a build was gated.
    let result = invoke("fpga.build.v1.start", json!({"target": "artix7"})).await;
    for field in ["inputs_complete", "rtl_sources_count", "constraints_present", "part_resolved"] {
        assert!(result.get(field).is_some(),
            "diagnostic field `{field}` missing from response: {result:?}");
    }
}
