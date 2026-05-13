# rag-conductor Ingestion Pipeline

**Target Conductor**: rag-conductor
**Source**: Design Specification §13.7.3 (around lines 3282-3287), §13.7.4 (around lines 3289-3294)

## Pipeline Configuration

### PDF 7-stage Pipeline

| Stage | Processing | Tools Used |
|-------|-----------|------------|
| 1 | Text extraction | PyPDF / pdfplumber |
| 2 | OCR fallback | Tesseract OCR (300 DPI, confidence >= 60%) |
| 3 | Table extraction | Camelot |
| 4 | Image extraction | PyPDF / pdfplumber |
| 5 | Section recognition | Header/heading detection |
| 6 | Metadata attachment | Source, page number, creation date, etc. |
| 7 | Pass to common pipeline | — |

### Web 8-stage Pipeline

| Stage | Processing | Tools Used |
|-------|-----------|------------|
| 1 | URL enumeration | Sitemap / crawl |
| 2 | robots.txt check | robots.txt parser |
| 3 | HTTP fetch | HTTP client |
| 4 | Content extraction | trafilatura |
| 5 | Noise removal | BeautifulSoup |
| 6 | Language detection | CLD3 / fasttext |
| 7 | Metadata attachment | URL, title, date, etc. |
| 8 | Pass to common pipeline | — |

### Common 6-stage Pipeline

| Stage | Processing | Description |
|-------|-----------|-------------|
| 1 | Normalization | Unicode normalization, whitespace unification, HTML entity expansion |
| 2 | Quality gate | Quality check with 6 rules (failure → quarantine) |
| 3 | Chunking | Default 1000 tokens / overlap 200 |
| 4 | Embedding | Ollama `nomic-embed-text` (768 dimensions) |
| 5 | Upsert | Vector registration to Chroma / Qdrant |
| 6 | Logging | Ingestion result log recording |

## Quality Gate 6 Rules

| Rule | Condition | On Pass | On Fail |
|------|-----------|---------|---------|
| Minimum character count | Chunk >= minimum threshold | Proceed to next stage | Quarantine |
| Maximum character count | Chunk <= maximum threshold | Proceed to next stage | Split and retry |
| Language detection | Supported language | Proceed to next stage | Quarantine |
| HTML noise removal | No noise | Proceed to next stage | Reprocess |
| Deduplication (cosine >= 0.95) | Dissimilar to existing chunks | Proceed to next stage | Skip |
| UTF-8 validity | Valid encoding | Proceed to next stage | Quarantine |
| OCR confidence | >= 60% | Proceed to next stage | Quarantine |

## Incremental Updates

Changes are detected via ETag / SHA-256, and only modified sources are re-ingested. A full rebuild that takes 180 minutes is reduced to approximately 3 minutes with incremental updates.

## License Management

| License Type | Ingestion Allowed | Condition |
|-------------|-------------------|-----------|
| OSS / free | Allowed | Unconditional |
| CC-BY-* | Allowed | Attribution required |
| vendor-proprietary | Conditionally allowed | `terms_accepted=true` required |
| unknown | Rejected | — |

## PII Masking

- Originals: stored with GPG encryption
- Index: masked text only (PII replaced with `[REDACTED]`, etc.)
- Masking targets: names, email addresses, phone numbers, IP addresses, etc.

## self_learning Ingestion (§13.7.8)

Automatic accumulation pipeline for work content from other conductors.

| Category | Content | Timing |
|----------|---------|--------|
| design_case | Successful design parameters + build results | On build success |
| bugfix_case | Error → fix patch → verification results | On fix completion |
| build_log | Tool output summary | On build completion |
| verification_result | Verification pass/fail history | On verification completion |
| decision_cot | Design decision CoT | On planning/design completion |
| agent_action_log | Work logs | On exec_job completion |
| probe_result | Compatibility probe results | On verification completion |

## Related Documentation

- [rag/config_schema.md](config_schema.md) — config.toml [rag] schema
- [rag/search_engine.md](search_engine.md) — Search engine specification
- [rag/state_machines.md](state_machines.md) — Index state transitions
- [rag/error_types.md](error_types.md) — rag-conductor error codes