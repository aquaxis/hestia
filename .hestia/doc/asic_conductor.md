# ASIC Design Flow Orchestrator

**Target Domain**: asic-conductor
**Source**: Design specification §6 (lines 1761-1981)

---

## Overview

asic-conductor is an orchestrator that automates the entire 13-step RTL-to-GDSII pipeline. Centered on the open-source toolchain (Yosys / OpenROAD / Magic / Netgen / OpenLane 2), it integrates PDK management (Sky130 / GF180MCU / IHP SG13G2) and executes everything from logic synthesis through physical design to signoff verification.

---

## Crate Structure

```
asic-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona, main.rs
│   ├── project-model/              # asic.toml parser
│   ├── plugin-registry/            # Tool registration and resolution (AsicToolAdapter trait)
│   ├── adapter-openlane/           # OpenLane 2 integration
│   ├── adapter-yosys/              # Yosys logic synthesis (via RTLIL)
│   ├── adapter-openroad/           # OpenROAD placement and routing
│   ├── pdk-manager/                # PDK management (Sky130 / GF180MCU / IHP SG13G2)
│   ├── podman-runtime/             # Container management
│   └── conductor-sdk/              # Shared SDK
├── asic-cli/                       # Rust CLI client
└── conductor-sdk/
```

---

## RTL-to-GDSII 13-Step Pipeline

```
RTL (SystemVerilog / Verilog)
    │
    ▼ 1. Yosys (logic synthesis)
    │   read_verilog → RTLIL → proc → opt → fsm → memory → abc
    │
    ▼ 2. OpenSTA (initial timing analysis)
    │   Early detection of setup/hold violations
    │
    ▼ 3. OpenROAD floorplan
    │   PDN Generation / I/O Placement / Macro Placement
    │
    ▼ 4. RePlAce (global placement)
    │
    ▼ 5. OpenDP (detailed placement)
    │
    ▼ 6. TritonCTS (clock tree synthesis)
    │   Buffer insertion and skew minimization
    │
    ▼ 7. FastRoute (global routing)
    │
    ▼ 8. TritonRoute (detailed routing)
    │   DRC-compliant metal routing
    │
    ▼ 9. OpenRCX (parasitic capacitance extraction)
    │
    ▼ 10. OpenSTA (final timing analysis)
    │    SPEF-based accurate timing verification
    │
    ▼ 11. Magic (DRC) / Netgen (LVS)
    │    Design rule check / layout versus schematic verification
    │
    ▼ 12-13. GDSII output
```

---

## Supported PDKs

| PDK | Process | Provider | Use Case |
|-----|---------|----------|----------|
| Sky130 | 130nm CMOS | SkyWater Technology | Digital and mixed-signal, most stable |
| GF180MCU | 180nm CMOS | GlobalFoundries | MCU-oriented, high reliability |
| IHP SG13G2 | 130nm BiCMOS | IHP | High-speed analog and RF design |

---

## AsicToolAdapter Trait

ASIC-specific tool adapter interface. Unlike the FPGA VendorAdapter, it covers physical design steps (floorplan, CTS, parasitic extraction, etc.).

```rust
#[async_trait]
pub trait AsicToolAdapter: Send + Sync + 'static {
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> &AsicCapabilitySet;

    // Core flow (7 steps)
    async fn synthesize(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn floorplan(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn place(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn cts(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn route(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn extract(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_gdsii(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;

    // Signoff
    async fn timing_signoff(&self, ctx: &AsicBuildContext) -> Result<TimingReport, AdapterError>;
    async fn drc(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
    async fn lvs(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
}
```

---

## 13-State Build State Machine

| State | Progress | Description |
|-------|----------|-------------|
| `Idle` | 0% | Initial state |
| `PdkResolving` | 3% | Resolving PDK version and validating paths |
| `Synthesizing` | 10% | Executing logic synthesis (Yosys) |
| `Floorplanning` | 20% | Creating floorplan |
| `Placing` | 30% | Executing cell placement |
| `CTS` | 45% | Executing clock tree synthesis |
| `Routing` | 60% | Executing routing |
| `Extraction` | 70% | Executing parasitic extraction |
| `TimingSignoff` | 75% | Verifying timing signoff |
| `DRC` | 80% | Running design rule check |
| `LVS` | 90% | Running layout versus schematic verification |
| `GDSII` | 95% | Generating GDSII stream |
| `Success` | 100% | Build successful |

