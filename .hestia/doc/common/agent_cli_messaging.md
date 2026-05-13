# agent-cli Complete Messaging Specification

**Domain**: common — Communication Infrastructure
**Source**: Design Specification §14, §2.3, §20 / Phase 113

## Overview

All HESTIA communications are unified under agent-cli compatible IPC. This document defines the complete specification for transport, framing, payload format, and dispatch logic.

**Phase 113 — Engine Switching Impact**: When `.hestia/config.toml` has `[engine] type = "claude_cli_shim"` selected, the peer-driven binary switches to `claude-cli-shim`, but subcommands (`run` / `list` / `send` / `providers` / `doctor`) and options remain fully compatible with agent-cli. The messaging conventions and JSONL log schema (`kind: thinking|tool_call|tool_result|user|assistant`) defined in this specification apply unchanged. For details, see [backend_switching.md](backend_switching.md) §"Engine (Phase 113)".

## Transport

- **Infrastructure**: agent-cli native IPC
- **Socket**: Unix Domain Socket under `$XDG_RUNTIME_DIR/agent-cli/` (managed automatically by agent-cli)
- **Permissions**: Registry directory `0700`, each peer socket `0600`
- **Protocol**: Length-delimited framing, body max 16 MiB

## Peer Model

### Conductor Peers

| Peer Name | Conductor | Role |
|---------|-----------|------|
| `ai` | ai-conductor | Meta-orchestrator |
| `rtl` | rtl-conductor | RTL design flow |
| `fpga` | fpga-conductor | FPGA design flow |
| `asic` | asic-conductor | ASIC design flow |
| `pcb` | pcb-conductor | PCB design flow |
| `hal` | hal-conductor | HAL generation |
| `apps` | apps-conductor | Application firmware |
| `debug` | debug-conductor | Debugging |
| `rag` | rag-conductor | Knowledge base |

### Shared Service Peers

| Peer Name | Service |
|---------|---------|
| `lsp` | HDL LSP Broker |
| `constraint-bridge` | Constraint Bridge |
| `ip-manager` | IP Manager |
| `cicd` | CI/CD API |
| `observability` | Observability |
| `waveform` | WASM Waveform Viewer |
| `mcp` | MCP Server |

### Frontend Peers

| Peer Name | Client |
|---------|------------|
| `vscode` | VSCode extension |
| `tauri` | Tauri desktop application |
| `cli` | CLI client (optional)|

## Payload Format

### Structured JSON Payload

If the payload starts with `{`, it is interpreted as structured JSON:

```json
{
  "method": "fpga.build.v1.synthesize",
  "params": { "target": "artix7" },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Natural Language Payload

If the payload does not start with `{`, it is interpreted as natural language text:

```
Please start a build with Vivado, target=artix7
```

### Dispatch Logic

```
Received payload
  |
  +-- Starts with '{' -> Interpret as structured JSON, convert to tool call
  |                      Dispatch according to method namespace convention (§14.2)
  |
  +-- Otherwise       -> Interpret as natural language text, pass directly to agent-cli's LLM
```

## Operation API

| API | Description |
|-----|------|
| `agent-cli list` | List active peers |
| `agent-cli send <peer> <payload>` | Send payload to specified peer |
| REPL: `/send <peer> <payload>` | Send from within REPL |

## ConductorRpc Common API

Common RPC trait implemented by all conductors:

| Method Group | Example Methods |
|----------|----------|
| Project management | `project_open` / `project_targets` / `project_files` |
| Build | `build_start` / `build_cancel` / `build_status` |
| Reports | `report_timing` / `report_resource` / `report_messages` |
| Programming | `program_targets` / `program_flash` |
| Toolchain | `toolchain_list` / `toolchain_install` / `toolchain_select` |
| Agent | `agent_status` / `agent_patch_list` / `agent_apply_patch` |
| Container | `container_list` / `container_start` / `container_stop` / `container_update` |
| System | `system_readiness` / `system_health` |

## Payload Selection Guidelines

| Communication Type | Recommended Payload | Reason |
|---------|-------------|------|
| Structured operations (build / test / status) | Structured JSON | Type-safe, SDK-compatible |
| Inter-conductor structured tool calls | Structured JSON | Reproducibility, trace ID chaining |
| Inter-conductor natural language collaboration | Natural language text | Free-form, CoT context sharing |
| Progress / CoT / thought process sharing | Natural language text | Lightweight propagation |
| Event notifications | Structured JSON (no id) | Subscribable / filterable |
| Error escalation | Natural language to ai-conductor -> Structured notification to UI | Context detail aggregation -> immediate reflection |

## Related Documents

- [agent_message.md](agent_message.md) — Payload format details
- [api_versioning.md](api_versioning.md) — Method namespace and versioning
- [error_registry.md](error_registry.md) — Error code conventions
- [backend_switching.md](backend_switching.md) — LLM backend switching