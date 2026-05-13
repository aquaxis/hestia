# Observability

**Domain**: common — Monitoring and Observation
**Source**: Design Specification §13.6, §19.8

## Overview

Visualizes the state of all Hestia components through structured logs, metrics, and distributed tracing. Integrates the three pillars (Logs / Metrics / Traces) and aggregates LLM call statistics, agent states, and development process metrics. Provided as agent-cli peer `observability`.

## Three Pillars

| Pillar | Technology | Endpoint | Storage |
|----|------|-------------|-------|
| Logs | Structured JSON (`tracing` crate)| — | `.hestia/observability/logs/<YYYY-MM-DD>.jsonl` |
| Metrics | OpenMetrics format (`prometheus` crate)| `localhost:9090/metrics` | Prometheus scrape |
| Traces | OpenTelemetry (OTLP gRPC)| `:4317` | Local Tempo / Jaeger (optional)|

## Health Check Endpoints

| Endpoint | Purpose |
|-------------|------|
| `:8080/health` | Process liveness check |
| `:8080/ready` | Service readiness check |
| `:8080/live` | Liveness check |

### HealthStatus

| Value | Meaning |
|----|------|
| `Healthy` | Normal |
| `Degraded` | Partial feature restrictions |
| `Unhealthy` | Abnormal |

## Structured Log Common Fields

```json
{
  "timestamp": "2026-04-23T16:00:00.123Z",
  "level": "INFO",
  "trace_id": "01HE...",
  "span_id": "...",
  "component": "fpga-conductor",
  "event": "build.started",
  "target": "artix7_dev",
  "job_id": 42,
  "metadata": { ... }
}
```

## Key Metrics

### Common Metrics

| Metric Name | Type | Description |
|-------------|-----|------|
| `hestia_build_total{conductor,status}` | Counter | Build count (by success/failure)|
| `hestia_build_duration_seconds{conductor,step}` | Histogram | Duration per step |
| `hestia_agent_active{skill}` | Gauge | Active agent count by skill |
| `hestia_agent_pending_tasks{skill}` | Gauge | Queue length |

### LLM Metrics

| Metric Name | Type | Description |
|-------------|-----|------|
| `hestia_llm_requests_total{model,status}` | Counter | LLM call count |
| `hestia_llm_tokens_total{model,direction}` | Counter | Input/output token count |
| `hestia_llm_latency_seconds{model}` | Histogram | Latency distribution |

### RAG Metrics

| Metric Name | Type | Description |
|-------------|-----|------|
| `hestia_rag_retrieval_seconds` | Histogram | RAG retrieval time |
| `hestia_rag_hit_ratio` | Gauge | Knowledge base useful hit ratio |

### Container Metrics

| Metric Name | Type | Description |
|-------------|-----|------|
| `hestia_container_build_total{image,status}` | Counter | Build count |
| `hestia_container_build_duration_seconds{image,stage}` | Histogram | Duration per stage |
| `hestia_container_image_size_bytes{image,tag}` | Gauge | Image size |
| `hestia_container_vuln_total{image,severity}` | Gauge | Vulnerability count |
| `hestia_container_signature_verified{image}` | Gauge | Signature verification success |

### Feedback Loop Metrics

| Metric Name | Type | Description |
|-------------|-----|------|
| `hestia_feedback_loops_total{outcome}` | Counter | Feedback loop occurrence count |

## Aggregation by ConductorName

Metrics and health are aggregated per `ConductorName` (`Ai` / `Fpga` / `Asic` / `Pcb` / `Debug` / `Rag`).

## Development Process KPIs (Derived Metrics)

- Specification to implementation lead time
- Testbench-first rate (TDD compliance)
- CoT presence rate / average stage count
- Hallucination detection rate (RAG `out-of-reference` flag ratio)

## Dashboard

```bash
ai-cli observability dashboard --open
```

Main views: Build Health / Agent Fleet / LLM Spend / Feedback Loop / Knowledge Coverage

## Operational Rules

- All components output structured logs via `tracing`
- `trace_id` is shared across CoT / Action Log / Prompt Archive
- Metrics are scraped at 30-second intervals, retained for 90 days
- Anomaly detection: Warning when `hestia_build_duration_seconds` p99 exceeds 2x normal

## Related Documents

- [health_check_orchestration.md](health_check_orchestration.md) — Health checks
- [error_registry.md](error_registry.md) — Error codes
- [agent_cli_messaging.md](agent_cli_messaging.md) — Messaging