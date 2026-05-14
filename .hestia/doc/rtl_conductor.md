# RTL Design Flow Orchestrator

**Target Domain**: rtl-conductor
**Source**: Design Specification §4 (lines 1241-1397)

---

## Overview

`rtl-conductor` is a shared-layer orchestrator positioned upstream of FPGA / ASIC implementation, responsible for design, verification, and analysis at the HDL language level (SystemVerilog / VHDL / Chisel / SpinalHDL / Amaranth / MyHDL). It has a handoff mechanism that passes synthesizable RTL to downstream `fpga-conductor` / `asic-conductor`, providing a vendor-tool-independent RTL development process through a unified interface.

---

## Crate Structure

```
crates/rtl-conductor/
├── src/
│   ├── lib.rs              # Conductor main body and agent-cli message handler
│   ├── adapter.rs          # RtlToolAdapter trait
│   ├── language.rs         # HDL language identification / transpilation management
│   ├── lint.rs             # Lint / format / static analysis
│   ├── simulation.rs       # Simulation integration
│   ├── formal.rs           # Formal verification integration
│   ├── repository.rs       # RTL module registry
│   ├── handoff.rs          # Handoff to downstream conductors
│   └── fsm_states.rs       # Build state machine
└── Cargo.toml
```

---

## RtlToolAdapter Trait

Unified interface. A trait that all RTL tool adapters must implement.

```rust
#[async_trait]
pub trait RtlToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_languages(&self) -> &[HdlLanguage];   // SystemVerilog / VHDL / Chisel / SpinalHDL / Amaranth
    fn capabilities(&self) -> RtlCapability;            // Lint | Sim | Formal | Transpile

    async fn lint(&self, project: &RtlProject) -> Result<LintReport, AdapterError>;
    async fn simulate(&self, project: &RtlProject, tb: &TestBench) -> Result<SimReport, AdapterError>;
    async fn formal_verify(&self, project: &RtlProject, props: &[Property])
        -> Result<FormalReport, AdapterError>;
    async fn transpile(&self, src: &Path, target_lang: HdlLanguage)
        -> Result<PathBuf, AdapterError>;
}
```

---

## Build State Machine (7 States)

```
Idle → Resolving → Linting → Compiling → Simulating → FormalChecking → Reporting → Done
                                              ↓ (on failure)
                                          Failed → Diagnosing → Fix suggestion
```

---

## Unified Project Format (rtl.toml)

```toml
[project]
name = "core_v"
top = "Cv32e40p"
language = "systemverilog"            # "systemverilog" | "vhdl" | "chisel" | "spinalhdl" | "amaranth"

[sources]
rtl = ["src/**/*.sv"]
testbench = ["tb/**/*.sv"]
constraints_shared = ["constraints/timing_shared.sdc"]

[adapters]
lint = "verilator-lint"
simulation = "verilator"
formal = "symbiyosys"

[handoff]
fpga = ["build/synth_ready.sv"]       # Passed to fpga-conductor [sources]
asic = ["build/asic_ready.sv"]        # Passed to asic-conductor [sources]
hal_bus_decl = "build/bus_iface.rdl"  # hal-conductor bus definition input
```

---

## 11 Primary Adapters

| Adapter | Role | Supported Languages |
|----------|------|---------------------|
| `verilator-lint` | Lint | SystemVerilog / Verilog |
| `verible` | Format / Lint | SystemVerilog |
| `verilator` | Cycle-accurate simulation | SystemVerilog / Verilog |
| `iverilog` | Simulation | Verilog |
| `ghdl` | VHDL simulation | VHDL |
| `symbiyosys` | Formal verification (properties) | SystemVerilog |
| `riscof` | RISC-V ISA compliance | RV32I/M/A/F/D/C |
| `cocotb` | Python testbench | All languages |
| `chisel-bridge` | Chisel → Verilog | Chisel |
| `spinalhdl-bridge` | SpinalHDL → Verilog | SpinalHDL |
| `amaranth-bridge` | Amaranth → Verilog | Amaranth (Python) |

---

## Downstream Integration (Handoff)

When rtl-conductor completes a build, it emits a `meta.handoff` event, and ai-conductor triggers the downstream workflow (fpga-conductor / asic-conductor / hal-conductor). Handoff artifacts are explicitly specified in the `[handoff]` section of rtl.toml.

---

## Public Methods

Five method families: `rtl.lint.v1` / `rtl.simulate.v1` / `rtl.formal.v1` / `rtl.transpile.v1` / `rtl.handoff.v1`. Sent as structured JSON payloads over agent-cli IPC.

---

## Sub-agent Configuration

rtl-conductor has 4 types of sub-agents: **planner / designer / coder (multiple) / tester**, with multiple coders assigned in parallel per functional module to streamline RTL development. Each sub-agent runs as an independent agent-cli process and coordinates with the rtl-conductor main body (peer name `rtl`) via `agent-cli send <peer>` IPC.

| Sub-agent | Peer Name | Role | Multiplicity |
|-----------|-----------|------|--------------|
| **planner** | `rtl-planner` | Overall RTL development planning (module partition plan, development order, verification strategy, work assignment proposal to coders) | 1 |
| **designer** | `rtl-designer` | RTL development detailed specification (module interface, signal definitions, state machines, timing constraints) | 1 |
| **coder** | `rtl-coder-{module}` | HDL code implementation per assigned functional module | **N** (dynamically started in parallel per module count, capped by `per_conductor_max` default 4, Phase 126) |
| **tester** | `rtl-tester` | RTL verification (lint / simulation / formal verification / testbench execution / coverage aggregation) | 1 (parallelizable as needed) |

**Parallel Development Flow:**

1. Request planner to create a plan (module list + dependencies + development order)
2. Request designer to create detailed module specifications
3. Start and assign N coders in parallel per module count (actual spawning capped by `per_conductor_max` via `ConductorLimiter`, Phase 126)
4. Request tester to verify (after module completion / full integration)
5. Aggregate all deliverables → notify ai-conductor of completion + handoff to downstream conductors

> **Phase 126 / 129 concurrency control**: The legacy hardcoded 16-way parallelism was
> replaced in Phase 126 with `conductor_sdk::concurrency::ConductorLimiter` (env-driven),
> and **Phase 129 switched `per_conductor_max` semantics to "alive cap"**.
>
> When calling dispatch_coders.v1, `count_alive_peers_with_prefix("rtl-coder-")`
> retrieves the current alive coder count, and `available = per_conductor_max - alive`
> calculates remaining slots. When the alive cap is reached, `status: "cap_exhausted"`
> causes 0 spawns to be skipped, and the system waits for existing coders to complete
> or consolidation via `hestia kill`.
>
> The limit can be changed with `HESTIA_PER_CONDUCTOR_MAX` (default 4), and after
> `HESTIA_ACQUIRE_TIMEOUT_SECS` (default 600) elapsed, an acquire timeout skips that
> coder. See [`user_guide.md`](user_guide.md) §3.12 for details.

**Startup Command Examples:**

```bash
# Resident sub-agents
agent-cli run --persona-file ./.hestia/personas/rtl-planner.md  --name rtl-planner  &
agent-cli run --persona-file ./.hestia/personas/rtl-designer.md --name rtl-designer &
agent-cli run --persona-file ./.hestia/personas/rtl-tester.md   --name rtl-tester   &
# coders are dynamically started by rtl-conductor based on planner output
```

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [ai_conductor.md](ai_conductor.md) — ai-conductor overall overview
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator (downstream)
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator (downstream)
- [hal_conductor.md](hal_conductor.md) — HAL generation orchestrator (downstream)