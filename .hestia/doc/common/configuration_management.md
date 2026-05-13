# Configuration File Management

**Domain**: common — Configuration Management
**Source**: Design Specification §18.9, §13.7.6, §19.3.5

## Overview

HESTIA monitors various configuration files (`config.toml` / `fpga.toml` / `container.toml` / `sources.toml`, etc.) using `inotify` and performs hot reload upon change detection. This allows configuration changes to be reflected without restarting conductors.

## inotify Change Detection

### Monitored Paths

| Path | Monitored Content |
|------|---------|
| `.hestia/config.toml` | Global settings (agent_cli / health, etc.)|
| `.hestia/<conductor>/*.toml` | Conductor-specific settings |
| `.hestia/rag/sources.toml` | RAG source declarations |
| `.hestia/rag/sources/` | RAG source files (PDF / Web)|

### Detection Flow

```
[inotify watch] -> File change detected -> SHA-256 hash comparison -> Diff exists ->
  +-- Configuration reload (HestiaConfig::from_toml_file)
  +-- Change diff recorded in structured log
  +-- Configuration update notification to affected conductors
```

## Hot Reload

### Reloadable Configuration Items

| Item | Reloadable | Notes |
|------|------------|------|
| `[health] interval_secs` | Yes | Reflected from next health check cycle |
| `[rag] top_k / chunk_size` | Yes | Reflected from next ingest |
| `[agent_cli] model / max_tokens` | Yes | Reflected from next agent-cli subprocess startup |
| `[agent_cli] backend` | Requires restart | Backend switching requires process restart |
| `[build]` target changes | Yes | No impact on running jobs |

### Non-reloadable Configuration Items

- `[agent_cli] backend` — Process restart required
- `[container]` — Container image rebuild required
- Port numbers / socket paths — Process restart required

## cron / inotify Scheduling

RAG source auto-updates trigger on:

1. **cron**: `0 3 * * *` (daily at 03:00 UTC, default)
2. **File change**: `inotify` / `fswatch` monitoring of `.hestia/rag/sources/`
3. **Manual**: `hestia rag ingest --source-id <id>`

## Implementation Crates

- `configuration_management` — inotify wrapper (`inotify` crate, LGPL)
- `project-model` — TOML parser and configuration model (`serde` + `toml`)

## Related Documents

- [config_common.md](config_common.md) — Common configuration sections
- [backend_switching.md](backend_switching.md) — LLM backend switching
- [observability.md](observability.md) — Monitoring and logging