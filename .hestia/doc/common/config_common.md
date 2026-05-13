# Conductor Common Configuration Sections

**Domain**: common — Configuration
**Source**: Design Specification §3.8, §13.7.6, §20.2

## Overview

Each conductor's TOML configuration file (`fpga.toml` / `asic.toml` / `pcb.toml`, etc.) defines common sections. This document specifies the schema and semantics of these common sections.

## Common Sections

### `[project]`

Basic project information.

| Key | Type | Required | Description |
|------|----|------|------|
| `name` | string | Required | Project name |
| `version` | string | Required | Project version |
| `description` | string | Optional | Project description |

### `[adapters]`

Declaration of tool adapters to use.

| Key | Type | Required | Description |
|------|----|------|------|
| `active` | string[] | Required | List of adapter names to use |
| `search_paths` | string[] | Optional | adapter.toml search paths |

### `[build]`

Build settings.

| Key | Type | Required | Description |
|------|----|------|------|
| `targets` | table[] | Required | Build target definitions |
| `steps` | string[] | Optional | Build step order |
| `timeout_secs` | int | Optional | Build timeout (default 3600)|
| `max_parallel` | int | Optional | Maximum parallelism (default 4)|

### `[container]`

Container execution settings (only when container execution is selected).

| Key | Type | Required | Description |
|------|----|------|------|
| `name` | string | Required | Container name |
| `base_image` | string | Required | Base image |
| `conductor` | string | Required | Target conductor |

### `[health]`

Health check settings.

| Key | Type | Default | Description |
|------|----|------|------|
| `cmd` | string | — | Quick verification command in local execution mode |
| `interval_secs` | int | 30 | Polling interval |
| `timeout_secs` | int | 3 | Single response timeout |
| `max_retries` | int | 3 | Consecutive retry count on failure |
| `escalate_on_fail` | bool | true | Notify frontend on consecutive failures |
| `restart_on_fail` | bool | true | Automatic restart attempt |

### `[agent_cli]`

agent-cli backend settings (see §20).

| Key | Type | Default | Description |
|------|----|------|------|
| `backend` | string | `"claude"` | LLM backend type |
| `binary_path` | string | `""` | agent-cli binary path |
| `anthropic_base_url` | string | `""` | OpenAI-compatible API endpoint |
| `anthropic_api_key_env` | string | `"ANTHROPIC_API_KEY"` | API key environment variable name |
| `model` | string | `"claude-opus-4-7"` | LLM model name |
| `max_tokens` | int | 4096 | Response token limit |
| `registry_dir` | string | `""` | IPC registry directory |

### `[rag]`

RAG settings (for rag-conductor).

| Key | Type | Default | Description |
|------|----|------|------|
| `backend` | string | `"chroma"` | Vector DB backend |
| `embedding_model` | string | `"nomic-embed-text"` | Embedding model |
| `top_k` | int | 5 | Number of top search results |
| `chunk_size` | int | 1000 | Chunk size |
| `chunk_overlap` | int | 200 | Chunk overlap |
| `self_learning_enabled` | bool | true | Self-learning feature ON/OFF |

## TOML Parser

The common parser is implemented in the `project-model` crate and used by each conductor. It leverages `serde` deserialization with `#[serde(default)]` to make each key individually optional.

## Related Documents

- [configuration_management.md](configuration_management.md) — Configuration file management (hot reload)
- [backend_switching.md](backend_switching.md) — LLM backend switching
- [health_check_orchestration.md](health_check_orchestration.md) — Health checks