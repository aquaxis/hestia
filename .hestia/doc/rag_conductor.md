# Knowledge Base Orchestrator

**Target Domain**: rag-conductor
**Source**: Design Specification §13.7 (lines 3252-3491)

---

## Overview

rag-conductor is a Conductor that provides knowledge base construction (Ingest), management, and search as an independent process. The legacy `ai-conductor::rag-engine` (TypeScript + LangChain) and `rag-ingest` (Rust) have been fully migrated to rag-conductor. Other conductors call it via agent-cli IPC using structured `rag.*` messages to the `rag` peer.

---

## Configuration and Technology Stack

| Category | Technology |
|----------|------------|
| Binary | `hestia-rag-conductor` (Rust + tokio) |
| Vector DB | Chroma (default) / Qdrant |
| Embedding | Ollama `nomic-embed-text` (768 dimensions) |
| Rust part | `rag-ingest` crate (PDF 7-stage / Web 8-stage pipeline) |
| TS part | `rag-engine` (Vector Search / Embedding / Citation Generation) |
| PDF parsing | PyPDF / pdfplumber / Tesseract OCR (300 DPI, confidence >= 60%) / Camelot (table extraction) |
| Web fetching | trafilatura / BeautifulSoup / CLD3 / fasttext |

---

## Knowledge Base Structure

```
.hestia/rag/
├── sources/                    # Raw data from sources (PDF/HTML snapshots)
│   ├── conductor-work-logs/    # Self-learning accumulation area
│   │   ├── ai/        YYYY-MM-DD_<task_id>.md
│   │   ├── rtl/       YYYY-MM-DD_<task_id>.md
│   │   ├── fpga/      YYYY-MM-DD_<task_id>.md
│   │   ├── asic/      YYYY-MM-DD_<task_id>.md
│   │   ├── pcb/       YYYY-MM-DD_<task_id>.md
│   │   ├── hal/       YYYY-MM-DD_<task_id>.md
│   │   ├── apps/      YYYY-MM-DD_<task_id>.md
│   │   └── debug/     YYYY-MM-DD_<task_id>.md
│   ├── datasheets/             # External reference materials
│   └── vendor-guides/          # Vendor guides
├── chunks/                     # Chunked text
├── embeddings/                 # Vectorized (indexed in Chroma/Qdrant)
├── index-metadata.toml
├── queries/                    # Query logs and hit rates
├── quarantine/                  # Data that failed quality gate (held)
└── queue/                       # Local buffer when rag is offline
```

---

## Ingestion Pipeline

### PDF 7-stage pipeline

Text extraction → OCR fallback → Table extraction → Image extraction → Section recognition → Metadata attachment → Common pipeline

### Web 8-stage pipeline

URL enumeration → robots.txt check → HTTP fetch → Content extraction → Noise removal → Language detection → Metadata attachment → Common pipeline

### Common 6-stage pipeline

Normalization → Quality gate → Chunking (default 1000 tokens / overlap 200) → Embedding (Ollama) → Upsert (Chroma/Qdrant) → Logging

---

## Quality Gate 6 Rules

1. Minimum/maximum character count
2. Language detection
3. HTML noise removal
4. Deduplication (cosine >= 0.95)
5. UTF-8 validity
6. OCR confidence

---

## Incremental Updates and Operations

- Change detection via ETag / SHA-256 → incremental updates (full rebuild of 180 minutes reduced to approximately 3 minutes)
- License management: OSS / free allowed, `vendor-proprietary` (requires `terms_accepted=true`), `CC-BY-*` (attribution required), `unknown` rejected
- PII masking: originals stored with GPG encryption, index contains only masked text
- Cache retention: PDF unlimited / Web 90 days / quarantine 30 days

---

## RPC / CLI / Metrics

### Primary RPCs

| Method | Role |
|--------|------|
| `rag.ingest` | Ingestion (source_type/file_path/url/source_id/all, force/incremental) |
| `rag.search` | Search (query/top_k/filter/trace_id) |
| `rag.cleanup` | Cleanup |
| `rag.status` | Status check |

### Self-learning RPCs

| Method | Role |
|--------|------|
| `rag.ingest_work.v1` | Persist conductor work content (category specified) |
| `rag.search_similar.v1` | Search for similar tasks (fpga.build, asic.synth, etc.) |
| `rag.search_bugfix.v1` | Search past fix cases from error signatures |
| `rag.search_design.v1` | Search past adopted design parameters |

- `IngestJobStatus`: `Queued` / `Processing` / `Completed` / `Failed` / `PartiallyCompleted`
- TypeScript I/F: `RagQuery { text, top_k, filter, trace_id }` / `RagResult { chunks, citations, embedding_time_ms, retrieval_time_ms }`
- MCP tool: `hestia_rag_search`
- CLI: `hestia rag ingest|search|cleanup`

