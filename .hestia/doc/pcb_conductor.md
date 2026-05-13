# PCB Design Flow Orchestrator

**Target Domain**: pcb-conductor
**Source**: Design specification §7 (lines 1982-2174)

---

## Overview

pcb-conductor is an orchestrator that manages the PCB (printed circuit board) design flow. Its most distinctive feature is the AI-driven schematic design pipeline, which constructs a knowledge graph from natural language specifications and auto-synthesizes schematics using Chain-of-Thought prompting. It integrates KiCad as the primary tool and collaborates with SKiDL / Freerouting to provide end-to-end support from design to manufacturing output.

---

## Crate Structure

```
pcb-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona, main.rs
│   ├── project-model/              # pcb.toml parser
│   ├── plugin-registry/            # Tool registration and resolution
│   ├── adapter-kicad/              # KiCad integration
│   ├── schematic-ai/               # AI schematic design engine (Rust)
│   │   └── src/
│   │       ├── lib.rs              # SchematicAiEngine
│   │       ├── cot_prompt.rs       # Chain-of-Thought prompt generation
│   │       └── requirements.rs     # CircuitRequirements parser
│   ├── knowledge-graph/            # Datasheet knowledge graph
│   │   └── src/
│   │       ├── lib.rs              # KnowledgeGraph
│   │       ├── node.rs             # IC/pin/external component nodes
│   │       └── edge.rs             # Connection constraint edges
│   ├── constraint-verifier/        # Multi-level verification engine
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── syntax.rs           # Level 1: Syntax verification
│   │       ├── erc.rs              # Level 2: ERC
│   │       ├── kg_intra.rs         # Level 3: KG-based verification (intra-pin)
│   │       ├── kg_inter.rs         # Level 4: KG-based verification (inter-pin)
│   │       └── topology.rs         # Level 5: Topology verification
│   └── podman-runtime/             # Container management
├── packages/
│   └── pcb-ai/                     # LangChain integration (TypeScript)
│       └── src/
│           └── schematic_synthesizer.ts  # LLM schematic synthesis
├── pcb-cli/                        # Rust CLI
└── conductor-sdk/
```

---

## AI-Driven Schematic Design Pipeline

```
Natural language specification input
    │  "STM32F103 + BME280 + USB Type-C temperature/humidity sensor board"
    ↓
[Step 1] Requirements Parser
    │  Natural language → structured requirements (CircuitRequirements)
    ↓
[Step 2] BOM Generator
    │  Requirements → component list (part number, quantity, manufacturer)
    ↓
[Step 3] Datasheet Fetcher
    │  Download and parse datasheets for each IC
    ↓
[Step 4] Knowledge Graph Builder
    │  Datasheets → KG (IC nodes + pin nodes + edges)
    ↓
[Step 5] Schematic Synthesizer ← LLM core
    │  Chain-of-Thought (CoT) 6 stages:
    │    Stage 1: RequirementsAnalysis — Analyze circuit purpose, I/O, power, constraints
    │    Stage 2: BlockDiagram — Define functional blocks and signal flow
    │    Stage 3: ComponentSelection — Select components balancing availability, cost, performance
    │    Stage 4: CircuitTopology — Detailed circuit design including bypass caps/pull-ups/ESD protection
    │    Stage 5: NetlistGeneration — KiCad-compatible netlist output
    │    Stage 6: Verification — Power/GND/decoupling/signal integrity verification
    ↓
[Step 6] Constraint Verifier (multi-level verification)
    │  Level 1: Python / SKiDL syntax check, library existence verification
    │  Level 2: ERC — unconnected pins, power connections, driver conflicts, short detection
    │  Level 3: KG-based verification (intra-pin) — VDD/VSS connections, bypass capacitors
    │  Level 4: KG-based verification (inter-pin) — inter-IC interface consistency
    │  Level 5: Topology verification — subgraph isomorphism, signal path completeness
    ↓  ← Feedback loop (returns to Step 5 on failure, up to 3 attempts)
[Step 7] Output Generator
    │  Netlist output in KiCad / Altium format
    ↓
Design complete
```

---

## Knowledge Graph Structure

