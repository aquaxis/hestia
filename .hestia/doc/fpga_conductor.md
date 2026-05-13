# FPGA Design Flow Orchestrator

**Target Domain**: fpga-conductor
**Source**: Design Specification §5 (lines 1398-1760)

---

## Overview

fpga-conductor is an orchestrator that uniformly manages FPGA design synthesis, place-and-route, bitstream generation, and device programming. It abstracts all vendors — AMD Vivado / Intel Quartus Prime / Efinix Efinity / Lattice Radiant / OSS (Yosys + nextpnr) — via the VendorAdapter trait, and provides declarative project definition through fpga.toml and reproducibility through fpga.lock.

---

## Crate Structure

```
fpga-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona, state machine, main.rs
│   │   └── src/
│   │       ├── main.rs             # Daemon entry point
│   │       ├── rpc.rs              # agent-cli message handler
│   │       ├── state_machine.rs    # Build state machine
│   │       ├── router.rs           # CapabilityRouter
│   │       └── self_healing.rs     # SelfHealingPipeline
│   ├── project-model/              # fpga.toml parser and model
│   │   └── src/
│   │       ├── lib.rs              # ProjectInfo, Target definitions
│   │       ├── parser.rs           # TOML parser
│   │       └── lock.rs            # fpga.lock management
│   ├── plugin-registry/            # Adapter registration and resolution engine
│   │   └── src/
│   │       ├── lib.rs              # PluginRegistry
│   │       ├── adapter/
│   │       │   ├── mod.rs          # VendorAdapter trait
│   │       │   ├── script.rs       # ScriptAdapter (adapter.toml)
│   │       │   ├── dynamic.rs      # Dynamic Adapter (dlopen)
│   │       │   └── remote.rs       # Remote Adapter (gRPC)
│   │       └── capability.rs       # CapabilitySet, CapabilityRouter
│   ├── adapter-vivado/             # AMD Vivado adapter
│   │   └── src/
│   │       ├── lib.rs              # VivadoAdapter implementation
│   │       └── templates/          # TCL templates (minijinja)
│   ├── adapter-quartus/            # Intel Quartus Prime adapter
│   │   └── src/
│   │       └── lib.rs              # QuartusAdapter (QSF/QIP generation)
│   ├── adapter-efinity/            # Efinix Efinity adapter
│   │   └── src/
│   │       └── lib.rs              # EfinityAdapter (Python API invocation)
│   ├── constraint-bridge/          # XDC ⇔ SDC ⇔ Efinity XML ⇔ PCF conversion
│   ├── toolchain-registry/         # Version detection and resolution
│   ├── compat-matrix/              # Compatibility matrix DB (SQLite)
│   ├── podman-runtime/             # Podman container management
│   ├── hdl-lsp-broker/             # HDL language server proxy
│   ├── waveform-core/              # VCD/FST parser (WASM support)
│   └── agent-system/               # AI agent group (Rust part)
│       └── src/
│           ├── watcher.rs          # WatcherAgent
│           ├── probe.rs            # ProbeAgent
│           └── validator.rs        # ValidatorAgent
├── packages/
│   ├── vscode-extension/           # VSCode extension (TypeScript)
│   ├── agent-system/               # PatcherAgent (TypeScript + Anthropic SDK)
│   ├── fpga-ci/                    # CI/CD CLI (TypeScript)
│   └── tauri-app/                  # Tauri desktop app
├── fpga-cli/                       # Rust CLI client
└── conductor-sdk/                  # Third-party SDK
```

---

## VendorAdapter Trait

A unified interface that all adapters must implement.

```rust
#[async_trait::async_trait]
pub trait VendorAdapter: Send + Sync + 'static {
    // --- Required: Self-description ---
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> CapabilitySet;

    // --- Required: Core flow ---
    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;

    // --- Optional (default: returns CapabilityUnsupported) ---
    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError>;
    async fn start_debug_session(&self, ctx: &BuildContext) -> Result<DebugSession, AdapterError>;
    async fn hls_compile(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn program_device(&self, ctx: &ProgramContext) -> Result<(), AdapterError>;
    async fn simulate(&self, ctx: &SimContext) -> Result<SimResult, AdapterError>;

    // --- Log diagnostics (default: None) ---
    fn parse_log_line(&self, line: &str) -> Option<Diagnostic> { None }
}
```

**AdapterManifest**: `id` / `name` / `version` / `vendor` / `api_version` / `supported_devices` / `capabilities` / `release_notes_url`

**CapabilitySet**: `synthesis` / `implementation` / `bitstream` / `timing_analysis` / `on_chip_debug` / `device_program` / `hls` / `simulation` / `ip_catalog`

---

## Build State Machine

