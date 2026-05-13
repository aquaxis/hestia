# Architecture Overview

**Scope**: Hestia overall
**Source**: Design specification §1 (Design philosophy and 8 principles), §2 (5-layer + 9 Conductor architecture), §1.4 (Technology stack), §1.7 (Execution environment)

---

## 1. Design Philosophy and Fundamental Challenges

Hardware development (FPGA, ASIC, PCB) involves multiple vendor tools coexisting. For FPGAs alone, AMD Vivado, Intel Quartus Prime, Efinix Efinity, and Lattice Radiant each have their own project formats, constraint descriptions, and CLI interfaces. For ASICs, the OpenLane 2 / Yosys / OpenROAD / Magic OSS toolchain must be combined, and for PCBs, KiCad / SKiDL / Freerouting exist independently.

Projects dealing with these heterogeneous tools incur enormous context-switching costs. Furthermore, each tool has 1-2 major releases per year, and each version upgrade requires modifications to scripts, log parsers, and constraint formats. Hestia solves these challenges through an AI-powered integrated environment.

---

## 2. Design Principles (8 Principles)

### Principle 1: Abstraction, Not Replacement

Vendor tools are certified chains for FPGA bitstream generation and ASIC GDSII output — complete replacement is impossible. We build "a layer that orchestrates via a unified interface." Each tool's unique functionality is abstracted through VendorAdapter / ToolAdapter traits, making it operable via the same API from upper layers.

### Principle 2: Zero-Modification Extension

Adding a new vendor tool requires no changes to core code whatsoever. An adapter can be added simply by writing `adapter.toml`. As a Script adapter strategy, commands, log parsing rules, and report extraction rules are defined as regular expressions within the TOML file.

### Principle 3: Sustainable Maintenance

AI agents (WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent) automate the response to tool version upgrades, minimizing human maintenance costs.

### Principle 4: Security

Tool execution can be selected as either container execution or local execution. Container execution uses Podman rootless for unprivileged execution, `--network=none` for network isolation, and `--security-opt=no-new-privileges` to prevent privilege escalation.

### Principle 5: Reproducibility

Complete build reproducibility is guaranteed by fpga.lock / asic.lock. Container execution achieves this through container image hash pinning; local execution through lock recording of tool versions, execution paths, and environment variables.

### Principle 6: Vendor Independence

OSS tools are prioritized, and any vendor tool can be integrated via the plugin system. Lock-in to specific vendors is eliminated.

### Principle 7: AI Utilization

Leverages spec-driven development and generative AI's Tool Use functionality to support the entire design process with AI.

### Principle 8: Unified Interface

All communication between conductors and between the frontend ↔ ai-conductor is unified under agent-cli compatible IPC. Each conductor itself is an AI agent launched as an agent-cli compatible engine (`agent-cli` or the `claude-cli-shim` wrapper added in Phase 113) process, and the frontend also joins as an agent-cli peer. The engine can be switched in the `[engine]` section of `.hestia/config.toml` (see [backend_switching.md](common/backend_switching.md)).

---

## 3. 5-Layer + 9 Conductor Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Frontend Layer                                │
│    VSCode Extension (TypeScript)  /  Tauri IDE (Rust + React)       │
│    CLI: hestia + {ai,rtl,fpga,asic,pcb,hal,apps,debug,rag}-cli │
└─────────────────────────┬───────────────────────────────────────────┘
                          │ agent-cli IPC (peer "ai")
┌─────────────────────────▼───────────────────────────────────────────┐
│                   Meta-Orchestration Layer                           │
│    ai-conductor (orchestrates all conductors + sustainable upgrade) │
│    ┌──────────────────────────────────────────────────────────────┐ │
│    │  container-manager (container lifecycle management for all)   │ │
│    └──────────────────────────────────────────────────────────────┘ │
└───┬──────────┬──────────┬──────────┬──────────┬────────────────────┘
    │          │          │          │          │  agent-cli IPC (per peer)
