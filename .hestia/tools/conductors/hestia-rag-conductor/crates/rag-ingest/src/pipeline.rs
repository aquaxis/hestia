//! Ingest pipeline traits and concrete implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use thiserror::Error;
use uuid::Uuid;

/// Default path for the sled vector store.
const DEFAULT_DB_PATH: &str = "/tmp/hestia-rag-store";

/// Global sled DB instance (opened once, shared across pipelines).
static DB: OnceLock<sled::Db> = OnceLock::new();

/// Obtain a handle to the shared sled DB.
pub fn get_db() -> sled::Db {
    DB.get_or_init(|| sled::open(DEFAULT_DB_PATH).expect("failed to open sled DB"))
        .clone()
}

/// Errors during ingestion.
#[derive(Debug, Error)]
pub enum IngestError {
    #[error("extraction failed: {0}")]
    ExtractionFailed(String),
    #[error("chunking failed: {0}")]
    ChunkingFailed(String),
    #[error("embedding failed: {0}")]
    EmbeddingFailed(String),
    #[error("quality gate rejected: {0}")]
    QualityGateRejected(String),
    #[error("storage failed: {0}")]
    StorageFailed(String),
    #[error("indexing failed: {0}")]
    IndexingFailed(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("fetch failed: {0}")]
    FetchFailed(String),
    #[error("parse failed: {0}")]
    ParseFailed(String),
}

/// Result of an ingestion pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    /// Unique source identifier assigned during ingestion.
    pub source_id: String,
    /// Number of chunks produced.
    pub chunks_count: usize,
    /// Final status of the pipeline.
    pub status: IngestStatus,
}

/// Final status of an ingestion pipeline run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IngestStatus {
    /// All stages completed successfully.
    Success,
    /// One or more stages failed.
    Failed,
    /// Pipeline was partially completed (quality gate warnings).
    Partial,
}

/// Trait for ingestion pipelines.
#[async_trait]
pub trait IngestPipeline: Send + Sync {
    /// Run the full ingestion pipeline on the given source.
    async fn ingest(&self, source: &str) -> Result<IngestResult, IngestError>;
}

// ---------------------------------------------------------------------------
// Shared constants
// ---------------------------------------------------------------------------

/// Target chunk size in characters.
const CHUNK_TARGET_SIZE: usize = 500;
/// Maximum chunk size before forced splitting.
const CHUNK_MAX_SIZE: usize = 1000;
/// Minimum chunk length (used by quality gate).
const MIN_CHUNK_LENGTH: usize = 10;
/// Embedding dimension for placeholder vectors.
const PLACEHOLDER_EMBEDDING_DIM: usize = 768;

// ---------------------------------------------------------------------------
// Shared helper functions
// ---------------------------------------------------------------------------

/// Split text into semantic chunks (paragraphs, then sentences).
fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for paragraph in text.split("\n\n") {
        let trimmed = paragraph.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.len() <= CHUNK_TARGET_SIZE {
            if !current.is_empty() && current.len() + trimmed.len() + 2 > CHUNK_MAX_SIZE {
                chunks.push(current.trim().to_string());
                current.clear();
            }
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(trimmed);
        } else {
            // Paragraph is too long; split by sentences.
            for sentence in split_sentences(trimmed) {
                if !current.is_empty() && current.len() + sentence.len() + 1 > CHUNK_MAX_SIZE {
                    chunks.push(current.trim().to_string());
                    current.clear();
                }
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(&sentence);
            }
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

/// Naive sentence splitter (splits on `.`, `?`, `!` followed by whitespace).
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        if ch == '.' || ch == '?' || ch == '!' {
            let end = i + ch.len_utf8();
            let sentence = text[start..end].trim();
            if !sentence.is_empty() {
                sentences.push(sentence.to_string());
            }
            start = end;
        }
    }

    // Trailing text without terminal punctuation.
    if start < text.len() {
        let trailing = text[start..].trim();
        if !trailing.is_empty() {
            sentences.push(trailing.to_string());
        }
    }

    sentences
}

