# pcb-conductor Tool Adapter

**Target Conductor**: pcb-conductor
**Source**: Design specification §7.2 (around lines 2021-2061), §7.5 (around lines 2099-2118)

## AI-Driven Schematic Design Pipeline

An AI pipeline that auto-synthesizes KiCad / SKiDL-compatible schematics from natural language specifications.

### Pipeline Flow

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
    │  Input: CircuitRequirements
    │  Output: GeneratedSchematic (netlist, component list, connection info, CoT log)
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

## Knowledge Graph Structure

### Nodes

| Node Type | Attributes |
|-----------|-----------|
| IC | Part number, manufacturer, category, package |
| Pin | Number, name, pin role |
| External component | Type, value, connection target |

### Pin Roles (PinRole — 11 types)

| Role | Description |
|------|-------------|
| PrimaryVdd / PrimaryVss | Main power / GND |
| AnalogVdd / AnalogVss | Analog power / GND |
| SignalInput / SignalOutput / Bidirectional | Signal input / output / bidirectional |
| ClockInput / Reset / BootConfig | Clock / reset / boot configuration |
| NoConnect | No connect |

### Edges

| Edge Type | Description |
|-----------|-------------|
| must_connect_to | Required connection from pin to external component/net |
| requires_bypass_cap | VDD pin → bypass capacitor value |
| pull_up_required / pull_down_required | Pin → pull-up/pull-down resistor value |
| crystal_pair | OSC_IN ↔ OSC_OUT, frequency, load capacitance |

## KiCad Adapter

| Field | Value |
|-------|-------|
| Adapter ID | `org.kicad.kicad8` |
| Supported formats | `kicad*`, `*.kicad_pcb`, `*.kicad_sch` |
| API version | 1 |

### KiCad CLI Subcommand Mapping

| Method | Subcommand | Purpose |
|--------|-----------|---------|
| `generate_schematic` | `sch export netlist` | Netlist output |
| `run_drc` | `pcb drc` | Run DRC |
| `run_erc` | `sch erc` | Run ERC |
| `generate_bom` | `sch export bom` | Generate BOM |
| `place_components` | `pcb export pos` | Component placement data |
| `route_traces` | `pcb export drill` | Drill data |
| `generate_output` | `pcb export gerbers` | Gerber output |

## Crate Structure

```
pcb-conductor/
├── crates/
│   ├── conductor-core/             # agent-cli persona, main.rs
│   ├── project-model/              # pcb.toml parser
│   ├── plugin-registry/            # Tool registration and resolution
│   ├── adapter-kicad/              # KiCad integration
│   ├── schematic-ai/               # AI schematic design engine (Rust)
│   ├── knowledge-graph/            # Datasheet knowledge graph
│   ├── constraint-verifier/        # Multi-level verification engine
│   └── podman-runtime/             # Container management
├── packages/
│   └── pcb-ai/                     # LangChain integration (TypeScript)
└── pcb-cli/                        # Rust CLI
```

## Related Documentation

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli binary specification
- [pcb/config_schema.md](config_schema.md) — pcb.toml schema
- [pcb/state_machines.md](state_machines.md) — PCB build steps
- [pcb/error_types.md](error_types.md) — pcb-conductor error codes