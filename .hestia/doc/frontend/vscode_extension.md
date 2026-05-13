# VSCode Extension

**Target Domain**: frontend — VSCode integration
**Source**: Design Specification §16.1

## Overview

Hestia's VSCode extension. Implemented in TypeScript, it provides Monaco Editor integration, HDL LSP, waveform viewer, and conductor management within VSCode.

## Package Information

| Item | Value |
|------|-------|
| Package name | `hestia-vscode` |
| Publisher | `aquaxis` |
| VSCode version | >= 1.85.0 |

## Activation

### onCommand Triggers

30+ commands are registered. Key commands include:

- `hestia.start` / `hestia.stop` / `hestia.status`
- `hestia.ai` / `hestia.spec` / `hestia.fpga` / `hestia.debug` / `hestia.rag`

### onView Triggers

- `hestia-conductor`
- `agents`
- `specs`

### onLanguage Triggers

- `verilog` / `vhdl` / `systemverilog` / `xdc` / `pcf` / `toml`

## Views (5 types)

| View | Purpose |
|------|---------|
| `ConductorStatusView` | Status list and control for all conductors |
| `AgentListView` | Sub-agent list and management |
| `SpecViewer` | Structured display and editing of specifications |
| `DesignFlowView` | Design flow (DAG) visualization |
| `LogViewer` | Real-time log streaming |

## Monaco Editor Integration

- HDL syntax highlighting (Verilog / SystemVerilog / VHDL)
- Code completion (via HDL LSP Broker, §13.1)
- Inline diagnostic display
- Go to Definition / Find References / Rename (LSP features)

## Waveform Viewer (WebView)

- WASM rendering inside VSCode WebView
- Uses `waveform-core` crate compiled to WASM (§13.2)
- Performance ensured via WebWorker + SharedArrayBuffer
- Supports VCD / FST / GHW / EVCD

## agent-cli IPC

- Uses `AgentCliClient` (TypeScript version) from `conductor-sdk`
- Invokes `agent-cli list` / `agent-cli send` via spawn or native bindings
- `ConductorId = 'ai' | 'rtl' | 'fpga' | 'asic' | 'pcb' | 'hal' | 'apps' | 'debug' | 'rag'`

## Configuration Schema

See `config_schema.md` for the full `hestia.*` settings list.

## Related Documentation

- [config_schema.md](config_schema.md) — VSCode configuration schema
- [agent_cli_client.md](agent_cli_client.md) — agent-cli client specification
- [ui_components.md](ui_components.md) — UI component library
- [hdl_lsp_broker.md](../common/hdl_lsp_broker.md) — HDL LSP Broker
- [wasm_waveform_viewer.md](../common/wasm_waveform_viewer.md) — WASM waveform viewer