/// FNV-1a hash for deterministic placeholder embedding generation.
fn fnv1a_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Generate a placeholder embedding based on a simple hash of the content.
fn placeholder_embedding(text: &str) -> Vec<f32> {
    let hash = fnv1a_hash(text);
    let mut vec = Vec::with_capacity(PLACEHOLDER_EMBEDDING_DIM);
    for i in 0..PLACEHOLDER_EMBEDDING_DIM {
        let seed = hash.wrapping_add(i as u64);
        // Deterministic but varied values in [-1, 1].
        let val = ((seed as f64).sin() * 0.5) as f32;
        vec.push(val);
    }
    // Normalize to unit length.
    let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in vec.iter_mut() {
            *v /= norm;
        }
    }
    vec
}

/// Try to get an embedding from the Ollama API. Returns `None` on any failure.
async fn ollama_embed(text: &str) -> Option<Vec<f32>> {
    let payload = serde_json::json!({
        "model": "nomic-embed-text",
        "prompt": text
    })
    .to_string();

    let output = tokio::process::Command::new("curl")
        .args([
            "-s",
            "--max-time",
            "10",
            "-X",
            "POST",
            "http://localhost:11434/api/embeddings",
        ])
        .args(["-H", "Content-Type: application/json"])
        .arg("-d")
        .arg(&payload)
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    body.get("embedding")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        })
}

/// Strip HTML tags and decode common entities.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut skip_depth: usize = 0;

    for line in html.lines() {
        let trimmed = line.trim();
        // Skip <script> and <style> blocks.
        if trimmed.starts_with("<script") {
            skip_depth += 1;
        }
        if trimmed.starts_with("<style") {
            skip_depth += 1;
        }
        if skip_depth > 0 {
            if trimmed.contains("</script>") || trimmed.contains("</style>") {
            skip_depth = skip_depth.saturating_sub(1);
        }
            continue;
        }

        for ch in trimmed.chars() {
            if ch == '<' {
                in_tag = true;
            } else if ch == '>' {
                in_tag = false;
            } else if !in_tag {
                result.push(ch);
            }
        }
        result.push('\n');
    }

    // Decode common HTML entities.
    let result = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    // Collapse consecutive blank lines.
    let mut cleaned = String::with_capacity(result.len());
    let mut last_was_blank = false;
    for line in result.lines() {
        if line.trim().is_empty() {
            if !last_was_blank {
                cleaned.push('\n');
                last_was_blank = true;
            }
        } else {
            cleaned.push_str(line.trim());
            cleaned.push('\n');
            last_was_blank = false;
        }
    }

    cleaned
}

/// Store chunks and embeddings into sled trees.
async fn store_chunks(
    db: &sled::Db,
    chunks: &[String],
    embeddings: &[Vec<f32>],
    source_id: &str,
) -> Result<(), IngestError> {
    let chunks_tree = db
        .open_tree("chunks")
        .map_err(|e| IngestError::StorageFailed(format!("open chunks tree: {}", e)))?;
    let embeddings_tree = db
        .open_tree("embeddings")
        .map_err(|e| IngestError::StorageFailed(format!("open embeddings tree: {}", e)))?;

    for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
        let key = format!("{}_{}", source_id, i);

        let chunk_data = serde_json::json!({
            "source_id": source_id,
            "index": i,
            "text": chunk,
        });
        chunks_tree
            .insert(
                key.as_bytes(),
                serde_json::to_vec(&chunk_data)
                    .map_err(|e| IngestError::StorageFailed(format!("serialize chunk: {}", e)))?,
            )
            .map_err(|e| IngestError::StorageFailed(format!("store chunk {}: {}", i, e)))?;

        let emb_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
        embeddings_tree
            .insert(key.as_bytes(), emb_bytes)
            .map_err(|e| IngestError::StorageFailed(format!("store embedding {}: {}", i, e)))?;
    }

    db.flush()
        .map_err(|e| IngestError::StorageFailed(format!("flush db: {}", e)))?;

    Ok(())
}