---

## AsicCapabilityRouter (Routing Strategy)

| Strategy | Description |
|---------|------------|
| `PreferOpenLane` | Delegate steps that OpenLane2 can handle to OpenLane2 |
| `StepOptimal` | Select the optimal adapter individually for each step |
| `Explicit` | Use the adapter explicitly specified in asic.toml |

---

## SignoffChecker

Responsible for final verification before tape-out.

```rust
pub struct SignoffResult {
    pub tool: SignoffTool,
    pub check_type: CheckType,     // DRC or LVS
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub summary: SignoffSummary,
}
```

| Tool | Verification Type | Description |
|-------|------------------|------------|
| Magic | DRC | Layout DRC engine |
| Netgen | LVS | SPICE-level circuit comparison |
| KLayout | DRC + LVS | Scriptable layout verification |

**AI Agent Integration Features:**

| Feature | Description |
|---------|------------|
| Automatic timing violation fix | Suggests constraint relaxation or buffer insertion on timing signoff failure |
| Automatic DRC violation fix | Generates layout fix patches based on DRC violation patterns |
| PDK migration | Assists design migration between different PDK families |
| Floorplan optimization | Suggests floorplan improvements based on placement density and routing congestion |

---

## asic.toml Configuration Example

```toml
[project]
name = "my-asic-project"
version = "0.1.0"
rtl_files = ["src/*.v"]
top = "top_module"

[target]
pdk = "sky130_fd_sc_hd"
clock_period_ns = 10.0

[synthesis]
flatten = true
abc_script = "resyn2"
strategy = "area"

[placement]
target_density = 0.6

[cts]
max_skew_ns = 0.5

[routing]
min_layer = "met1"
max_layer = "met5"
```

---

## Hestia Integration Method

Hestia runs OpenLane 2 inside a Podman container and controls it via agent-cli IPC from conductor-core. It leverages OpenLane 2's Python-based Step-based Execution, enabling individual re-execution of each step. PDKs are automatically resolved by pdk-manager, and automatic downloading is supported through volare integration.

---

## Sub-Agent Configuration

asic-conductor has 6 types of sub-agents: **planner / designer / synthesizer / implementer / signoff_checker / tester**, sharing the 13-step RTL-to-GDSII flow. Each sub-agent is launched as an independent agent-cli process and coordinates with the asic-conductor main body (peer name `asic`) via `agent-cli send <peer>` IPC.

| Sub-Agent | Peer Name | Role | Multiplicity |
|----------------|---------|------|-------|
| **planner** | `asic-planner` | ASIC development planning (PDK selection, step execution strategy, signoff plan) | 1 |
| **designer** | `asic-designer` | ASIC detailed specification (floorplan policy, constraints, power plan, tape-out requirements) | 1 |
| **synthesizer** | `asic-synthesizer` | Logic synthesis (Yosys, SDC timing constraint application) | 1 |
| **implementer** | `asic-implementer` | Floorplan + placement + CTS + routing (OpenROAD / TritonCTS / TritonRoute) | 1 |
| **signoff_checker** | `asic-signoff` | DRC / LVS / timing signoff / EM/IR drop analysis (Magic / Netgen / OpenSTA) | 1 |
| **tester** | `asic-tester` | Post-layout simulation, formal verification (SymbiYosys), analog verification via Ngspice | 1 |

**Flow**: Sequential execution in the order planner → designer → synthesizer → implementer → signoff_checker → tester. Aligned with OpenLane 2's Step-based Execution, specific steps can be re-executed by re-invoking the corresponding sub-agent.

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [rtl_conductor.md](rtl_conductor.md) — RTL design flow orchestrator (upstream)
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator
- [hal_conductor.md](hal_conductor.md) — HAL generation orchestrator
- [debug_conductor.md](debug_conductor.md) — Debug environment orchestrator