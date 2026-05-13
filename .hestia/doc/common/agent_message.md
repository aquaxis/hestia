# agent-cli Message Specification

**Domain**: common — Messaging
**Source**: Design Specification §14.1

## Overview

All HESTIA communications are unified under agent-cli native IPC. This chapter details the specification for structured messages (JSON payloads) sent and received over agent-cli IPC. Natural language payloads are handled directly by agent-cli's persona / LLM and are out of scope.

## Transport and Framing

- **Transport**: agent-cli native IPC (Unix Domain Socket under `$XDG_RUNTIME_DIR/agent-cli/`, managed automatically by agent-cli)
- **Permissions**: Registry directory `0700` (owner only), each peer socket `0600`
- **Framing**: agent-cli native frame (length-delimited, body max 16 MiB)
- **Connection**: Peer discovery via `agent-cli list`, sending via `agent-cli send <peer> <payload>` or REPL `/send <peer> <payload>`
- **Payload dispatch**: If the payload starts with `{`, interpret as structured JSON; otherwise, interpret as natural language text (§2.3)

## Structured Payload Format

### Request

```json
{
  "method": "fpga.build.v1.synthesize",
  "params": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Success Response

```json
{
  "result": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Error Response

```json
{
  "error": { "code": -32200, "message": "...", "data": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Notification (no id, no response)

```json
{
  "method": "agent.status_update",
  "params": { ... },
  "trace_id": "trace_xyz789"
}
```

### Batch (responses in same order)

```json
[
  { "method":"...", "params":{}, "id":"msg_1" },
  { "method":"...", "params":{}, "id":"msg_2" }
]
```

## ID Conventions

- `id` format: `msg_{ISO8601 timestamp}_{random}` (consistent with agent-cli conventions)
- `trace_id`: Cross-workflow trace ID (chained with §19 observability practices)
- The legacy JSON-RPC 2.0 `"jsonrpc": "2.0"` field is unnecessary (agent-cli IPC itself defines the transport)

## Error Response data Field

The error response `data` must include the following:

| Field | Description |
|---------|------|
| `tool` | Originating tool name |
| `exit_code` | Process exit code |
| `log_path` | Log file path |
| `errors[]` | Error detail array |
| `retry_possible` | Whether retry is possible |
| `suggested_action` | Recommended action |

## Payload Selection Guidelines

| Communication Type | Recommended Payload | Reason |
|---------|-------------|------|
| Structured operations (build / test / query / status) | Structured JSON | Type-safe, error code conventions, SDK-compatible |
| Inter-conductor structured tool calls | Structured JSON | Reproducibility, trace ID chaining |
| Inter-conductor natural language collaboration | Natural language text | Free-form, CoT context sharing |
| Event notifications | Structured JSON (no id) | Subscribable / filterable |

## Related Documents

- [api_versioning.md](api_versioning.md) — Method namespace and versioning
- [error_registry.md](error_registry.md) — Error code complete listing
- [agent_cli_messaging.md](agent_cli_messaging.md) — Complete messaging specification
- [observability.md](observability.md) — trace_id chaining