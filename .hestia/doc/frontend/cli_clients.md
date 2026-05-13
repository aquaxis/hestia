# CLI Execution

**Target Domain**: frontend — CLI clients
**Source**: Design Specification §15

## Overview

HESTIA provides 10 Rust CLI binaries: the unified runner `hestia` and 9 individual CLIs. Full workflows can be executed without the frontend (VSCode / Tauri IDE).

## CLI Structure

### Unified Runner

| Binary | Main Subcommands |
|--------|-----------------|
| `hestia` | `init` / `start [domain]` / `status` / `ai` / `rtl` / `fpga` / `asic` / `pcb` / `hal` / `apps` / `debug` / `rag` / `spec` |

### Individual CLIs (9 types)

| Binary | Main Subcommands |
|--------|-----------------|
| `hestia-ai-cli` | `exec` / `run --file` / `agent ls` / `container ls|start|stop|create` / `workflow run` / `review start` |
| `hestia-rtl-cli` | `init` / `lint` / `simulate` / `formal` / `transpile` / `handoff` / `status` |
| `hestia-fpga-cli` | `init` / `build` / `synthesize` / `implement` / `bitstream` / `simulate` / `program` / `report timing|resource` / `status` |
| `hestia-asic-cli` | `init` / `build` / `pdk install|list` / `advance` / `drc` / `lvs` / `status` |
| `hestia-pcb-cli` | `init` / `build` / `ai-synthesize` / `output kicad|gerber|bom` / `drc` / `erc` / `status` |
| `hestia-hal-cli` | `init` / `parse` / `validate` / `generate c|rust|python|svd` / `export-rtl` / `diff` / `status` |
| `hestia-apps-cli` | `init` / `build` / `flash` / `test sil|hil|qemu` / `size` / `debug` / `status` |
| `hestia-debug-cli` | `create` / `connect` / `disconnect` / `program` / `capture start|stop` / `signals read` / `trigger set` / `reset` / `status` |
| `hestia-rag-cli` | `ingest` / `search` / `cleanup` / `status` |

## Common Options (CommonOpts)

| Option | Description |
|--------|-------------|
| `--output (human|json)` | Output format |
| `--timeout` | Timeout |
| `--registry` | agent-cli registry (`$XDG_RUNTIME_DIR/agent-cli/`) |
| `--config` | Configuration file path |
| `--verbose` | Verbose output |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## CLI Architecture

Each CLI is a Rust client binary (`tokio` + `serde` + `clap`) that connects to the corresponding conductor's agent-cli peer via agent-cli native IPC. The shared implementation conforms to `conductor-sdk::transport` and the `AgentCliClient` specification.

## Usage Examples

```bash
# Unified runner
hestia init
hestia start fpga
hestia status

# RTL
hestia rtl init
hestia rtl lint
hestia rtl simulate --tb tb_alu --simulator verilator

# FPGA
hestia fpga build artix7
hestia fpga report timing

# ASIC
hestia asic pdk install sky130A
hestia asic build --pdk sky130A

# PCB
hestia pcb build --board-name "Sensor Board"
hestia pcb output gerber --output-dir ./gb

# HAL
hestia hal parse regs/soc.rdl
hestia hal generate c --output-dir build/hal/inc

# Apps
hestia apps build --target thumbv7em-none-eabihf
hestia apps test sil

# Debug
hestia debug create STM32F407 --interface-type swd
hestia debug capture start --session-id 1 --duration-ms 1000

# RAG
hestia rag ingest --source-id stm32_datasheet
hestia rag search "STM32F103 SPI pinout" --top-k 5

# AI
hestia ai exec "Create a UART LED control circuit on Artix-7"
hestia ai workflow run --workflow fpga-to-pcb-test-board
```

## Related Documentation

- [agent_cli_client.md](agent_cli_client.md) — agent-cli client specification
- [cargo_workspace.md](../common/cargo_workspace.md) — Workspace structure
- [installation.md](../common/installation.md) — Build instructions