# rag-conductor Search Engine Specification

**Target Conductor**: rag-conductor
**Source**: Design Specification §13.7.5 (around lines 3296-3305)

## RPC / CLI / Metrics

### Primary RPCs

| RPC | Parameters | Description |
|-----|-----------|-------------|
| `rag.ingest` | source_type / file_path / url / source_id / all / force / incremental | Document ingestion |
| `rag.search` | query / top_k / filter / trace_id | Vector similarity search |
| `rag.cleanup` | — | Old index cleanup |
| `rag.status` | — | Index status and metrics |

### Self-learning RPCs (§13.7.8)

| RPC | Parameters | Description |
|-----|-----------|-------------|
| `rag.ingest_work.v1` | category / conductor / content / metadata | Conductor work content accumulation |
| `rag.search_similar.v1` | query / top_k | Similar task search |
| `rag.search_bugfix.v1` | query / top_k | Error fix case search |
| `rag.search_design.v1` | query / top_k | Past design parameter search |

### CLI

```bash
hestia rag ingest --source-type <type> --file-path <path>
hestia rag search --query <text> --top-k <n>
hestia rag cleanup
hestia rag status
```

### MCP Tool

`hestia_rag_search` — External tools can use RAG search via Model Context Protocol.

## Vector Search Specification

### Embedding Model

| Item | Value |
|------|-------|
| Model | `nomic-embed-text` |
| Dimensions | 768 |
| Runtime | Ollama (local, privacy-protected) |

### Search Flow

```
1. Query text → Embedding generation (Ollama nomic-embed-text)
2. Similarity search in vector DB (Chroma / Qdrant) (cosine similarity)
3. Retrieve top-k related chunks
4. Citation generation (source, page number, confidence)
5. Return results (chunks + citations + metrics)
```

### TypeScript I/F

```typescript
interface RagQuery {
  text: string;
  top_k: number;
  filter?: Record<string, any>;
  trace_id?: string;
}

interface RagResult {
  chunks: RagChunk[];
  citations: Citation[];
  embedding_time_ms: number;
  retrieval_time_ms: number;
}
```

### Filtering

Search filter conditions allow narrowing results by specific source types, conductor categories, etc.

```json
{
  "filter": {
    "source_type": "datasheet",
    "conductor": "fpga"
  }
}
```

## Prometheus Metrics

| Metric Name | Type | Description |
|-------------|------|-------------|
| `ingest_duration` | Histogram | Ingestion duration |
| `docs_total` | Counter | Total document count |
| `chunks_total` | Counter | Total chunk count |
| `quarantine_total` | Counter | Quarantine hold count |
| `incremental_skipped` | Counter | Skipped count during incremental updates |
| `license_violations` | Counter | License violation count |
| `cache_size` | Gauge | Cache size |
| `retrieval_seconds` | Histogram | Search duration |
| `hit_ratio` | Gauge | Search hit rate |

## Sub-agents

| Sub-agent | Peer Name | Role | Multiplicity |
|-----------|-----------|------|--------------|
| planner | `rag-planner` | Ingestion planning | 1 |
| designer | `rag-designer` | Knowledge base specification | 1 |
| ingest | `rag-ingest-{source}` | Document ingestion | N (source-parallel) |
| search | `rag-search` | Vector search + reranking | 1 (N under high load) |
| quality_gate | `rag-quality` | Quality checks | 1 |
| archivist | `rag-archivist` | Self-learning accumulation pipeline management | 1 (N under high load) |

## Related Documentation

- [rag/binary_spec.md](binary_spec.md) — hestia-rag-cli binary specification
- [rag/ingest_pipeline.md](ingest_pipeline.md) — Ingestion pipeline
- [rag/state_machines.md](state_machines.md) — Index state transitions
- [rag/config_schema.md](config_schema.md) — config.toml [rag] schema