### Prometheus Metrics

`ingest_duration` / `docs_total` / `chunks_total` / `quarantine_total` / `incremental_skipped` / `license_violations` / `cache_size` / `retrieval_seconds` / `hit_ratio` / `work_log_ingested_total` / `similar_task_hits` / `bugfix_search_latency_seconds`

---

## config.toml [rag] Settings

```toml
[rag]
backend = "chroma"                 # "chroma" | "qdrant"
embedding_model = "nomic-embed-text"
top_k = 5
chunk_size = 1000
chunk_overlap = 200
vector_db_url = "http://localhost:8000"
batch_size = 32
retention_days = 90                # Existing sources (datasheets / web, etc.)
retention_days_work_log = 365      # Self-learning conductor-work-logs/ retention period (design_case / bugfix_case are permanent)
self_learning_enabled = true       # Self-learning feature ON/OFF
queue_dir = ".hestia/rag/queue"    # Local buffer when rag is offline
```

---

## Sub-agent Configuration

rag-conductor has 6 types of sub-agents: **planner / designer / ingest (multiple) / search / quality_gate / archivist**, each sharing the knowledge base construction and search workflow. Each sub-agent runs as an independent agent-cli process and coordinates with the rag-conductor main body (peer name `rag`) via `agent-cli send <peer>` IPC.

| Sub-agent | Peer Name | Role | Multiplicity |
|-----------|-----------|------|--------------|
| **planner** | `rag-planner` | Ingestion planning (crawl strategy, source priority, incremental update schedule) | 1 |
| **designer** | `rag-designer` | Knowledge base specification (chunk strategy, metadata schema, embedding model selection, retention policy) | 1 |
| **ingest** | `rag-ingest-{source}` | Document ingestion (PDF 7-stage pipeline / Web 8-stage pipeline) | **N** (started in parallel per source) |
| **search** | `rag-search` | Vector search + reranking (Chroma/Qdrant, `top_k` retrieval, citation generation) | 1 (N under high load) |
| **quality_gate** | `rag-quality` | Quality checks (PII detection / license determination / deduplication / quarantine management) | 1 |
| **archivist** | `rag-archivist` | Self-learning conductor-work-logs/ accumulation pipeline management. Metadata normalization, PII re-masking verification, old log aggregation and summarization | 1 (N under high load) |

**Flow**: planner → designer → ingest (source-parallel) → quality_gate → search (on search request). Self-learning is handled by archivist in an independent flow processing `rag.ingest_work.v1` from other conductors.

---

## Self-learning Feature

When rag-conductor is running, all other conductors and their sub-agents automatically send completed work content to rag-conductor for persistence in the knowledge base. Accumulated cases are searched during subsequent similar tasks and injected as decision-making context for AI agents (self-learning loop).

### Auto-accumulation Categories

| Category | Content | Sender | Timing |
|----------|---------|--------|--------|
| **design_case** | Successful design parameters + build result summary | All conductors | On build success |
| **bugfix_case** | Error → root cause analysis → fix patch → verification result pair | All conductors + ai-conductor | On fix completion |
| **build_log** | Tool output summary | fpga / asic / rtl / apps / hal | On build completion |
| **verification_result** | Simulation / formal verification / DRC / LVS / signoff pass/fail history | Test sub-agents | On verification completion |
| **decision_cot** | Chain-of-thought for important design decisions | Each planner / designer sub-agent | On planning completion |
| **agent_action_log** | AI_LOG from each agent-cli workspace | All agent-cli processes | On exec_job completion |
| **probe_result** | Verification logs from WatcherAgent / ProbeAgent / ValidatorAgent | ai-conductor | On verification completion |

### Knowledge Search Trigger Timing

| Scenario | Trigger | Query | Injection Target |
|----------|---------|-------|------------------|
| New build start | ai-conductor task-router | `rag.search_similar.v1` | planner sub-agent context |
| Error occurrence | Any conductor | `rag.search_bugfix.v1` | exec_job reasoning context |
| Design review | designer sub-agent | `rag.search_design.v1` | designer decision material |
| Patch generation | ai-conductor UpgradeManager | `rag.search_bugfix.v1` | Patch generation prompt |

### Behavior When rag Is Offline

Each conductor buffers work logs to `.hestia/rag/queue/<peer>/`. After rag recovers (detected as `online` by health-checker), ai-conductor flushes them in batch.

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [ai_conductor.md](ai_conductor.md) — ai-conductor overall overview
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator
- [apps_conductor.md](apps_conductor.md) — Application software development orchestrator