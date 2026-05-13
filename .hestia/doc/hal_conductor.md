# HAL Generation Orchestrator

**Target Domain**: hal-conductor
**Source**: Design specification §8 (lines 2175-2280)

---

## Overview

`hal-conductor` is the conductor responsible for generating and managing the Hardware Abstraction Layer (HAL) that sits at the boundary between HDL (RTL) and application software. It reads register definitions from SystemRDL / IP-XACT / custom schemas and auto-generates driver skeletons, register access APIs, and memory map definitions in multiple languages (C / Rust / Python). It bridges the bus interface definitions from `rtl-conductor` with the high-level drivers used by `apps-conductor`.

---

## Crate Structure

```
crates/hal-conductor/
├── src/
│   ├── lib.rs              # Conductor body, agent-cli message handler
│   ├── adapter.rs          # HalToolAdapter trait
│   ├── register_map.rs     # Register map model (fields/bits/access rights)
│   ├── codegen.rs          # Multi-language code generation (C / Rust / Python)
│   ├── memory_map.rs       # Address space management, overlap detection
│   ├── bus_protocol.rs     # Bus protocol definitions (AXI / Wishbone / AHB)
│   └── fsm_states.rs       # Build state machine
└── Cargo.toml
```

---

## HalToolAdapter Trait

A unified interface. All HAL tool adapters must implement this trait.

```rust
#[async_trait]
pub trait HalToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_inputs(&self) -> &[RegisterFormat];   // SystemRDL / IP-XACT / TOML
    fn supported_outputs(&self) -> &[OutputLang];      // C / Rust / Python / Markdown / SVD

    async fn parse(&self, src: &Path) -> Result<RegisterMap, AdapterError>;
    async fn validate(&self, map: &RegisterMap) -> Result<ValidationReport, AdapterError>;
    async fn generate(&self, map: &RegisterMap, target: OutputLang, out: &Path)
        -> Result<PathBuf, AdapterError>;
}
```

---

## Build State Machine (5 States)

```
Idle → Parsing → Validating → Generating → Reporting → Done
                          ↓ (bus boundary violation / address overlap / type mismatch)
                      Failed → Diagnosing → Fix proposal
```

---

## Unified Project Format (hal.toml)

```toml
[project]
name = "soc_hal"
input_format = "systemrdl"             # "systemrdl" | "ipxact" | "toml"

[sources]
register_definitions = ["regs/**/*.rdl"]
memory_map = "config/memory_map.toml"

[bus]
protocol = "axi4-lite"                  # "axi4-lite" | "axi4" | "wishbone-b4" | "ahb-lite"
data_width = 32
addr_width = 32

[outputs]
c_header = "build/hal/inc/soc_hal.h"
rust_crate = "build/hal/rust/soc-hal"
python_module = "build/hal/python/soc_hal.py"
documentation = "build/hal/docs/registers.md"
svd = "build/hal/svd/soc_hal.svd"
```

---

## Major Adapters

| Adapter | Role | Input | Output |
|---------|------|-------|--------|
| `peakrdl` | SystemRDL multi-language generation | SystemRDL | C / Markdown / HTML |
| `peakrdl-rust` | Rust driver generation | SystemRDL | Rust (embedded-hal compatible) |
| `ipyxact` | IP-XACT parsing | IP-XACT XML | Internal register model |
| `csr2regs` | CSR register generation | TOML / YAML | C / SystemVerilog |
| `cmsis-svd-gen` | CMSIS SVD generation | Internal model | SVD XML |
| `svd2rust-bridge` | SVD → Rust crate | SVD | Rust (svd2rust compatible) |

---

## Upstream/Downstream Integration

- **Upstream (rtl-conductor)**: Takes bus interface declarations defined by rtl-conductor as input and can export the register map in SystemRDL format. Triggered from rtl-conductor via the `hal.handoff` event
- **Downstream (apps-conductor)**: Generated C headers / Rust crates / Python modules are imported via apps-conductor's `[hal] import = "..."`
- **Cross-cutting (debug-conductor)**: debug-conductor reuses the same register map for live debugging register display and editing UI
- **Cross-cutting (asic-conductor / fpga-conductor)**: SystemVerilog template output of register blocks can be passed directly to the corresponding conductor's `[sources]`

---

## Public Methods

Five method families: `hal.parse.v1` / `hal.validate.v1` / `hal.generate.v1` / `hal.export.v1` (rtl/asic export) / `hal.diff.v1` (register map diff). Sent as structured JSON payloads over agent-cli IPC.

---

## Sub-agent Configuration

hal-conductor has four types of sub-agents: **planner / designer / coder (multiple) / validator**, each sharing the multi-language driver generation flow from register maps. Each sub-agent is launched as an independent agent-cli process and coordinates with the hal-conductor main body (peer name `hal`) via `agent-cli send <peer>` IPC.

| Sub-agent | Peer Name | Role | Multiplicity |
|-----------|-----------|------|-------------|
| **planner** | `hal-planner` | HAL generation planning (register block partitioning, bus protocol selection, output language decision) | 1 |
| **designer** | `hal-designer` | HAL detailed specification (register fields, access rights, memory map, SystemRDL/IP-XACT schema) | 1 |
| **coder** | `hal-coder-{lang}` | Per-language driver code generation (`hal-coder-c` / `hal-coder-rust` / `hal-coder-python` / `hal-coder-svd`) | **N** (launched in parallel for each output language) |
| **validator** | `hal-validator` | Register map validation (address overlap / type consistency / bus boundary checks / protocol compliance) | 1 |

**Flow**: planner → designer → coder (in parallel per language) → validator, executed sequentially. If output languages are C / Rust / Python, three coders launch in parallel.

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [rtl_conductor.md](rtl_conductor.md) — RTL design flow orchestrator (upstream)
- [apps_conductor.md](apps_conductor.md) — Application software development orchestrator (downstream)
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator (cross-cutting)
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator (cross-cutting)
- [debug_conductor.md](debug_conductor.md) — Debug environment orchestrator (cross-cutting)