┌───▼───┐  ┌──▼───┐  ┌──▼───┐  ┌──▼───┐  ┌──▼───┐
│ fpga  │  │ asic │  │ pcb  │  │debug │  │ rag  │   Conductor Layer
│ cond. │  │ cond.│  │ cond.│  │cond. │  │cond. │   (domain-specific orchestrators)
│       │  │      │  │      │  │      │  │      │
│Vivado │  │Open  │  │KiCad │  │JTAG  │  │Chroma│
│Quartus│  │Lane 2│  │SKiDL │  │SWD   │  │Qdrant│
│Efinity│  │Yosys │  │AI    │  │ILA   │  │Ollama│
│nextpnr│  │Open  │  │design│  │Signal│  │Embed │
│Radiant│  │ROAD  │  │Free- │  │Tap   │  │ ing  │
│Gowin  │  │Magic │  │routing│  │sigrok│  │      │
└───┬───┘  └──┬───┘  └──┬───┘  └──┬───┘  └──┬───┘
    │         │         │         │         │
┌───▼─────────▼─────────▼─────────▼─────────▼──────────────────────────┐
│         Tool Execution Layer (container [Podman rootless] / local)   │
│    fpga/vivado:2025.2  │  asic/openlane:latest  │  pcb/kicad:latest   │
│    fpga/quartus:25.1   │  asic/magic:latest     │  debug/tools:latest │
│    fpga/efinity:2025.2 │  fpga/oss:latest       │  (debug: local only)│
│    fpga/radiant:2024.2 │                        │                     │
└─────────────────────────┬───────────────────────────────────────────-┘
                          │
