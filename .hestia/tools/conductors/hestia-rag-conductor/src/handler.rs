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

    async fn handle_search(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let query = params.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let top_k = params.get("top_k").and_then(|v| v.as_u64()).unwrap_or(10);
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

    async fn handle_cleanup(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.cleanup",
            "cleaned": 0,
        }))
    }

    async fn handle_status() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.status",
            "index_size": 0,
            "total_documents": 0,
            "total_chunks": 0,
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

    async fn handle_search_similar(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.search_similar.v1",
            "results": [],
        }))
    }

    async fn handle_search_bugfix(params: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "rag.search_bugfix.v1",
            "results": [],
        }))
    }

    async fn handle_search_design(params: serde_json::Value) -> Result<serde_json::Value, String> {
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