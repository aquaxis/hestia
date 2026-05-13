# MCP Server Specification

**Domain**: common — AI Tool Integration
**Source**: Design Specification §17.2, §13.7.5, §18.9

## Overview

The MCP (Model Context Protocol) server provides a standardized interface for invoking external tools from LLMs. In HESTIA, it is implemented as a shared service layer peer `mcp`, enabling AI agents (agent-cli processes) to call various tools via LLM Tool Use functionality.

## Architecture

```
[agent-cli (LLM)] -> Tool Use request -> [MCP Server] -> Tool execution -> Result returned
                                              |
                                              +-- HDL LSP Broker
                                              +-- Constraint Bridge
                                              +-- IP Manager
                                              +-- CI/CD API
                                              +-- Observability
                                              +-- RAG (hestia_rag_search)
                                              +-- kicad-mcp-python
```

## Provided Tools

| Tool Name | Target Service | Function |
|---------|------------|------|
| `hestia_rag_search` | rag-conductor | Knowledge base search (equivalent to `rag.search`)|
| `hestia_lsp_diagnostics` | HDL LSP Broker | HDL diagnostic information retrieval |
| `hestia_constraint_convert` | Constraint Bridge | Constraint file conversion |
| `hestia_ip_resolve` | IP Manager | IP dependency resolution |
| `hestia_pipeline_run` | CI/CD API | Pipeline execution |
| `hestia_health_check` | Observability | Health check execution |

## Difference Between MCP and agent-cli Backend Switching

| Item | MCP Server | agent-cli Backend Switching |
|------|------------|-------------------------|
| Purpose | External tool invocation from AI | Selection of agent-cli's own LLM backend |
| Path | Tool Use -> MCP Server -> Tool | agent-cli -> LLM API |
| Design | Independent | §20 `[agent_cli]` |

The two are independent designs: the MCP server handles "the path for AI to call tools," while backend switching handles "the selection of AI's own inference engine."

## Implementation Crate

```
hestia-mcp-server/
├── Cargo.toml
└── src/
    ├── lib.rs          # MCP server entry point
    ├── tools.rs        # Tool definition and dispatch
    └── transport.rs    # MCP protocol handling
```

## kicad-mcp-python Integration

KiCad integration uses `kicad-mcp-python` (MIT license). The PCB conductor exposes KiCad operations to AI via MCP.

## Related Documents

- [backend_switching.md](backend_switching.md) — LLM backend switching
- [agent_cli_messaging.md](agent_cli_messaging.md) — Messaging specification
- [observability.md](observability.md) — Monitoring