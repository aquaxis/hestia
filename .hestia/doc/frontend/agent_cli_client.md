# agent-cli Client Specification

**Target Domain**: frontend — client communication
**Source**: Design Specification §16.4

## Overview

Common `AgentCliClient` specification for Rust / TypeScript. A shared interface used by VSCode extension, Tauri IDE, and CLI clients to communicate with conductors via agent-cli native IPC.

## Message Types

### AgentCliRequest

```typescript
interface AgentCliRequest {
  method: string;
  params?: Record<string, unknown>;
  id: string;
  trace_id: string;
}
```

### AgentCliResponse

```typescript
type AgentCliResponse =
  | AgentCliSuccessResponse
  | AgentCliErrorResponse;

interface AgentCliSuccessResponse {
  result: Record<string, unknown>;
  id: string;
  trace_id: string;
}

interface AgentCliErrorResponse {
  error: {
    code: number;
    message: string;
    data?: Record<string, unknown>;
  };
  id: string;
  trace_id: string;
}
```

### AgentCliNotification

```typescript
interface AgentCliNotification {
  method: string;
  params?: Record<string, unknown>;
  trace_id: string;
}
```

## HestiaClientConfig

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `agentCliRegistryDir` | string | `$XDG_RUNTIME_DIR/agent-cli/` | Registry directory |
| `requestTimeout` | number | 30000 | Request timeout (ms) |
| `reconnectInterval` | number | 5000 | Reconnection interval (ms) |
| `maxReconnectAttempts` | number | 10 | Maximum reconnection attempts |
| `logLevel` | string | `"info"` | Log level |
| `retryPolicy` | RetryPolicy | See below | Retry policy |
| `maxFrameLength` | number | 16777216 | Maximum frame length (16 MiB) |

## RetryPolicy

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `maxRetries` | number | 3 | Maximum retry count |
| `initialBackoffMs` | number | 1000 | Initial backoff (ms) |
| `maxBackoffMs` | number | 60000 | Maximum backoff (ms) |
| `multiplier` | number | 2.0 | Backoff multiplier |
| `retryableCodes` | number[] | [-32001, -32006] | Retryable error codes |

## ConnectionState

| State | Meaning |
|-------|---------|
| `disconnected` | Not connected |
| `connecting` | Connecting |
| `connected` | Connected |
| `reconnecting` | Reconnecting |
| `error` | Error state |

## Internal Implementation

- Peer discovery: Runs `agent-cli list` on startup
- Sending: Invokes `agent-cli send <peer> <payload>` via spawn or FFI
- Rust version: Implemented in `conductor-sdk::transport`
- TypeScript version: Implemented in VSCode extension / Tauri IDE

## Related Documentation

- [cli_clients.md](cli_clients.md) — CLI clients
- [vscode_extension.md](vscode_extension.md) — VSCode extension
- [agent_cli_messaging.md](../common/agent_cli_messaging.md) — Messaging specification