```
Idle → Resolving (resolve toolchain)
     → ContainerStarting (start Podman container)
     → Synthesizing (adapter.synthesize)
     → Implementing (adapter.implement)
     → Bitstreamming (adapter.generate_bitstream)
     → Success

On failure at any step → SelfHealingPipeline.on_build_failure()
    → Diagnose via CompatibilityMatrix
    → Known patch available → Auto-apply/notify
    → Unknown error      → Launch PatcherAgent
```

---

## Unified Project Format (fpga.toml)

```toml
[project]
name    = "my_dsp_core"
version = "0.2.0"
hdl_files   = ["hdl/top.sv", "hdl/fir_filter.sv", "hdl/bram_ctrl.sv"]
include_dirs = ["hdl/include"]
testbenches = ["sim/tb_top.sv", "sim/tb_fir.sv"]

# Target definitions
[targets.artix7_dev]
vendor      = "xilinx"
device      = "xc7a35tcsg324-1"
top         = "top"
constraints = ["constraints/artix7.xdc"]

[targets.cyclone10]
vendor      = "intel"
device      = "10CL025YU256C8G"
top         = "top"
constraints = ["constraints/cyclone10.sdc"]

[targets.trion_t20]
vendor            = "efinix"
device            = "T20F256"
top               = "top"
interface_script  = "constraints/trion_t20.peri.xml"

[targets.ice40]
vendor      = "yosyshq"     # OSS chain (adapter.toml)
device      = "iCE40HX8K"
top         = "top"
constraints = ["constraints/ice40.pcf"]

# Toolchain version constraints (semver)
[toolchain]
vivado   = ">=2023.1, <2026"
quartus  = "~23.1"
efinity  = "*"

[toolchain.lock]
vivado   = "2025.2.0"
quartus  = "23.1.1"
efinity  = "2025.2.0"

[ip.fifo_gen]
vendor  = "xilinx"
name    = "fifo_generator"
version = "13.2"
config  = "ip/fifo_gen.xci"

[build]
parallel_jobs       = 8
incremental_compile = true
cache_dir           = ".fpga-cache"

[sim]
tool    = "iverilog"
top_tb  = "tb_top"
plusargs = ["+DUMP_WAVES=1"]
```

---

## Vivado Adapter Implementation

- Auto-generates TCL scripts via minijinja template engine
- Runs Vivado in `-mode batch`
- Real-time log parsing (regex matching `ERROR: [Synth 8-439]` format)

## Quartus Adapter Implementation

- Auto-generates .qpf / .qsf project files
- Runs full flow via `quartus_sh --flow compile`

## Efinity Adapter Implementation

- Generates interface scripts (XML) directly via Rust serde
- Generates build scripts via Rust template engine
- Runs using Efinity-bundled Python (no external Python dependency)

---

## Sub-Agent Configuration

fpga-conductor has six sub-agent types: **planner / designer / synthesizer / implementer / tester / programmer**. Each sub-agent is launched as an independent agent-cli process and coordinates with the fpga-conductor main body (peer name `fpga`) via `agent-cli send <peer>` IPC.

| Sub-Agent | Peer Name | Role | Multiplicity |
|-----------|-----------|------|--------------|
| **planner** | `fpga-planner` | FPGA development planning (target/family selection, build strategy, IP usage decisions) | 1 |
| **designer** | `fpga-designer` | FPGA detailed specifications (XDC/SDC/PCF constraints, IO mapping, clock domains, IP configuration) | 1 |
| **synthesizer** | `fpga-synthesizer` | RTL → netlist synthesis (Vivado / Quartus / Efinity / Yosys+nextpnr) | 1 (N for parallel targets) |
| **implementer** | `fpga-implementer` | Place-and-route + bitstream generation | 1 (N for parallel targets) |
| **tester** | `fpga-tester` | Simulation + timing verification + resource analysis | 1 |
| **programmer** | `fpga-programmer` | Bitstream programming to FPGA (debug-conductor integration) | 1 |

**Flow**: Sequential execution: planner → designer → synthesizer → implementer → tester → programmer. During multi-target parallel builds, synthesizer / implementer are dynamically launched per target.

**Launch command examples:**

```bash
agent-cli run --persona-file ./.hestia/personas/fpga-planner.md     --name fpga-planner     &
agent-cli run --persona-file ./.hestia/personas/fpga-designer.md    --name fpga-designer    &
agent-cli run --persona-file ./.hestia/personas/fpga-synthesizer.md --name fpga-synthesizer &
agent-cli run --persona-file ./.hestia/personas/fpga-implementer.md --name fpga-implementer &
agent-cli run --persona-file ./.hestia/personas/fpga-tester.md      --name fpga-tester      &
agent-cli run --persona-file ./.hestia/personas/fpga-programmer.md  --name fpga-programmer  &
```

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [rtl_conductor.md](rtl_conductor.md) — RTL design flow orchestrator (upstream)
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator
- [hal_conductor.md](hal_conductor.md) — HAL generation orchestrator
- [debug_conductor.md](debug_conductor.md) — Debug environment orchestrator