//! rag-conductor-core -- Main daemon entry and message handler for the RAG conductor

use conductor_sdk::error::error_code;
use conductor_sdk::message::{ErrorResultResponse, Request, Response, SuccessResponse};
use rag_ingest::{get_db, IngestPipeline, IngestResult, PdfPipeline, WebPipeline};
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
    db: sled::Db,
}

impl RagConductor {
    /// Create a new RAG conductor with default pipelines.
    pub fn new() -> Self {
        let db = get_db();
        Self {
            pdf_pipeline: PdfPipeline::new(),
            web_pipeline: WebPipeline::new(),
            db,
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
            .ok_or_else(|| {
                RagConductorError::InvalidRequest("missing 'source' field".to_string())
            })?;

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

    /// Handle `rag.search` -- search the vector store using token-based matching.
    async fn handle_search(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, RagConductorError> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                RagConductorError::InvalidRequest("missing 'query' field".to_string())
            })?;

        let limit = params
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        tracing::info!("RagConductor: rag.search (query='{}')", query);

        let index_tree = self
            .db
            .open_tree("search_index")
            .map_err(|e| RagConductorError::SearchFailed(format!("open index tree: {}", e)))?;
        let chunks_tree = self
            .db
            .open_tree("chunks")
            .map_err(|e| RagConductorError::SearchFailed(format!("open chunks tree: {}", e)))?;

        // Tokenize the query and look up inverted-index entries.
        let query_tokens: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let mut matches: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for token in &query_tokens {
            let prefix = format!("idx:{}:", token);
            for item in index_tree.scan_prefix(prefix.as_bytes()) {
                let (_, value) = item.map_err(|e| {
                    RagConductorError::SearchFailed(format!("scan index: {}", e))
                })?;
                let chunk_key = String::from_utf8_lossy(&value).to_string();
                *matches.entry(chunk_key).or_insert(0) += 1;
            }
        }

        // Rank by number of matching tokens (descending).
        let mut ranked: Vec<_> = matches.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        ranked.truncate(limit);

        // Fetch matched chunk content.
        let mut results = Vec::new();
        for (chunk_key, score) in ranked {
            if let Some(iv) = chunks_tree.get(chunk_key.as_bytes()).map_err(|e| {
                RagConductorError::SearchFailed(format!("read chunk: {}", e))
            })? {
                let chunk_data: serde_json::Value =
                    serde_json::from_slice(&iv).unwrap_or_default();
                results.push(serde_json::json!({
                    "chunk_key": chunk_key,
                    "score": score,
                    "source_id": chunk_data.get("source_id").and_then(|v| v.as_str()).unwrap_or(""),
                    "text": chunk_data.get("text").and_then(|v| v.as_str()).unwrap_or(""),
                    "index": chunk_data.get("index").and_then(|v| v.as_u64()).unwrap_or(0),
                }));
            }
        }

        Ok(serde_json::json!({"results": results}))
    }

