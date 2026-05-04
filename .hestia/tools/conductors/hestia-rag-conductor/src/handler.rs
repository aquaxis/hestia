//! RAG Conductor メッセージハンドラ

use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;

/// RAG Conductor メッセージハンドラ
pub struct RagHandler;

#[async_trait::async_trait]
impl MessageHandler for RagHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            "rag.ingest" => Self::handle_ingest(params).await,
            "rag.design.v1" => Self::handle_design(params).await,
            "rag.dispatch_ingest.v1" => Self::handle_dispatch_ingest(params).await,
            "rag.search" => Self::handle_search(params).await,
            "rag.cleanup" => Self::handle_cleanup(params).await,
            "rag.status" => Self::handle_status().await,
            "rag.ingest_work.v1" => Self::handle_ingest_work(params).await,
            "rag.search_similar.v1" => Self::handle_search_similar(params).await,
            "rag.search_bugfix.v1" => Self::handle_search_bugfix(params).await,
            "rag.search_design.v1" => Self::handle_search_design(params).await,
            "system.health.v1" => Self::handle_health().await,
            _ => {
                return Response::Error(ErrorResultResponse {
                    error: ErrorResponse {
                        code: -32601,
                        message: format!("Method not found: {method}"),
                        data: None,
                    },
                    id,
                    trace_id,
                });
            }
        };

        match result {
            Ok(value) => Response::Success(SuccessResponse {
                result: value,
                id,
                trace_id,
            }),
            Err(msg) => Response::Error(ErrorResultResponse {
                error: ErrorResponse {
                    code: -32000,
                    message: msg,
                    data: None,
                },
                id,
                trace_id,
            }),
        }
    }
}

impl RagHandler {
    async fn handle_ingest(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let source_type = params.get("source_type").and_then(|v| v.as_str()).unwrap_or("pdf");
        let file_path = params.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
        let force = params.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.ingest",
            "source_type": source_type,
            "file_path": file_path,
            "force": force,
            "chunks_ingested": 0,
        }))
    }

    /// Phase 58 — `rag.design.v1`: ナレッジベース構造設計依頼を rag-designer に dispatch。
    async fn handle_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
        let designer_peer = "rag-designer";
        let designer_alive = conductor_sdk::workspace::agent_cli_peer_alive(designer_peer);
        let expected_artifacts = vec!["rag/index_schema.json", "rag/ingest_plan.json"];
        if designer_alive {
            let prompt = format!(
                "[rag.design.v1] {instruction}\nfs_write rag/index_schema.json + rag/ingest_plan.json."
            );
            let dispatched = conductor_sdk::workspace::agent_cli_send(designer_peer, &prompt).is_ok();
            Ok(serde_json::json!({
                "status": "delegated",
                "method": "rag.design.v1",
                "phase": "phase58",
                "designer_peer": designer_peer,
                "designer_alive": true,
                "dispatched": dispatched,
                "expected_artifacts": expected_artifacts,
                "instruction": instruction,
            }))
        } else {
            Ok(serde_json::json!({
                "status": "input_required",
                "method": "rag.design.v1",
                "phase": "phase58-fallback",
                "designer_peer": designer_peer,
                "designer_alive": false,
                "expected_artifacts": expected_artifacts,
                "fallback": "ai-conductor fs_write rag/index_schema.json + rag/ingest_plan.json",
                "instruction": instruction,
            }))
        }
    }

    /// Phase 60b — `rag.dispatch_ingest.v1`: rag-ingest-{source} 動的並列起動。
    async fn handle_dispatch_ingest(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let sources: Vec<String> = params.get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let spec = params.get("spec").and_then(|v| v.as_str()).unwrap_or("");
        if sources.is_empty() {
            return Ok(serde_json::json!({
                "status": "input_required",
                "method": "rag.dispatch_ingest.v1",
                "phase": "phase60b",
                "note": "sources array required (each entry becomes a `rag-ingest-{source}` peer)",
            }));
        }
        let mut spawned: Vec<String> = Vec::new();
        let mut dispatched_all = true;
        for s in &sources {
            // sanitize peer name (replace path separators / spaces)
            let safe = s.replace(['/', ' ', '\\'], "_");
            let peer = format!("rag-ingest-{safe}");
            let r = std::process::Command::new("hestia")
                .args(["spawn-subagent", "--persona", "rag-ingest", "--name", &peer])
                .output();
            match r {
                Ok(o) if o.status.success() => spawned.push(peer.clone()),
                _ => dispatched_all = false,
            }
            let prompt = format!("[rag-ingest-{safe}] ingest source `{s}`: {spec}");
            if conductor_sdk::workspace::agent_cli_send(&peer, &prompt).is_err() {
                dispatched_all = false;
            }
        }
        // Phase 80: dispatch 完了後に ai-reviewer auto-spawn
        let auto_review_dispatched = conductor_sdk::workspace::auto_review_after_dispatch(
            "rag", "rag.dispatch_ingest.v1", spawned.len(),
        );

        Ok(serde_json::json!({
            "status": if dispatched_all { "delegated" } else { "partial" },
            "method": "rag.dispatch_ingest.v1",
            "phase": "phase60b",
            "spawned": spawned,
            "dispatched_all": dispatched_all,
            "sources_requested": sources.len(),
            "auto_review_dispatched": auto_review_dispatched,
        }))
    }

    async fn handle_search(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let _top_k = params.get("top_k").and_then(|v| v.as_u64()).unwrap_or(10);
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.search",
            "query": query,
            "chunks": [],
            "citations": [],
            "embedding_time_ms": 0,
            "retrieval_time_ms": 0,
        }))
    }

    async fn handle_cleanup(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.cleanup",
            "cleaned": 0,
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "online",
            "method": "rag.status",
        }))
    }

    async fn handle_ingest_work(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let category = params.get("category").and_then(|v| v.as_str()).unwrap_or("design_case");
        let conductor = params.get("conductor").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.ingest_work.v1",
            "category": category,
            "conductor": conductor,
        }))
    }

    async fn handle_search_similar(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.search_similar.v1",
            "results": [],
        }))
    }

    async fn handle_search_bugfix(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.search_bugfix.v1",
            "results": [],
        }))
    }

    async fn handle_search_design(_params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.search_design.v1",
            "results": [],
        }))
    }

    async fn handle_health() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": [],
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
        }))
    }
}