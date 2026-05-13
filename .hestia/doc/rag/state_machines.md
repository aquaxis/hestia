# rag-conductor Index State Transitions

**Target Conductor**: rag-conductor
**Source**: Design Specification §13.7 (lines 3252-3491)

## Index State Transitions

State machine that manages the lifecycle of Ingestion jobs.

```
Queued → Processing → Completed
                  ├── Failed
                  └── PartiallyCompleted
```

### IngestJobStatus

| State | Description |
|-------|-------------|
| Queued | Waiting in queue (ingestion request received, awaiting resources) |
| Processing | Processing (PDF/Web pipeline running) |
| Completed | All sources ingested successfully |
| Failed | Ingestion failed (fatal error) |
| PartiallyCompleted | Partially completed (some sources failed, others succeeded) |

## Per-source Pipeline States

### PDF Ingestion State Transitions

```
Text extraction → OCR fallback → Table extraction → Image extraction
    → Section recognition → Metadata attachment → Common pipeline
```

### Web Ingestion State Transitions

```
URL enumeration → robots.txt check → HTTP fetch → Content extraction
    → Noise removal → Language detection → Metadata attachment → Common pipeline
```

### Common Pipeline State Transitions

```
Normalization → Quality gate → Chunking → Embedding → Upsert → Logging
```

## Quality Gate Decisions

Data that fails the quality gate is held in `quarantine/`.

| Rule | Condition | Action |
|------|-----------|--------|
| Minimum character count | Chunk too short | Quarantine |
| Maximum character count | Chunk too long | Split and retry |
| Language detection | Unsupported language | Quarantine |
| HTML noise removal | Noise remaining | Reprocess |
| Deduplication (cosine >= 0.95) | High similarity to existing chunk | Skip |
| UTF-8 validity | Invalid encoding | Quarantine |
| OCR confidence | < 60% | Quarantine |

## Incremental Update Flow

```
Change detection (ETag / SHA-256)
    |
    ├── No changes → Skip (incremental_skipped metric updated)
    |
    └── Changes detected → Re-ingest only the affected sources
```

## License Decision Flow

```
Source ingestion request
    |
    ├── OSS / free → Ingestion allowed
    ├── CC-BY-* → Ingestion with attribution
    ├── vendor-proprietary → terms_accepted=true required
    └── unknown → Rejected (license_violations metric updated)
```

## PII Masking Flow

```
Original text (may contain PII)
    |
    ├── PII detection → Masking applied
    |
    ├── Original → Stored with GPG encryption
    |
    └── Index → Masked text only
```

## Cache Retention Periods

| Source Type | Retention Period |
|------------|-----------------|
| PDF | Unlimited |
| Web | 90 days |
| quarantine | 30 days |
| conductor-work-logs (design_case/bugfix_case) | Unlimited |
| conductor-work-logs (build_log, etc.) | 365 days |

## Related Documentation

- [rag/ingest_pipeline.md](ingest_pipeline.md) — Ingestion pipeline details
- [rag/search_engine.md](search_engine.md) — Search engine specification
- [rag/config_schema.md](config_schema.md) — config.toml [rag] schema
- [rag/error_types.md](error_types.md) — rag-conductor error codes