# Shared Services Layer

**Scope**: Shared services (Layer 5) / rag-conductor
**Source**: Design specification §13 (around lines 3186-3491)

---

## 1. Overview

The Shared Services Layer (Layer 5) provides **6 cross-cutting services**. Each service is launched as an agent-cli peer and is available to all conductors.

**Table HD-040: 6 Shared Services**

| Service | Crate / Binary | Socket | Primary specification |
|---------|---------------------|---------|---------|
| HDL LSP Broker | `hdl-lsp-broker` | agent-cli peer `lsp` | `common/hdl_lsp_broker.md` |
| WASM Waveform Viewer | `waveform-core` (cdylib + rlib) | agent-cli peer `waveform` (host path) / WebWorker (WASM path) | `common/wasm_waveform_viewer.md` |
| Constraint Bridge | `constraint-bridge` | agent-cli peer `constraint-bridge` | `common/constraint_bridge.md` |
| IP Manager | `ip-manager` | agent-cli peer `ip-manager` | `common/ip_manager.md` |
| CI/CD API | `cicd-api` | agent-cli peer `cicd` | `common/cicd_api.md` |
| Observability | `observability` (Prometheus+tracing+OTLP) | agent-cli peer `observability` + HTTP `:9090/metrics` + OTLP `:4317` + Health `:8080` | `common/observability.md` |

---

## 2. HDL LSP Broker (§13.1)

An LSP proxy that provides a unified interface for Verilog / SystemVerilog / VHDL / Verilog-AMS LSP server clusters. From the frontend (VSCode Extension / Tauri IDE), completion, diagnostics, go-to-definition, references, and rename for multiple languages are available via a single connection.

### 2.1 Key Types

- `HdlLanguage`: `Verilog` / `SystemVerilog` / `Vhdl` / `VerilogAms`
- `LspServerConfig`: LSP server configuration
- `RoutingTable`: Language → LSP server routing

### 2.2 Supported LSP Servers

| LSP Server | Version | Supported Languages |
|-----------|----------|---------|
| svls | v0.2.x | SystemVerilog |
| vhdl_ls | v0.3.x | VHDL |
| verilog-ams-ls | v0.1.x | Verilog-AMS |

### 2.3 Extension Map

| Extension | Language |
|-------|------|
| `.v` | Verilog |
| `.sv` / `.svh` | SystemVerilog |
| `.vhd` / `.vhdl` | VHDL |
| `.va` / `.vams` | Verilog-AMS |

### 2.4 Parameter Defaults

- `max_instances=4`
- `idle_timeout_secs=300`

---

## 3. WASM Waveform Viewer (§13.2)

A waveform viewer capable of streaming-parse VCD / FST / GHW / EVCD. The `waveform-core` crate is built as `cdylib` + `rlib`; browsers load it via WebWorker + SharedArrayBuffer, while Tauri / VSCode WebView uses the same crate directly. Targets 60fps at 1 million sample display.

### 3.1 Supported Formats

`WaveformFormat`: `Vcd` / `Fst` / `Ghw` / `Evcd`

### 3.2 Signal Model

- `Signal`: `id`, `full_name`, `display_name`, `bit_width`, `signal_type` (`Wire` / `Reg` / `Integer` / `Real`), `scope`
- `SignalValue`: `Logic(char)` / `Vector{bits, hex}` / `Real(f64)` / `String`

### 3.3 Performance Target

Target 60fps at 1 million sample display. WebWorker + SharedArrayBuffer avoids blocking the main thread.

---

## 4. Constraint Bridge (§13.3)

A mutual conversion engine for constraint files. Using `ConstraintModel` as an intermediate representation, conversion between N formats is possible with 2N parsers/generators (reduced from N x N to N + M).

### 4.1 Supported Formats

| Format | Target | Extension |
|------------|------|-------|
| XDC | Xilinx | `.xdc` |
| PCF | iCE40 | `.pcf` |
| SDC | Synopsys | `.sdc` |
| Efinity XML | Efinix | XML |
| QSF | Intel Quartus | `.qsf` |
| UCF | Legacy ISE | `.ucf` |

`ConstraintFormat`: `Xdc` / `Pcf` / `Sdc` (others are extended types)

