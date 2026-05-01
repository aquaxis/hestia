# Hestia

## Hardware Engineering Stack for Tool Integration and Automation

**English** | [日本語](./README.md)

Hestia is an integrated hardware development environment that orchestrates FPGA, ASIC, PCB, HAL, and embedded software development tools through a unified AI-powered architecture.

## Features

- **9 Conductor Architecture** — Domain-specific AI agents for RTL, FPGA, ASIC, PCB, HAL, Apps, Debug, and RAG
- **Unified IPC** — All communication via agent-cli native IPC (`agent-cli send <peer> <payload>`)
- **Spec-Driven Development** — Generate HDL, constraints, and testbenches from natural language specifications
- **Vendor Abstraction** — Unified `ToolAdapter`/`VendorAdapter` trait layer; add tools via `adapter.toml` with zero code changes
- **Container & Local Execution** — Podman rootless containers or local execution; reproducible builds via lock files
- **AI Agent Pipeline** — WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent for automated tool maintenance

## Architecture

```text
                    ┌─────────────────────────────────────┐
                    │          hestia (CLI runner)          │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │         ai-conductor (meta)           │
                    │  ConductorManager │ WorkflowEngine   │
                    │  SpecDriven      │ SkillSystem       │
                    │  ContainerMgr    │ UpgradeManager    │
                    └──────────────┬──────────────────────┘
                                   │ agent-cli IPC
          ┌────────┬────────┬─────┼─────┬────────┬────────┐
          │        │        │     │     │        │        │
   ┌──────▼──┐ ┌──▼───┐ ┌─▼──┐ ┌▼──┐ ┌▼──────┐ ┌▼──────┐
   │  rtl    │ │ fpga │ │asic│ │pcb│ │  hal   │ │ apps  │
   │  cond.  │ │ cond.│ │cond│ │ c.│ │  cond. │ │ cond. │
   └─────────┘ └──────┘ └────┘ └───┘ └────────┘ └───────┘
   ┌────────┐ ┌──────┐                                      Frontend
   │ debug  │ │ rag  │    Shared Services                   ┌──────────┐
   │ cond.  │ │ cond.│    hdl-lsp-broker  waveform-core      │  VSCode   │
   └────────┘ └──────┘    constraint-bridge  ip-manager      │  hestia-  │
                              cicd-api  observability         │  ui       │
                              hestia-mcp-server               │ Tauri IDE │
                                                               └──────────┘
```

## Quick Start

### One-line install

```bash
curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh
```

Install to a custom prefix:

```bash
curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh -s -- --prefix ~/.local/bin
```

### Build from source

```bash
git clone https://github.com/AQUAXIS/hestia.git
cd hestia/.hestia/tools
make build
make install PREFIX=~/.local/bin
```

### Initialize a project

```bash
hestia init          # Create .hestia/ directory structure
hestia start         # Start all conductor daemons
hestia status        # Show daemon status
```

## Requirements

- **Rust** 1.75+ (install via [rustup](https://rustup.rs))
- **Linux** x86_64 (kernel 5.x+)
- **agent-cli** (for IPC communication between conductors)

## Workspace Structure

```text
.hestia/tools/
├── Cargo.toml                  # Workspace root (resolver = "2")
├── conductors/                 # 9 conductor daemons
│   ├── hestia-ai-conductor/
│   ├── hestia-rtl-conductor/
│   ├── hestia-fpga-conductor/
│   ├── hestia-asic-conductor/
│   ├── hestia-pcb-conductor/
│   ├── hestia-hal-conductor/
│   ├── hestia-apps-conductor/
│   ├── hestia-debug-conductor/
│   └── hestia-rag-conductor/
├── clis/                       # 10 CLI binaries
│   ├── hestia/                 # Unified runner
│   └── hestia-{domain}-cli/   # Domain-specific CLIs
├── crates/                     # Shared crates
│   ├── conductor-sdk/          # Transport / message / agent / config / error
│   ├── adapter-core/           # ToolAdapter / VendorAdapter traits
│   ├── project-model/          # TOML parser / config models
│   ├── hdl-lsp-broker/        # HDL LSP proxy (svls / vhdl_ls / verilog-ams-ls)
│   ├── waveform-core/         # VCD / FST / GHW / EVCD parser (WASM + native)
│   ├── constraint-bridge/      # XDC / SDC / PCF / Efinity XML / QSF / UCF converter
│   ├── ip-manager/             # IP core registry with DAG dependency resolution
│   ├── cicd-api/               # CI/CD pipeline abstraction (GitHub / GitLab / Local)
│   ├── observability/          # Prometheus + tracing + OTLP
│   └── hestia-mcp-server/      # MCP server for LLM tool use
└── packages/                   # Frontend
    ├── hestia-ui/               # React component library
    ├── hestia-vscode/           # VSCode extension
    └── hestia-ide/              # Tauri desktop IDE
```

## Conductors

| Conductor | Domain | Description |
| ---------- | ------ | ----------- |
| **ai** | Meta-orchestration | Manages all conductors, AI agents, containers, workflows |
| **rtl** | RTL design | Lint, simulate, formal verification, transpile, handoff |
| **fpga** | FPGA | Vivado / Quartus / Efinity build, synthesis, bitstream |
| **asic** | ASIC | OpenLane / Yosys / OpenROAD, PDK management |
| **pcb** | PCB | KiCad schematic & layout, AI synthesis, DRC/ERC |
| **hal** | Hardware Abstraction | Register maps, codegen (C/Rust/Python/SVD), bus protocols |
| **apps** | Embedded software | Toolchain, RTOS, HIL/SIL, flash & debug |
| **debug** | Debug | JTAG / SWD / ILA, waveform capture, protocol analysis |
| **rag** | Knowledge retrieval | Vector search, embedding, citation, 6 sub-agents |

## CLI Usage

```bash
# Unified runner
hestia init                    # Initialize project
hestia start fpga              # Start FPGA conductor
hestia status                  # Show all conductor status
hestia ai -- exec "review"     # Dispatch to ai-cli

# Domain-specific CLIs
hestia-fpga-cli init           # Initialize FPGA project
hestia-fpga-cli build          # Build FPGA project
hestia-rtl-cli lint            # Lint RTL sources
hestia-asic-cli pdk install   # Install PDK
hestia-pcb-cli drc             # Run DRC
hestia-hal-cli generate        # Generate HAL code
hestia-apps-cli flash          # Flash firmware
hestia-debug-cli capture       # Capture waveforms
hestia-rag-cli search "FIFO"  # Search knowledge base
```

## Build Targets

```bash
make build          # Release build (all 19 binaries)
make test           # Run all tests
make lint           # Run clippy
make fmt            # Check formatting
make install        # Install to /usr/local/bin
make install PREFIX=~/.local/bin  # Custom prefix
make clean          # Clean build artifacts
```

## License

MIT OR Apache-2.0
