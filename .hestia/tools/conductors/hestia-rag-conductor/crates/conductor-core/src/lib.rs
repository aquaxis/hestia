//! rag-conductor-core -- Main daemon entry and message handler for the RAG conductor

use conductor_sdk::error::error_code;
use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use rag_ingest::{IngestPipeline, IngestResult, PdfPipeline, WebPipeline};
use thiserror::Error;

/// RAG conductor errors.
#[derive(Debug, Error)]
pub enum RagConductorError {
    #[error("ingest failed: {0}")]
    IngestFailed(String),
    #[error("search failed: {0}")]
    SearchFailed(String),
    #[error("cleanup failed: {0}")]
    CleanupFailed(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

/// RAG conductor daemon.
pub struct RagConductor {
    pdf_pipeline: PdfPipeline,
    web_pipeline: WebPipeline,
}

impl RagConductor {
    /// Create a new RAG conductor with default pipelines.
    pub fn new() -> Self {
        Self {
            pdf_pipeline: PdfPipeline::new(),
            web_pipeline: WebPipeline::new(),
        }
    }

    /// Start the RAG conductor daemon.
    pub async fn start(&self) -> Result<(), RagConductorError> {
        tracing::info!("RagConductor: starting");
        Ok(())
    }

    /// Handle an incoming message by dispatching to the appropriate handler.
    pub async fn handle_message(&self, request: &Request) -> Response {
        let result = match request.method.as_str() {
            "rag.ingest" => self.handle_ingest(&request.params).await,
            "rag.search" => self.handle_search(&request.params).await,
            "rag.cleanup" => self.handle_cleanup(&request.params).await,
            "rag.status" => self.handle_status().await,
            _ => Err(RagConductorError::InvalidRequest(format!(
                "Unknown method: {}",
                request.method
            ))),
        };

        match result {
            Ok(value) => Response::Success(SuccessResponse {
                result: value,
                id: request.id.clone(),
                trace_id: request.trace_id.clone(),
            }),
            Err(err) => {
                let err_resp = conductor_sdk::error::ErrorResponse {
                    code: error_code::RAG_START,
                    message: err.to_string(),
                    data: None,
                };
                Response::Error(ErrorResultResponse {
                    error: err_resp,
                    id: request.id.clone(),
                    trace_id: request.trace_id.clone(),
                })
            }
        }
    }

    /// Handle `rag.ingest` -- ingest a document (PDF or web URL).
    async fn handle_ingest(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, RagConductorError> {
        let source = params
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RagConductorError::InvalidRequest("missing 'source' field".to_string()))?;

        let source_type = params
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("pdf");

        let result: IngestResult = match source_type {
            "web" => self
                .web_pipeline
                .ingest(source)
                .await
                .map_err(|e| RagConductorError::IngestFailed(e.to_string()))?,
            _ => self
                .pdf_pipeline
                .ingest(source)
                .await
                .map_err(|e| RagConductorError::IngestFailed(e.to_string()))?,
        };

        Ok(serde_json::to_value(result).unwrap_or_else(|_| serde_json::json!({})))
    }

    /// Handle `rag.search` -- search the vector store.
    async fn handle_search(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, RagConductorError> {
        let _query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RagConductorError::InvalidRequest("missing 'query' field".to_string()))?;

        tracing::info!("RagConductor: rag.search");
        // Stub: return empty results
        Ok(serde_json::json!({"results": []}))
    }

    /// Handle `rag.cleanup` -- clean up stale entries.
    async fn handle_cleanup(
        &self,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, RagConductorError> {
        tracing::info!("RagConductor: rag.cleanup");
        Ok(serde_json::json!({"cleaned": 0}))
    }

    /// Handle `rag.status` -- return conductor status.
    async fn handle_status(&self) -> Result<serde_json::Value, RagConductorError> {
        Ok(serde_json::json!({
            "status": "online",
            "pipelines": ["pdf", "web"]
        }))
    }
}

impl Default for RagConductor {
    fn default() -> Self {
        Self::new()
    }
}