### 4.2 Key Structures

- `ClockConstraint`
- `PinConstraint`
- `TimingConstraint`
- `PlacementConstraint`
- `RawConstraint`

### 4.3 Supported Properties

Covers pin assignment, I/O standards, drive strength, slew rate, and differential pairs.

---

## 5. IP Manager (§13.4)

Provides IP core registration, search, version resolution, license management, and dependency resolution. Uses `petgraph`'s DAG-based resolution algorithm (topological sort) to resolve multi-level dependencies.

### 5.1 IP Core Data Model

- `IpCore`: `id` (`com.vendor.name`) / `version` (semver) / `vendor` / `library` / `device_families[]` / `supported_languages[]` / `dependencies[]` / `files[]` / `parameters[]`
- `IpDependency`: `ip_id` + `VersionReq` + `optional`
- `IpFile.type`: `rtl` / `testbench` / `doc` / `constraint`, `language`: `verilog` / `vhdl` / others

### 5.2 Dependency Resolution

Multi-level dependency resolution via DAG-based topological sort using `petgraph`.

### 5.3 Version Management

Resolution via version requirements (VersionReq) based on semver (semantic versioning).

### 5.4 License Classification

| Classification | Content |
|------|------|
| `Oss` | MIT / Apache-2.0 / BSD / GPL / ISC / CC0 |
| `VendorProprietary` | FlexLM / seat restrictions |
| `Unknown` | Rejected |

---

## 6. CI/CD API (§13.5)

Declaratively defines CI/CD pipelines and executes them across multiple backends (GitHub Actions / GitLab CI / Local).

### 6.1 Backends

`Backend`: `GithubActions` / `GitlabCi` / `LocalPipeline`

### 6.2 Key Structures

- `PipelineDefinition` / `PipelineStage` / `PipelineJob`
- `StageCondition`: `Always` / `OnSuccess` / `OnFailure` / `Custom`

### 6.3 Control Features

Artifact retention, retry policy, timeout secs, and cache key control via JSON.

---

## 7. Observability (§13.6)

### 7.1 Metrics

- `prometheus` crate, port `:9090/metrics`
- Per-conductor / per-service counters / gauges / histograms

### 7.2 Logging

- `tracing` crate, JSON output to `.hestia/logs/observability.log`

### 7.3 Tracing

- OpenTelemetry SDK, OTLP/gRPC `:4317`

### 7.4 Health Check

- HTTP `:8080/{health, ready, live}`
- `HealthStatus`: `Healthy` / `Degraded` / `Unhealthy`

### 7.5 Configuration

Metrics / health are aggregated per `ConductorName` (`Ai` / `Fpga` / `Asic` / `Pcb` / `Debug` / `Rag`).

---

## 8. rag-conductor — Knowledge Base Orchestrator (§13.7)

rag-conductor is the **6th Conductor**, providing knowledge base construction (Ingest), management, and search as an independent process. Primary specification: `.hestia/doc/rag_conductor.md` and `.hestia/doc/rag/*.md`.

> Separation from ai-conductor: The former `ai-conductor::rag-engine` (TypeScript + LangChain) and `rag-ingest` (Rust) have been fully migrated to rag-conductor. ai-conductor calls it by sending `rag.*` structured messages to the `rag` peer via agent-cli IPC.

### 8.1 Architecture and Technology Stack

| Category | Technology |
|------|------|
| Binary | `hestia-rag-conductor` (Rust + tokio) |
| Vector DB | Chroma (default) / Qdrant |
| Embedding | Ollama `nomic-embed-text` (768 dimensions) |
| Rust portion | `rag-ingest` crate (PDF 7-stage / Web 8-stage pipeline) |
| TS portion | `rag-engine` (Vector Search / Embedding / Citation Generation) |
| PDF parsing | PyPDF / pdfplumber / Tesseract OCR (300 DPI, confidence >= 60%) / Camelot (table extraction) |
| Web retrieval | trafilatura / BeautifulSoup / CLD3 / fasttext |

### 8.2 Knowledge Base Structure

