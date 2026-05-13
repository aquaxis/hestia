# rag-conductor Message Method List

**Target Conductor**: rag-conductor
**Source**: Design Specification §13.7.5 (around lines 3296-3305), §14 (around lines 3492-3630)

## Transport

All communication is unified via agent-cli native IPC. Peer name: `rag`.

## rag.* Method List

### Ingestion

| Method | Direction | Description |
|--------|-----------|-------------|
| `rag.ingest` | Request | Document ingestion (source_type / file_path / url / source_id / all / force / incremental) |

### Search

| Method | Direction | Description |
|--------|-----------|-------------|
| `rag.search` | Request | Vector similarity search (query / top_k / filter / trace_id) |

### Management

| Method | Direction | Description |
|--------|-----------|-------------|
| `rag.cleanup` | Request | Old index cleanup (delete data past retention period) |
| `rag.status` | Request | Index status and metrics retrieval |

### Self-learning (§13.7.8)

| Method | Direction | Description |
|--------|-----------|-------------|
| `rag.ingest_work.v1` | Request | Conductor work content accumulation (design_case / bugfix_case / build_log, etc.) |
| `rag.search_similar.v1` | Request | Similar task search (retrieve past cases of the same type) |
| `rag.search_bugfix.v1` | Request | Error fix case search |
| `rag.search_design.v1` | Request | Past design parameter search |

### conductor-core Common

| Method | Direction | Description |
|--------|-----------|-------------|
| `system.health.v1` | Request | Health check |

## Payload Examples

### rag.ingest Request

```json
{
  "method": "rag.ingest",
  "params": {
    "source_type": "pdf",
    "file_path": "datasheets/STM32F103.pdf",
    "force": false
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### rag.search Request

```json
{
  "method": "rag.search",
  "params": {
    "query": "STM32F103 SPI pinout",
    "top_k": 5,
    "filter": { "source_type": "datasheet" }
  },
  "id": "msg_2026-05-01T12:00:00Z_def456",
  "trace_id": "trace_xyz789"
}
```

### rag.ingest_work.v1 Request

```json
{
  "method": "rag.ingest_work.v1",
  "params": {
    "category": "design_case",
    "conductor": "fpga",
    "content": "<markdown>",
    "metadata": { "target": "artix7", "outcome": "success" }
  },
  "id": "msg_2026-05-01T12:00:00Z_ghi789"
}
```

## TypeScript I/F

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

## MCP Tool

`hestia_rag_search` — External tools can use RAG search via Model Context Protocol.

## Related Documentation

- [rag/binary_spec.md](binary_spec.md) — hestia-rag-cli binary specification
- [rag/ingest_pipeline.md](ingest_pipeline.md) — Ingestion pipeline
- [rag/search_engine.md](search_engine.md) — Search engine specification
- [rag/error_types.md](error_types.md) — rag-conductor error codes