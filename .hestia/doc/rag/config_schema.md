# rag-conductor Configuration Schema

**Target Conductor**: rag-conductor
**Source**: Design Specification §13.7.6 (around lines 3307-3323)

## config.toml [rag] Section

Declaratively defines rag-conductor operational settings.

### Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `backend` | string | `"chroma"` | Vector DB backend (`chroma` / `qdrant`) |
| `embedding_model` | string | `"nomic-embed-text"` | Embedding model name (768 dimensions) |
| `top_k` | integer | 5 | Number of results to retrieve on search |
| `chunk_size` | integer | 1000 | Chunk size (tokens) |
| `chunk_overlap` | integer | 200 | Chunk overlap (tokens) |
| `vector_db_url` | string | `"http://localhost:8000"` | Vector DB connection URL |
| `batch_size` | integer | 32 | Embedding batch size |
| `retention_days` | integer | 90 | Existing source retention period (days) |
| `retention_days_work_log` | integer | 365 | Self-learning conductor-work-logs/ retention period (days) |
| `self_learning_enabled` | boolean | true | Self-learning feature (§13.7.8) ON/OFF |
| `queue_dir` | string | `".hestia/rag/queue"` | Local buffer when rag is offline |

### Configuration Example

```toml
[rag]
backend = "chroma"
embedding_model = "nomic-embed-text"
top_k = 5
chunk_size = 1000
chunk_overlap = 200
vector_db_url = "http://localhost:8000"
batch_size = 32
retention_days = 90
retention_days_work_log = 365
self_learning_enabled = true
queue_dir = ".hestia/rag/queue"
```

## Knowledge Base Structure

```
.hestia/rag/
├── sources/                    # Raw data from sources
│   ├── conductor-work-logs/    # Self-learning accumulation area (§13.7.8)
│   │   ├── ai/      YYYY-MM-DD_<task_id>.md
│   │   ├── rtl/     YYYY-MM-DD_<task_id>.md
│   │   ├── fpga/    YYYY-MM-DD_<task_id>.md
│   │   ├── asic/    YYYY-MM-DD_<task_id>.md
│   │   ├── pcb/     YYYY-MM-DD_<task_id>.md
│   │   ├── hal/     YYYY-MM-DD_<task_id>.md
│   │   ├── apps/    YYYY-MM-DD_<task_id>.md
│   │   └── debug/   YYYY-MM-DD_<task_id>.md
│   ├── datasheets/             # External datasheet PDFs
│   └── vendor-guides/          # Vendor guides
├── chunks/                     # Chunked text
├── embeddings/                 # Vectorized (indexed in Chroma/Qdrant)
├── index-metadata.toml
├── queries/                    # Query logs and hit rates
├── quarantine/                 # Data that failed quality gate (held)
└── queue/                      # Local buffer when offline
```

## Technology Stack

| Category | Technology |
|----------|------------|
| Binary | `hestia-rag-conductor` (Rust + tokio) |
| Vector DB | Chroma (default) / Qdrant |
| Embedding | Ollama `nomic-embed-text` (768 dimensions) |
| Rust part | `rag-ingest` crate (PDF 7-stage / Web 8-stage pipeline) |
| TS part | `rag-engine` (Vector Search / Embedding / Citation Generation) |
| PDF parsing | PyPDF / pdfplumber / Tesseract OCR / Camelot |
| Web fetching | trafilatura / BeautifulSoup / CLD3 / fasttext |

## Related Documentation

- [rag/binary_spec.md](binary_spec.md) — hestia-rag-cli binary specification
- [rag/ingest_pipeline.md](ingest_pipeline.md) — Ingestion pipeline
- [rag/search_engine.md](search_engine.md) — Search engine specification
- [rag/state_machines.md](state_machines.md) — Index state transitions