┌─────────────────────────▼───────────────────────────────────────────┐
│                      Shared Services Layer (Layer 5)                 │
│    HDL LSP Broker (svls/vhdl_ls/verilog-ams-ls)                    │
│    WASM Waveform Viewer (VCD/FST/GHW/EVCD)                         │
│    Constraint Bridge (XDC ⇔ SDC ⇔ PCF ⇔ Efinity XML)              │
│    IP Manager (OSS / VendorProprietary)                             │
│    CI/CD API (GitHub Actions / GitLab CI / Local)                  │
│    Observability (Prometheus + tracing + OpenTelemetry)             │
└─────────────────────────────────────────────────────────────────────┘
```

### 5-Layer Responsibilities

| Layer | Responsibility | Key Components |
|----|------|------------------|
| Frontend Layer | User interaction, editor integration, CLI experience | VSCode Extension / Tauri IDE / `hestia` unified CLI / individual conductor CLIs |
| Meta-Orchestration Layer | Orchestrates all conductors, container lifecycle management | ai-conductor / container-manager |
| Conductor Layer | Domain-specific orchestration, tool abstraction, build state machines | rtl / fpga / asic / pcb / hal / apps / debug / rag conductors |
| Tool Execution Layer | Vendor tool execution (container or local), reproducibility guarantee, security boundary | 8 Podman rootless container images + local installation support |
| Shared Services Layer | 6 cross-cutting services (LSP / Waveform / Constraint conversion / IP / CI/CD / Observability) | HDL LSP Broker / WASM Waveform Viewer / Constraint Bridge / IP Manager / CI/CD API / Observability |

---

## 4. 9 Conductor Roles

| Conductor | Role | Target Tools / Functions | Execution Mode |
|-----------|------|------------------|-----------|
| ai-conductor | Meta-orchestrator (orchestrates all conductors / sole entry point for humans) | Skill / Workflow / Spec-Driven / Backend Switching | Container + Local |
| rtl-conductor | RTL design flow orchestration (HDL Lint / Sim / Formal / Transpile) | Verilator, Verible, iverilog, GHDL, SymbiYosys, cocotb, Chisel/SpinalHSD/Amaranth bridges | Container + Local |
| fpga-conductor | FPGA design flow orchestration | Vivado, Quartus, Efinity, Radiant, Gowin, Yosys+nextpnr, OSS | Container + Local |
| asic-conductor | ASIC design flow orchestration (13-step RTL-to-GDSII) | OpenLane 2, Yosys, OpenROAD, OpenSTA, TritonCTS, TritonRoute, Magic, Netgen, KLayout, Ngspice, SymbiYosys | Container + Local |
| pcb-conductor | PCB design flow orchestration + AI schematic generation | KiCad 9, SKiDL, Freerouting, kicad-mcp-python | Container + Local |
| hal-conductor | Hardware Abstraction Layer generation (register maps / multi-language drivers) | peakrdl, peakrdl-rust, ipyxact, csr2regs, cmsis-svd-gen, svd2rust | Container + Local |
| apps-conductor | Application software (FW / RTOS / bare-metal) development | arm-gcc, riscv-gcc, cargo-embed, west-zephyr, freertos-builder, embassy-builder, qemu-system, probe-rs | Container + Local |
| debug-conductor | Debug environment orchestration | OpenOCD/pyOCD/JTAG/SWD, ILA/SignalTap/Reveal, sigrok/PulseView, WASM waveform viewer | **Local only** |
| rag-conductor | Knowledge base construction, management, and search (separated from ai-conductor) | Chroma/Qdrant, Ollama (nomic-embed-text), PyPDF/pdfplumber, Tesseract OCR, Camelot, trafilatura | Container + Local |

### Conductor Common Architecture Patterns

Each conductor follows the same architecture pattern:

- Rust workspace structure (under `.hestia/tools/`, `Cargo.toml` resolver = 2)
- ToolAdapter / VendorAdapter-based abstraction
- Capability-based adapter registration and resolution engine (AdapterRegistry)
- `adapter.toml` declarative extension (no Rust code changes required)
- Podman rootless container integration (except debug-conductor)
- agent-cli native IPC communication (Phase 113+ allows selecting `agent_cli` / `claude_cli_shim` via `[engine] type`)
- CLI client binary (`hestia-{conductor}-cli`)
- Shared crates: conductor-sdk / adapter-core etc.
- **Sub-agent concurrency control** (Phase 126): 3-tier hierarchical Semaphore (global / ai-dispatch / per-conductor) + acquire timeout via `conductor_sdk::concurrency::ConductorLimiter` to unify spawn caps. Fixed acquisition order + timeout + reviewer reserved slot breaks hold-and-wait and circular wait from Coffman's 4 conditions. Adjustable via `.hestia/config.toml` `[concurrency]`
- **version-TAG synchronization** (Phase 127): `clis/hestia/build.rs` injects `git describe` output as an env variable at build time. `hestia --version` auto-syncs with GitHub TAG, falling back to `[workspace.package] version` when git is unavailable

---

## 5. Technology Stack

| Layer | Technology | Selection Reason |
|---|---|---|
| Core daemon | Rust | Memory safety, async processing (tokio), cross-platform binary, fast execution |
| Frontend | TypeScript (VSCode Extension / Tauri) | Mature ecosystem, Monaco Editor integration, desktop app support |
| Container | Podman (rootless) | Daemonless, rootless unprivileged execution, SELinux support |
| AI agent | TypeScript + Anthropic SDK | Agent loop via Claude Sonnet's Tool Use functionality |
| Persistence | sled (KV) + SQLite | Rust-native, lightweight, embeddable, compatibility matrix DB |
| ASIC flow | Python (OpenLane 2) | OpenLane 2's Python-based Step-based Execution |
| PCB design | Python (SKiDL) | Circuit description language with high LLM compatibility |
| PCB AI | TypeScript + LangChain | Circuit schematic synthesis via LLM framework |

---

## 6. Execution Environment

HESTIA targets **Linux** as its execution environment.

| Category | Requirement |
|------|------|
| Host OS | Linux (x86_64 kernel 5.x or later recommended) |
| Recommended distributions | Ubuntu 22.04 LTS or later / RHEL 8 or later / Debian 12 or later |
| Required kernel features | user namespace (for rootless Podman) / cgroup v2 / SELinux or AppArmor / Unix Domain Socket |
| Unsupported OS | Windows / macOS (not supported as host OS) |
| Development environment (auxiliary) | Windows + WSL2 is acceptable for development assistance. However, CI / production must be native Linux |

Linux-specific dependencies:

- **Container runtime**: Podman rootless depends on Linux user namespace / cgroup / SELinux
- **IPC**: agent-cli native IPC uses POSIX/Linux primitives
- **Security**: SELinux labels are a Linux Security Module feature
- **Async runtime**: tokio uses Linux epoll as its primary backend
- **Container images**: All 8 are Linux-based

---

## Related Documentation

- [glossary.md](glossary.md) — Glossary
- [agent_communication.md](agent_communication.md) — Communication specification
- [security.md](security.md) — Security policy
- [shared_services.md](shared_services.md) — Shared services layer