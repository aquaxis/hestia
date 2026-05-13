# Database Schema

**Domain**: common — Data Persistence
**Source**: Design Specification §18.9

## Overview

HESTIA uses two types of data stores: SQLite for relational structured data and sled for high-throughput KV data.

## SQLite Schema

Purpose: Persistence of structured data. Lightweight embedded DB.

### compat-matrix

Tool version compatibility matrix.

```sql
CREATE TABLE compat_matrix (
    id          INTEGER PRIMARY KEY,
    tool_name   TEXT NOT NULL,
    version     TEXT NOT NULL,
    target      TEXT NOT NULL,
    compatible  BOOLEAN NOT NULL,
    tested_at   TEXT NOT NULL,
    notes       TEXT
);
```

### spec_history

Specification change history.

```sql
CREATE TABLE spec_history (
    id          INTEGER PRIMARY KEY,
    spec_path   TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    author      TEXT,
    changed_at  TEXT NOT NULL,
    change_type TEXT NOT NULL  -- 'created' | 'updated' | 'reviewed'
);
```

### work_log

Work log metadata.

```sql
CREATE TABLE work_log (
    id          INTEGER PRIMARY KEY,
    conductor   TEXT NOT NULL,
    task_id     TEXT NOT NULL,
    category    TEXT NOT NULL,  -- 'design_case' | 'bugfix_case' | 'build_log' | ...
    outcome     TEXT NOT NULL,  -- 'success' | 'failure' | 'partial'
    duration_secs INTEGER,
    created_at  TEXT NOT NULL
);
```

### ip_registry

IP core registration information.

```sql
CREATE TABLE ip_registry (
    id          TEXT PRIMARY KEY,  -- 'com.vendor.name'
    version     TEXT NOT NULL,
    vendor      TEXT NOT NULL,
    license     TEXT NOT NULL,     -- 'Oss' | 'VendorProprietary' | 'Unknown'
    device_families TEXT,          -- JSON array
    updated_at  TEXT NOT NULL
);
```

### container_images

Container image management.

```sql
CREATE TABLE container_images (
    id          INTEGER PRIMARY KEY,
    image_name  TEXT NOT NULL,
    tag         TEXT NOT NULL,
    digest      TEXT,
    size_bytes  INTEGER,
    signed      BOOLEAN DEFAULT 0,
    built_at    TEXT NOT NULL
);
```

### prompt-archive/index.db

Prompt archive index.

```sql
CREATE TABLE prompts (
    prompt_id    TEXT PRIMARY KEY,
    trace_id     TEXT NOT NULL,
    agent_id     TEXT NOT NULL,
    timestamp    TEXT NOT NULL,
    model_name   TEXT NOT NULL,
    template_id  TEXT,
    status       TEXT NOT NULL,
    tokens_input  INTEGER,
    tokens_output INTEGER,
    latency_ms   INTEGER,
    file_path    TEXT NOT NULL
);
CREATE INDEX idx_prompt_trace ON prompts(trace_id);
CREATE INDEX idx_prompt_template ON prompts(template_id);
```

## sled Schema

Purpose: High-throughput KV store. zstd compression, 1 GiB cache.

| KV Collection | Key | Value | Purpose |
|----------------|------|----|------|
| `messages` | `trace_id` | JSON | Message history |
| `agent_state` | `agent_id` | JSON | Agent state snapshot |
| `task_queue` | `task_id` | JSON | Task queue |
| `rag_cache` | `query_hash` | JSON | RAG query cache |
| `version_matrix` | `tool@version` | JSON | Version compatibility information |
| `workflow_state` | `workflow_id` | JSON | Workflow execution state |

## Related Documents

- [observability.md](observability.md) — Monitoring and metrics
- [ip_manager.md](ip_manager.md) — IP Manager (uses ip_registry)
- [cicd_api.md](cicd_api.md) — CI/CD API (uses work_log)