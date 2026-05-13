# rag-conductor Error Codes

**Target Conductor**: rag-conductor
**Source**: Design Specification §14.3 (around lines 3565-3581)

## Error Code Range

rag-conductor error codes use the **-32600 to -32699** range.

## Error Categories

### Ingest

| Code | Name | Description |
|------|------|-------------|
| -32600 | INGEST_FAILED | Ingestion processing failure |
| -32601 | INGEST_SOURCE_NOT_FOUND | Ingestion source not found |
| -32602 | INGEST_UNSUPPORTED_SOURCE_TYPE | Unsupported source type |

### PDF

| Code | Name | Description |
|------|------|-------------|
| -32610 | PDF_TEXT_EXTRACTION_FAILED | PDF text extraction failure |
| -32611 | PDF_OCR_FAILED | OCR processing failure (Tesseract) |
| -32612 | PDF_TABLE_EXTRACTION_FAILED | Table extraction failure (Camelot) |
| -32613 | PDF_IMAGE_EXTRACTION_FAILED | Image extraction failure |

### Web

| Code | Name | Description |
|------|------|-------------|
| -32620 | WEB_FETCH_FAILED | HTTP fetch failure |
| -32630 | WEB_ROBOTS_TXT_DENIED | Access denied by robots.txt |
| -32621 | WEB_CONTENT_EXTRACTION_FAILED | Content extraction failure (trafilatura) |
| -32622 | WEB_LANGUAGE_DETECTION_FAILED | Language detection failure (CLD3 / fasttext) |

### Quality Gate

| Code | Name | Description |
|------|------|-------------|
| -32640 | QUALITY_GATE_FAILED | Quality gate failure |
| -32641 | QUALITY_MIN_LENGTH | Below minimum character count |
| -32642 | QUALITY_MAX_LENGTH | Exceeds maximum character count |
| -32643 | QUALITY_DUPLICATE | Duplicate detected (cosine >= 0.95) |
| -32644 | QUALITY_UTF8_INVALID | UTF-8 validity error |
| -32645 | QUALITY_OCR_LOW_CONFIDENCE | OCR confidence below threshold (< 60%) |

### Chunk / Embedding

| Code | Name | Description |
|------|------|-------------|
| -32650 | CHUNK_SPLIT_FAILED | Chunk splitting failure |
| -32651 | EMBEDDING_FAILED | Embedding generation failure (Ollama) |
| -32652 | EMBEDDING_MODEL_NOT_FOUND | Embedding model not found |
| -32653 | UPSERT_FAILED | Upsert to vector DB failed |

### Vector / Search

| Code | Name | Description |
|------|------|-------------|
| -32660 | VECTOR_DB_CONNECTION_FAILED | Vector DB connection failure (Chroma / Qdrant) |
| -32661 | SEARCH_FAILED | Search execution failure |
| -32662 | SEARCH_TIMEOUT | Search timeout |

### License / PII

| Code | Name | Description |
|------|------|-------------|
| -32670 | LICENSE_VIOLATION | License violation (unknown / vendor-proprietary without terms_accepted) |
| -32671 | PII_DETECTION_FAILED | PII detection processing failure |
| -32672 | PII_MASKING_FAILED | PII masking processing failure |

### Scheduler / Cache

| Code | Name | Description |
|------|------|-------------|
| -32680 | SCHEDULER_QUEUE_FULL | Ingestion queue full |
| -32681 | CACHE_EXPIRED | Cache expired |
| -32682 | CACHE_READ_ERROR | Cache read error |

## IngestJobStatus

| Status | Description |
|--------|-------------|
| `Queued` | Waiting in queue |
| `Processing` | Processing |
| `Completed` | Completed |
| `Failed` | Failed |
| `PartiallyCompleted` | Partially completed (some sources failed) |

## Related Documentation

- [rag/message_methods.md](message_methods.md) — rag.* method list
- [rag/ingest_pipeline.md](ingest_pipeline.md) — Ingestion pipeline
- [rag/search_engine.md](search_engine.md) — Search engine specification
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry