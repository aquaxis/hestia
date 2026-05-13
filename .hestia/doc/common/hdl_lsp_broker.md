# HDL LSP Broker

**Domain**: common — HDL Development Support
**Source**: Design Specification §13.1

## Overview

An LSP proxy that provides unified access to Verilog / SystemVerilog / VHDL / Verilog-AMS LSP servers. Frontends (VSCode extension / Tauri IDE) can use a single connection for completion, diagnostics, go-to-definition, references, and rename across multiple languages. Provided as agent-cli peer `lsp`.

## Supported LSP Servers

| LSP Server | Version | Supported Languages |
|-----------|----------|---------|
| svls | v0.2.x | SystemVerilog |
| vhdl_ls | v0.3.x | VHDL |
| verilog-ams-ls | v0.1.x | Verilog-AMS |

## Key Types

### HdlLanguage

```rust
pub enum HdlLanguage {
    Verilog,
    SystemVerilog,
    Vhdl,
    VerilogAms,
}
```

### LspServerConfig

Startup configuration for each LSP server.

### RoutingTable

Routing table mapping file extensions to LSP servers.

## Extension Map

| Extension | Language | Routing Target |
|--------|------|-------------|
| `.v` | Verilog | svls (Verilog mode)|
| `.sv` / `.svh` | SystemVerilog | svls |
| `.vhd` / `.vhdl` | VHDL | vhdl_ls |
| `.va` / `.vams` | Verilog-AMS | verilog-ams-ls |

## Default Parameters

| Parameter | Default | Description |
|----------|-------|------|
| `max_instances` | 4 | Maximum number of simultaneously running LSP server instances |
| `idle_timeout_secs` | 300 | Auto-shutdown timeout when idle |

## Operation Flow

```
[VSCode / Tauri IDE] -> Single LSP connection -> HDL LSP Broker
                                              |
                                              +-- Extension detection -> HdlLanguage
                                              +-- RoutingTable selects LSP server
                                              +-- Start server if not running (within max_instances limit)
                                              +-- Forward LSP request -> Return response
```

## Integration Targets

- VSCode extension: Integrates HDL highlighting, completion, and diagnostics into Monaco Editor (§16.1)
- Tauri IDE: Provides the same editor functionality

## Related Documents

- [wasm_waveform_viewer.md](wasm_waveform_viewer.md) — WASM Waveform Viewer
- [constraint_bridge.md](constraint_bridge.md) — Constraint file conversion
- [observability.md](observability.md) — Monitoring