/// Create search index entries in sled.
async fn index_chunks(
    db: &sled::Db,
    chunks: &[String],
    source_id: &str,
) -> Result<(), IngestError> {
    let index_tree = db
        .open_tree("search_index")
        .map_err(|e| IngestError::IndexingFailed(format!("open index tree: {}", e)))?;
    let sources_tree = db
        .open_tree("sources")
        .map_err(|e| IngestError::IndexingFailed(format!("open sources tree: {}", e)))?;

    // Store source metadata with a Unix timestamp for expiry.
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let source_meta = serde_json::json!({
        "source_id": source_id,
        "chunks_count": chunks.len(),
        "indexed_at": now_secs,
    });
    sources_tree
        .insert(
            source_id.as_bytes(),
            serde_json::to_vec(&source_meta)
                .map_err(|e| IngestError::IndexingFailed(format!("serialize source: {}", e)))?,
        )
        .map_err(|e| IngestError::IndexingFailed(format!("store source meta: {}", e)))?;

    // Token-based inverted index.
    for (i, chunk) in chunks.iter().enumerate() {
        let chunk_key = format!("{}_{}", source_id, i);
        for word in chunk.to_lowercase().split_whitespace() {
            let word = word.trim_matches(|c: char| !c.is_alphanumeric());
            if word.is_empty() {
                continue;
            }
            let index_key = format!("idx:{}:{}", word, chunk_key);
            index_tree
                .insert(index_key.as_bytes(), chunk_key.as_bytes())
                .map_err(|e| IngestError::IndexingFailed(format!("index entry: {}", e)))?;
        }
    }

    db.flush()
        .map_err(|e| IngestError::IndexingFailed(format!("flush db: {}", e)))?;

    Ok(())
}

/// Verify that ingested data is retrievable from sled.
async fn verify_chunks(db: &sled::Db, source_id: &str) -> Result<(), IngestError> {
    let sources_tree = db
        .open_tree("sources")
        .map_err(|e| IngestError::VerificationFailed(format!("open sources tree: {}", e)))?;
    let chunks_tree = db
        .open_tree("chunks")
        .map_err(|e| IngestError::VerificationFailed(format!("open chunks tree: {}", e)))?;

    // Source metadata must exist.
    sources_tree
        .get(source_id.as_bytes())
        .map_err(|e| IngestError::VerificationFailed(format!("read source: {}", e)))?
        .ok_or_else(|| {
            IngestError::VerificationFailed(format!("source {} not found in index", source_id))
        })?;

    // At least one chunk must exist.
    let prefix = format!("{}_", source_id);
    let mut found = false;
    for item in chunks_tree.scan_prefix(prefix.as_bytes()) {
        item.map_err(|e| IngestError::VerificationFailed(format!("scan chunks: {}", e)))?;
        found = true;
        break;
    }

    if !found {
        return Err(IngestError::VerificationFailed(format!(
            "no chunks found for source {}",
            source_id
        )));
    }

    Ok(())
}

/// Run quality gate checks on chunks and embeddings.
fn run_quality_gate(chunks: &[String], embeddings: &[Vec<f32>]) -> Result<(), IngestError> {
    if chunks.is_empty() {
        return Err(IngestError::QualityGateRejected(
            "no chunks to validate".to_string(),
        ));
    }
    if embeddings.len() != chunks.len() {
        return Err(IngestError::QualityGateRejected(format!(
            "embedding count ({}) != chunk count ({})",
            embeddings.len(),
            chunks.len()
        )));
    }
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.trim().len() < MIN_CHUNK_LENGTH {
            return Err(IngestError::QualityGateRejected(format!(
                "chunk {} too short ({} chars, min {})",
                i,
                chunk.len(),
                MIN_CHUNK_LENGTH
            )));
        }
    }
    if let Some(dim) = embeddings.first().map(|v| v.len()) {
        for (i, emb) in embeddings.iter().enumerate() {
            if emb.len() != dim {
                return Err(IngestError::QualityGateRejected(format!(
                    "embedding {} dimension mismatch ({} != {})",
                    i,
                    emb.len(),
                    dim
                )));
            }
            if emb.is_empty() {
                return Err(IngestError::QualityGateRejected(format!(
                    "embedding {} is empty",
                    i
                )));
            }
        }
    }
    Ok(())
}

/// Generate embeddings for a slice of chunks, falling back to placeholders.
async fn embed_chunks(chunks: &[String]) -> Result<Vec<Vec<f32>>, IngestError> {
    let mut embeddings = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        match ollama_embed(chunk).await {
            Some(vec) => embeddings.push(vec),
            None => {
                tracing::warn!("Ollama embedding unavailable, using placeholder for chunk");
                embeddings.push(placeholder_embedding(chunk));
            }
        }
    }
    Ok(embeddings)
}

// ---------------------------------------------------------------------------
// PdfPipeline
// ---------------------------------------------------------------------------

