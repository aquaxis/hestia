# Rust Workspace Configuration

**Domain**: common — Build Configuration
**Source**: Design Specification §15.4, §1.6, §2.1

## Overview

The HESTIA Rust project is structured as a Cargo workspace rooted at `.hestia/tools/Cargo.toml`. It uses resolver = "2" and centrally manages 9 conductor daemons + 10 CLI binaries.

## Workspace Configuration

```
.hestia/tools/
├── Cargo.toml                    # Workspace root (resolver = "2")
├── conductors/                   # Rust daemons x9
│   ├── hestia-ai-conductor/
│   ├── hestia-rtl-conductor/
│   ├── hestia-fpga-conductor/
│   ├── hestia-asic-conductor/
│   ├── hestia-pcb-conductor/
│   ├── hestia-hal-conductor/
│   ├── hestia-apps-conductor/
│   ├── hestia-debug-conductor/
│   └── hestia-rag-conductor/
├── clis/                         # CLI x10
│   ├── hestia/                   # Unified runner
│   ├── hestia-ai-cli/
│   ├── hestia-rtl-cli/
│   ├── hestia-fpga-cli/
│   ├── hestia-asic-cli/
│   ├── hestia-pcb-cli/
│   ├── hestia-hal-cli/
│   ├── hestia-apps-cli/
│   ├── hestia-debug-cli/
│   └── hestia-rag-cli/
└── crates/                       # Shared crates
    ├── conductor-sdk/            # transport / message / agent / config / error
    ├── adapter-core/             # ToolAdapter / VendorAdapter traits
    ├── hestia-mcp-server/        # MCP server
    └── project-model/            # TOML parser and configuration model
```

## Binary Listing (19 binaries)

### Conductor Daemons (9)

| Binary | Corresponding conductor |
|---------|---------------|
| `hestia-ai-conductor` | ai-conductor |
| `hestia-rtl-conductor` | rtl-conductor |
| `hestia-fpga-conductor` | fpga-conductor |
| `hestia-asic-conductor` | asic-conductor |
| `hestia-pcb-conductor` | pcb-conductor |
| `hestia-hal-conductor` | hal-conductor |
| `hestia-apps-conductor` | apps-conductor |
| `hestia-debug-conductor` | debug-conductor |
| `hestia-rag-conductor` | rag-conductor |

### CLI Clients (10)

| Binary | Major Subcommands |
|---------|----------------|
| `hestia` | `init` / `start [domain]` / `status` / `ai` / `rtl` / `fpga` / `asic` / `pcb` / `hal` / `apps` / `debug` / `rag` |
| `hestia-ai-cli` | `exec` / `run --file` / `agent ls` / `container ls|start|stop|create` / `workflow run` |
| `hestia-rtl-cli` | `init` / `lint` / `simulate` / `formal` / `transpile` / `handoff` / `status` |
| `hestia-fpga-cli` | `init` / `build` / `synthesize` / `implement` / `bitstream` / `simulate` / `program` / `report` |
| `hestia-asic-cli` | `init` / `build` / `pdk install|list` / `advance` / `drc` / `lvs` / `status` |
| `hestia-pcb-cli` | `init` / `build` / `ai-synthesize` / `output` / `drc` / `erc` / `status` |
| `hestia-hal-cli` | `init` / `parse` / `validate` / `generate` / `export-rtl` / `diff` / `status` |
| `hestia-apps-cli` | `init` / `build` / `flash` / `test` / `size` / `debug` / `status` |
| `hestia-debug-cli` | `create` / `connect` / `disconnect` / `program` / `capture` / `signals` / `trigger` / `reset` |
| `hestia-rag-cli` | `ingest` / `search` / `cleanup` / `status` |

## Common Dependencies

| Crate | Purpose |
|---------|------|
| `tokio` | Async runtime (multi_thread, 4 workers)|
| `serde` | TOML/JSON serialization |
| `tracing` | Structured logging |
| `thiserror` / `anyhow` | Error handling (library / binary differentiation)|
| `clap` | CLI parser |
| `sled` | Rust-native KV store |
| `minijinja` | Template engine |
| `petgraph` | DAG resolution |

## Build Commands

```bash
cd .hestia/tools
cargo build --release                                    # All binaries
cargo build --release -p hestia-fpga-conductor           # Specific conductor
cargo test                                               # All tests
cargo test -p hestia-fpga-conductor                      # Specific conductor
cargo test -p container-manager                          # Specific crate
```

## Related Documents

- [installation.md](installation.md) — Build procedure details
- [error_handling_strategy.md](error_handling_strategy.md) — Error handling strategy
- [conductor_startup.md](conductor_startup.md) — Daemon startup order