//! Unit tests for HAL handler — Phase 21 template resolution semantics.
//!
//! Verifies the documented resolution order:
//!   params.sources > <root>/hal/register_map.json > template > empty.

use conductor_sdk::message::{MessageId, Request, Response};
use conductor_sdk::server::MessageHandler;
use hestia_hal_conductor::handler::HalHandler;
use serde_json::json;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

async fn invoke_in(tmp: &std::path::Path, method: &str, params: serde_json::Value) -> serde_json::Value {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    std::fs::create_dir_all(tmp.join(".hestia")).expect("mkdir .hestia");
    let prior_root = std::env::var("HESTIA_PROJECT_ROOT").ok();
    std::env::set_var("HESTIA_PROJECT_ROOT", tmp);

    let handler = HalHandler;
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
async fn parse_no_inputs_returns_skipped_with_empty_source_kind() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let result = invoke_in(tmp.path(), "hal.parse.v1", json!({})).await;
    assert_eq!(result["status"], "skipped",
        "no source should yield `skipped`, got {result:?}");
    assert_eq!(result["source_kind"], "empty");
    assert_eq!(result["registers_parsed"], 0);
}

#[tokio::test]
async fn parse_resolves_project_existing_when_root_file_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Place an existing register_map.json at <root>/hal/.
    let hal_dir = tmp.path().join("hal");
    std::fs::create_dir_all(&hal_dir).expect("mkdir hal");
    let payload = json!({
        "registers": [
            {"name": "CTRL", "offset": "0x00"},
            {"name": "STAT", "offset": "0x04"},
        ]
    });
    std::fs::write(hal_dir.join("register_map.json"),
                   serde_json::to_string_pretty(&payload).unwrap())
        .expect("seed register_map");

    let result = invoke_in(tmp.path(), "hal.parse.v1", json!({})).await;
    assert_eq!(result["status"], "ok");
    assert_eq!(result["source_kind"], "project_existing");
    assert_eq!(result["registers_parsed"], 2);
}

#[tokio::test]
async fn parse_resolves_project_template_when_only_template_present() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Place a template at <root>/.hestia/hal/templates/.
    let tpl_dir = tmp.path().join(".hestia/hal/templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir tpl");
    let payload = json!({"registers": [{"name": "TEMPLATE_REG"}]});
    std::fs::write(tpl_dir.join("register_map.json"),
                   serde_json::to_string_pretty(&payload).unwrap())
        .expect("seed template");

    let result = invoke_in(tmp.path(), "hal.parse.v1", json!({})).await;
    assert_eq!(result["status"], "ok");
    assert_eq!(result["source_kind"], "project_template");
    assert_eq!(result["registers_parsed"], 1);
}

#[tokio::test]
async fn parse_prefers_existing_over_template() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Both exist — existing must win per Phase 21 resolution order.
    let hal_dir = tmp.path().join("hal");
    std::fs::create_dir_all(&hal_dir).expect("mkdir hal");
    std::fs::write(hal_dir.join("register_map.json"),
                   r#"{"registers":[{"name":"FROM_EXISTING"}]}"#)
        .expect("seed existing");

    let tpl_dir = tmp.path().join(".hestia/hal/templates");
    std::fs::create_dir_all(&tpl_dir).expect("mkdir tpl");
    std::fs::write(tpl_dir.join("register_map.json"),
                   r#"{"registers":[{"name":"FROM_TPL_A"},{"name":"FROM_TPL_B"}]}"#)
        .expect("seed template");

    let result = invoke_in(tmp.path(), "hal.parse.v1", json!({})).await;
    assert_eq!(result["source_kind"], "project_existing",
        "existing file must override template per Phase 21 resolution order");
    assert_eq!(result["registers_parsed"], 1);
}