```
.hestia/rag/
├── sources/        # Raw data from sources (PDF, HTML snapshots)
├── chunks/         # Chunked text
├── embeddings/     # Vectorized (indexed in Chroma/Qdrant)
├── index-metadata.toml
├── queries/        # Query logs and hit rates
└── quarantine/     # Quality gate failed data (on hold)
```

### 8.3 Ingestion Pipeline

- **PDF 7 stages**: Text extraction → OCR fallback → Table extraction → Image extraction → Section recognition → Metadata attachment → Common pipeline
- **Web 8 stages**: URL enumeration → robots.txt check → HTTP retrieval → Body extraction → Noise removal → Language detection → Metadata attachment → Common pipeline
- **Common 6 stages**: Normalization → Quality gate → Chunk splitting (default 1000 tokens / 200 overlap) → Embedding (Ollama) → Upsert (Chroma/Qdrant) → Log
- **Quality gate 6 rules**: Minimum/maximum character count, language detection, HTML noise removal, duplication (cosine >= 0.95), UTF-8 validity, OCR confidence

### 8.4 Incremental Updates and Operations

- Change detection via ETag / SHA-256 → incremental updates (180 minutes full rebuild → equivalent to 3 minutes)
- License management: OSS / free allowed, `vendor-proprietary` (`terms_accepted=true` required), `CC-BY-*` (attribution required), `unknown` rejected
- PII masking: originals stored encrypted with GPG, index contains masked text only
- Cache retention: PDF unlimited / Web 90 days / quarantine 30 days

### 8.5 RPC / CLI / Metrics

- Primary RPCs: `rag.ingest` (source_type/file_path/url/source_id/all/force/incremental), `rag.search` (query/top_k/filter/trace_id), `rag.cleanup`, `rag.status`
- Self-learning RPCs: `rag.ingest_work.v1`, `rag.search_similar.v1`, `rag.search_bugfix.v1`, `rag.search_design.v1`
- `IngestJobStatus`: `Queued` / `Processing` / `Completed` / `Failed` / `PartiallyCompleted`
- TypeScript I/F: `RagQuery { text, top_k, filter, trace_id }` / `RagResult { chunks, citations, embedding_time_ms, retrieval_time_ms }`
- MCP tool: `hestia_rag_search`
- CLI: `hestia rag ingest|search|cleanup`
- Prometheus metrics: `ingest_duration`, `docs_total`, `chunks_total`, `quarantine_total`, `incremental_skipped`, `license_violations`, `cache_size`, `retrieval_seconds`, `hit_ratio`

### 8.6 Sub-agent Configuration

| Sub-agent | Peer name | Role | Multiplicity |
|----------------|---------|------|-------|
| **planner** | `rag-planner` | Ingestion planning (crawl strategy, source priority, incremental update schedule) | 1 |
| **designer** | `rag-designer` | Knowledge base specification (chunk strategy, metadata schema, embedding model selection, retention policy) | 1 |
| **ingest** | `rag-ingest-{source}` | Document ingestion (PDF 7-stage / Web 8-stage pipeline) | **N** (launched in parallel per source) |
| **search** | `rag-search` | Vector search + reranking | 1 (N under high load) |
| **quality_gate** | `rag-quality` | Quality check (PII detection / license determination / deduplication / quarantine management) | 1 |
| **archivist** | `rag-archivist` | Self-learning accumulation pipeline management for conductor-work-logs/ | 1 (N under high load) |

---

## Related Documentation

- [Architecture Overview](architecture_overview.md) — Position of the shared services layer in the overall architecture
- [Security](security.md) — API key protection and license management
- [Container Execution](container_execution.md) — Container build Observability integration
- [Hestia Flow](hestia_flow.md) — RAG concept (§1.3.9)
- `.hestia/doc/common/observability.md` — Observability detailed specification
- `.hestia/doc/common/hdl_lsp_broker.md` — HDL LSP Broker detailed specification
- `.hestia/doc/common/constraint_bridge.md` — Constraint Bridge detailed specification
- `.hestia/doc/common/ip_manager.md` — IP Manager detailed specification
- `.hestia/doc/common/cicd_api.md` — CI/CD API detailed specification
- `.hestia/doc/common/wasm_waveform_viewer.md` — WASM Waveform Viewer detailed specification