```
Nodes:
  ├── IC: {part number, manufacturer, category, package}
  ├── Pin: {number, name, pin role}
  └── External component: {type, value, connection target}

Pin roles (PinRole — 11 types):
  ├── PrimaryVdd / PrimaryVss (main power / GND)
  ├── AnalogVdd / AnalogVss (analog power / GND)
  ├── SignalInput / SignalOutput / Bidirectional
  ├── ClockInput / Reset / BootConfig
  └── NoConnect

Edges:
  ├── must_connect_to: {pin → external component/net}
  ├── requires_bypass_cap: {VDD pin → capacitor value}
  ├── pull_up_required / pull_down_required: {pin → resistance value}
  └── crystal_pair: {OSC_IN ↔ OSC_OUT, frequency, load capacitance}
```

---

## PCB Build Steps (9 Steps)

| Step | Description |
|------|-------------|
| `ParseRequirements` | Parse requirements |
| `GenerateBom` | Generate BOM |
| `AnalyzeDatasheet` | Analyze datasheets |
| `BuildKnowledgeGraph` | Build knowledge graph |
| `SynthesizeSchematic` | Synthesize schematic (LLM core) |
| `Verify` | Verify (DRC/ERC/KG 5 levels) |
| `PlaceComponents` | Place components |
| `RouteTraces` | Route traces |
| `GenerateOutput` | Generate manufacturing output (Gerber, etc.) |

---

## KiCad Adapter

| Field | Value |
|-------|-------|
| Adapter ID | `org.kicad.kicad8` |
| Supported formats | `kicad*`, `*.kicad_pcb`, `*.kicad_sch` |
| API version | 1 |

**KiCad CLI Subcommand Mapping:**

| Method | Subcommand | Purpose |
|--------|-----------|---------|
| `generate_schematic` | `sch export netlist` | Netlist output |
| `run_drc` | `pcb drc` | Run DRC |
| `run_erc` | `sch erc` | Run ERC |
| `generate_bom` | `sch export bom` | Generate BOM |
| `place_components` | `pcb export pos` | Component placement data |
| `route_traces` | `pcb export drill` | Drill data |
| `generate_output` | `pcb export gerbers` | Gerber output |

---

## pcb.toml Configuration Example

```toml
[project]
name = "my-pcb-project"
version = "0.1.0"
board_name = "motor_controller"

[board]
layer_count = 4
width_mm = 100
height_mm = 80

[[layers]]
name = "F.Cu"
type = "signal"

[[layers]]
name = "In1.Cu"
type = "power"

[[layers]]
name = "In2.Cu"
type = "ground"

[[layers]]
name = "B.Cu"
type = "signal"

[design]
input_format = "natural_language"
ai_enabled = true

[output]
format = "kicad"
output_dir = "output/"
```

---

## Sub-agent Configuration

pcb-conductor has five types of sub-agents: **planner / designer / schematic / layout / tester**, each sharing the schematic design → placement/routing → verification flow. Each sub-agent is launched as an independent agent-cli process and coordinates with the pcb-conductor main body (peer name `pcb`) via `agent-cli send <peer>` IPC.

| Sub-agent | Peer Name | Role | Multiplicity |
|-----------|-----------|------|-------------|
| **planner** | `pcb-planner` | PCB development planning (board scale, layer count, connector placement, component procurement strategy) | 1 |
| **designer** | `pcb-designer` | PCB detailed specification (circuit block configuration, I/O placement, power plan, signal integrity requirements) | 1 |
| **schematic** | `pcb-schematic` | AI-driven schematic design (SKiDL / KiCad, knowledge graph utilization) | 1 |
| **layout** | `pcb-layout` | Artwork (placement + routing, Freerouting integration) | 1 |
| **tester** | `pcb-tester` | DRC / ERC / BOM verification + Gerber output verification | 1 |

**Flow**: planner → designer → schematic → layout → tester, executed sequentially. AI-driven schematic generation is handled by the schematic sub-agent.

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [rtl_conductor.md](rtl_conductor.md) — RTL design flow orchestrator
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator
- [hal_conductor.md](hal_conductor.md) — HAL generation orchestrator