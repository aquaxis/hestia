# rag-conductor CLI Binary Specification

**Target Conductor**: rag-conductor
**Source**: Design Specification §15 (around lines 3631-3730), §13.7 (around lines 3252-3491)

## Binary Name

`hestia-rag-cli`

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `ingest` | Document ingestion (PDF / Web / Source code / conductor-work-logs) |
| `search` | Vector similarity search (query / top_k / filter / trace_id) |
| `cleanup` | Old index cleanup (delete data past retention period) |
| `status` | Index status and metrics display |

## Common Options (CommonOpts)

| Option | Value | Description |
|--------|-------|-------------|
| `--output` | `human` \| `json` | Output format (default: human) |
| `--timeout` | `<seconds>` | RPC timeout |
| `--registry` | `<path>` | agent-cli registry path |
| `--config` | `<path>` | Configuration file path |
| `--verbose` | — | Verbose logging |

## Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## ingest Subcommand Options

| Option | Value | Description |
|--------|-------|-------------|
| `--source-type` | `pdf` / `web` / `source` / `all` | Ingestion source type |
| `--file-path` | `<path>` | Single file ingestion |
| `--url` | `<url>` | URL ingestion |
| `--source-id` | `<id>` | Source ID specification |
| `--force` | — | Force re-ingestion |
| `--incremental` | — | Incremental update mode |

## search Subcommand Options

| Option | Value | Description |
|--------|-------|-------------|
| `--query` | `<text>` | Search query |
| `--top-k` | `<n>` | Number of results (default: 5) |
| `--filter` | `<json>` | Filter conditions |
| `--trace-id` | `<id>` | Trace ID |

## CLI Usage Examples

```bash
# PDF ingestion
hestia rag ingest --source-type pdf --file-path datasheets/STM32F103.pdf

# Web ingestion
hestia rag ingest --source-type web --url https://example.com/guide

# Incremental ingestion for all sources
hestia rag ingest --source-type all --incremental

# Search
hestia rag search --query "STM32F103 SPI pinout" --top-k 5

# Cleanup
hestia rag cleanup

# Status display
hestia rag status
```

## Related Documentation

- [rag/config_schema.md](config_schema.md) — config.toml [rag] schema
- [rag/message_methods.md](message_methods.md) — rag.* method list
- [rag/ingest_pipeline.md](ingest_pipeline.md) — Ingestion pipeline
- [rag/search_engine.md](search_engine.md) — Search engine specification