    /// Handle `rag.cleanup` -- clean up stale or orphaned entries from the vector store.
    async fn handle_cleanup(
        &self,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, RagConductorError> {
        tracing::info!("RagConductor: rag.cleanup");

        let max_age_secs = params
            .get("max_age_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(86400); // default: 1 day

        let sources_tree = self
            .db
            .open_tree("sources")
            .map_err(|e| RagConductorError::CleanupFailed(format!("open sources tree: {}", e)))?;
        let chunks_tree = self
            .db
            .open_tree("chunks")
            .map_err(|e| RagConductorError::CleanupFailed(format!("open chunks tree: {}", e)))?;
        let embeddings_tree = self
            .db
            .open_tree("embeddings")
            .map_err(|e| {
                RagConductorError::CleanupFailed(format!("open embeddings tree: {}", e))
            })?;
        let index_tree = self
            .db
            .open_tree("search_index")
            .map_err(|e| RagConductorError::CleanupFailed(format!("open index tree: {}", e)))?;

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let cutoff = now_secs.saturating_sub(max_age_secs);

        let mut cleaned = 0usize;

        // If a specific source_id is requested, clean that source only.
        if let Some(source_id) = params.get("source_id").and_then(|v| v.as_str()) {
            cleaned += self
                .remove_source(
                    source_id,
                    &sources_tree,
                    &chunks_tree,
                    &embeddings_tree,
                    &index_tree,
                )
                .await?;
        } else {
            // Remove sources older than max_age_secs.
            let stale_source_ids: Vec<String> = sources_tree
                .iter()
                .filter_map(|item| {
                    let (key, value) = item.ok()?;
                    let meta: serde_json::Value = serde_json::from_slice(&value).ok()?;
                    let indexed_at = meta.get("indexed_at").and_then(|v| v.as_u64())?;
                    if indexed_at < cutoff {
                        Some(String::from_utf8_lossy(&key).to_string())
                    } else {
                        None
                    }
                })
                .collect();

            for source_id in &stale_source_ids {
                cleaned += self
                    .remove_source(
                        source_id,
                        &sources_tree,
                        &chunks_tree,
                        &embeddings_tree,
                        &index_tree,
                    )
                    .await?;
            }

            // Also remove orphaned chunks (whose source_id is missing from sources_tree).
            let orphaned: Vec<String> = chunks_tree
                .iter()
                .filter_map(|item| {
                    let (_, value) = item.ok()?;
                    let chunk_data: serde_json::Value = serde_json::from_slice(&value).ok()?;
                    let sid = chunk_data.get("source_id").and_then(|v| v.as_str())?;
                    // Check if the source entry still exists.
                    if sources_tree.get(sid.as_bytes()).ok()?.is_none() {
                        Some(sid.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            let mut deduped_orphans: std::collections::HashSet<String> =
                std::collections::HashSet::from_iter(orphaned);
            // Don't double-count sources we already cleaned above.
            for sid in &stale_source_ids {
                deduped_orphans.remove(sid);
            }

            for source_id in &deduped_orphans {
                cleaned += self
                    .remove_source(
                        source_id,
                        &sources_tree,
                        &chunks_tree,
                        &embeddings_tree,
                        &index_tree,
                    )
                    .await?;
            }
        }

        self.db
            .flush()
            .map_err(|e| RagConductorError::CleanupFailed(format!("flush db: {}", e)))?;

        Ok(serde_json::json!({"cleaned": cleaned}))
    }

    /// Remove all data associated with a source_id from every sled tree.
    async fn remove_source(
        &self,
        source_id: &str,
        sources_tree: &sled::Tree,
        chunks_tree: &sled::Tree,
        embeddings_tree: &sled::Tree,
        index_tree: &sled::Tree,
    ) -> Result<usize, RagConductorError> {
        let prefix = format!("{}_", source_id);
        let mut removed = 0usize;

        // Remove chunks.
        for item in chunks_tree.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item.map_err(|e| {
                RagConductorError::CleanupFailed(format!("scan chunks: {}", e))
            })?;
            chunks_tree.remove(&key).map_err(|e| {
                RagConductorError::CleanupFailed(format!("remove chunk: {}", e))
            })?;
            removed += 1;
        }

        // Remove embeddings.
        for item in embeddings_tree.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item.map_err(|e| {
                RagConductorError::CleanupFailed(format!("scan embeddings: {}", e))
            })?;
            embeddings_tree.remove(&key).map_err(|e| {
                RagConductorError::CleanupFailed(format!("remove embedding: {}", e))
            })?;
        }

        // Remove inverted-index entries that reference this source.
        let mut index_keys_to_remove = Vec::new();
        for item in index_tree.iter() {
            let (key, value) = item.map_err(|e| {
                RagConductorError::CleanupFailed(format!("scan index: {}", e))
            })?;
            let value_str = String::from_utf8_lossy(&value);
            if value_str.starts_with(source_id) {
                index_keys_to_remove.push(key.to_vec());
            }
        }
        for key in index_keys_to_remove {
            index_tree.remove(&key).map_err(|e| {
                RagConductorError::CleanupFailed(format!("remove index entry: {}", e))
            })?;
        }

        // Remove source metadata.
        sources_tree
            .remove(source_id.as_bytes())
            .map_err(|e| RagConductorError::CleanupFailed(format!("remove source: {}", e)))?;

        Ok(removed)
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