/// PDF ingestion pipeline (7 stages).
///
/// Stages: extract_text -> chunk -> embed -> quality_gate -> store -> index -> verify
pub struct PdfPipeline {
    db: sled::Db,
}

impl PdfPipeline {
    /// Create a new PDF ingestion pipeline.
    pub fn new() -> Self {
        Self { db: get_db() }
    }

    /// Stage 1: Extract text content from the PDF using `pdftotext`.
    async fn extract_text(&self, source: &str) -> Result<String, IngestError> {
        tracing::info!("PdfPipeline: stage 1 - extract_text");

        let output = tokio::process::Command::new("pdftotext")
            .arg(source)
            .arg("-") // output to stdout
            .output()
            .await
            .map_err(|e| IngestError::ExtractionFailed(format!("pdftotext failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IngestError::ExtractionFailed(format!(
                "pdftotext error: {}",
                stderr
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        if text.trim().is_empty() {
            return Err(IngestError::ExtractionFailed(
                "no text extracted from PDF".to_string(),
            ));
        }

        Ok(text)
    }

    /// Stage 2: Chunk the extracted text into semantic segments.
    async fn chunk(&self, text: &str) -> Result<Vec<String>, IngestError> {
        tracing::info!("PdfPipeline: stage 2 - chunk ({} bytes)", text.len());
        let chunks = chunk_text(text);
        if chunks.is_empty() {
            return Err(IngestError::ChunkingFailed(
                "no chunks produced from text".to_string(),
            ));
        }
        Ok(chunks)
    }

    /// Stage 3: Generate vector embeddings for each chunk.
    async fn embed(&self, chunks: &[String]) -> Result<Vec<Vec<f32>>, IngestError> {
        tracing::info!("PdfPipeline: stage 3 - embed ({} chunks)", chunks.len());
        embed_chunks(chunks).await
    }

    /// Stage 4: Quality gate -- validate embeddings and chunks.
    async fn quality_gate(
        &self,
        chunks: &[String],
        embeddings: &[Vec<f32>],
    ) -> Result<(), IngestError> {
        tracing::info!("PdfPipeline: stage 4 - quality_gate ({} chunks)", chunks.len());
        run_quality_gate(chunks, embeddings)
    }

    /// Stage 5: Store chunks and embeddings in the vector store.
    async fn store(
        &self,
        chunks: &[String],
        embeddings: &[Vec<f32>],
        source_id: &str,
    ) -> Result<(), IngestError> {
        tracing::info!("PdfPipeline: stage 5 - store ({} chunks)", chunks.len());
        store_chunks(&self.db, chunks, embeddings, source_id).await
    }

    /// Stage 6: Update the search index.
    async fn index(&self, chunks: &[String], source_id: &str) -> Result<(), IngestError> {
        tracing::info!("PdfPipeline: stage 6 - index ({} chunks)", chunks.len());
        index_chunks(&self.db, chunks, source_id).await
    }

    /// Stage 7: Verify that the ingested data is retrievable.
    async fn verify(&self, source_id: &str) -> Result<(), IngestError> {
        tracing::info!("PdfPipeline: stage 7 - verify");
        verify_chunks(&self.db, source_id).await
    }
}

impl Default for PdfPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IngestPipeline for PdfPipeline {
    async fn ingest(&self, source: &str) -> Result<IngestResult, IngestError> {
        let source_id = format!("pdf_{}", Uuid::new_v4());

        // Stage 1
        let text = self.extract_text(source).await?;
        // Stage 2
        let chunks = self.chunk(&text).await?;
        // Stage 3
        let embeddings = self.embed(&chunks).await?;
        // Stage 4
        self.quality_gate(&chunks, &embeddings).await?;
        // Stage 5
        self.store(&chunks, &embeddings, &source_id).await?;
        // Stage 6
        self.index(&chunks, &source_id).await?;
        // Stage 7
        self.verify(&source_id).await?;

        Ok(IngestResult {
            source_id,
            chunks_count: chunks.len(),
            status: IngestStatus::Success,
        })
    }
}

// ---------------------------------------------------------------------------
// WebPipeline
// ---------------------------------------------------------------------------

/// Web page ingestion pipeline (8 stages).
///
/// Stages: fetch -> parse -> chunk -> embed -> quality_gate -> store -> index -> verify
pub struct WebPipeline {
    db: sled::Db,
}

impl WebPipeline {
    /// Create a new web ingestion pipeline.
    pub fn new() -> Self {
        Self { db: get_db() }
    }

    /// Stage 1: Fetch the web page content using `curl`.
    async fn fetch(&self, url: &str) -> Result<String, IngestError> {
        tracing::info!("WebPipeline: stage 1 - fetch");

        let output = tokio::process::Command::new("curl")
            .args(["-s", "-L", "--max-time", "30", url])
            .output()
            .await
            .map_err(|e| IngestError::FetchFailed(format!("curl failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IngestError::FetchFailed(format!("curl error: {}", stderr)));
        }

        let html = String::from_utf8_lossy(&output.stdout).to_string();
        if html.trim().is_empty() {
            return Err(IngestError::FetchFailed(
                "empty response from URL".to_string(),
            ));
        }

        Ok(html)
    }

    /// Stage 2: Parse the HTML into structured text.
    async fn parse(&self, raw: &str) -> Result<String, IngestError> {
        tracing::info!("WebPipeline: stage 2 - parse");
        let text = strip_html(raw);
        if text.trim().is_empty() {
            return Err(IngestError::ParseFailed(
                "no text content in HTML".to_string(),
            ));
        }
        Ok(text)
    }

    /// Stage 3: Chunk the parsed text into semantic segments.
    async fn chunk(&self, text: &str) -> Result<Vec<String>, IngestError> {
        tracing::info!("WebPipeline: stage 3 - chunk ({} bytes)", text.len());
        let chunks = chunk_text(text);
        if chunks.is_empty() {
            return Err(IngestError::ChunkingFailed(
                "no chunks produced from text".to_string(),
            ));
        }
        Ok(chunks)
    }

    /// Stage 4: Generate vector embeddings for each chunk.
    async fn embed(&self, chunks: &[String]) -> Result<Vec<Vec<f32>>, IngestError> {
        tracing::info!("WebPipeline: stage 4 - embed ({} chunks)", chunks.len());
        embed_chunks(chunks).await
    }

    /// Stage 5: Quality gate -- validate embeddings and chunks.
    async fn quality_gate(
        &self,
        chunks: &[String],
        embeddings: &[Vec<f32>],
    ) -> Result<(), IngestError> {
        tracing::info!("WebPipeline: stage 5 - quality_gate ({} chunks)", chunks.len());
        run_quality_gate(chunks, embeddings)
    }

    /// Stage 6: Store chunks and embeddings in the vector store.
    async fn store(
        &self,
        chunks: &[String],
        embeddings: &[Vec<f32>],
        source_id: &str,
    ) -> Result<(), IngestError> {
        tracing::info!("WebPipeline: stage 6 - store ({} chunks)", chunks.len());
        store_chunks(&self.db, chunks, embeddings, source_id).await
    }

    /// Stage 7: Update the search index.
    async fn index(&self, chunks: &[String], source_id: &str) -> Result<(), IngestError> {
        tracing::info!("WebPipeline: stage 7 - index ({} chunks)", chunks.len());
        index_chunks(&self.db, chunks, source_id).await
    }

    /// Stage 8: Verify that the ingested data is retrievable.
    async fn verify(&self, source_id: &str) -> Result<(), IngestError> {
        tracing::info!("WebPipeline: stage 8 - verify");
        verify_chunks(&self.db, source_id).await
    }
}

impl Default for WebPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IngestPipeline for WebPipeline {
    async fn ingest(&self, source: &str) -> Result<IngestResult, IngestError> {
        let source_id = format!("web_{}", Uuid::new_v4());

        // Stage 1
        let raw = self.fetch(source).await?;
        // Stage 2
        let text = self.parse(&raw).await?;
        // Stage 3
        let chunks = self.chunk(&text).await?;
        // Stage 4
        let embeddings = self.embed(&chunks).await?;
        // Stage 5
        self.quality_gate(&chunks, &embeddings).await?;
        // Stage 6
        self.store(&chunks, &embeddings, &source_id).await?;
        // Stage 7
        self.index(&chunks, &source_id).await?;
        // Stage 8
        self.verify(&source_id).await?;

        Ok(IngestResult {
            source_id,
            chunks_count: chunks.len(),
            status: IngestStatus::Success,
        })
    }
}