# pcb-conductor Build Steps

**Target Conductor**: pcb-conductor
**Source**: Design specification §7.4 (around lines 2085-2098)

## 9-Step Build Flow

| Step | Description | Key Component |
|------|-------------|---------------|
| `ParseRequirements` | Parse requirements | RequirementsParser (natural language → CircuitRequirements) |
| `GenerateBom` | Generate BOM | BOM Generator (requirements → component list) |
| `AnalyzeDatasheet` | Analyze datasheets | Datasheet Fetcher (download and parse datasheets for each IC) |
| `BuildKnowledgeGraph` | Build knowledge graph | Knowledge Graph Builder (datasheets → KG) |
| `SynthesizeSchematic` | Schematic synthesis (LLM core) | Schematic Synthesizer (CoT 6 stages) |
| `Verify` | Verify (DRC/ERC/KG 5 levels) | Constraint Verifier (multi-level verification engine) |
| `PlaceComponents` | Place components | KiCad adapter (`pcb export pos`) |
| `RouteTraces` | Route traces | KiCad adapter (`pcb export drill`) + Freerouting integration |
| `GenerateOutput` | Generate manufacturing output (Gerber, etc.) | KiCad adapter (`pcb export gerbers`) |

## State Transitions

```
ParseRequirements
       │
       ▼
GenerateBom
       │
       ▼
AnalyzeDatasheet
       │
       ▼
BuildKnowledgeGraph
       │
       ▼
SynthesizeSchematic ← Feedback loop (up to 3 attempts)
       │              ↑
       ▼              │
Verify ───── Fail →───┘
       │
       │ Pass
       ▼
PlaceComponents
       │
       ▼
RouteTraces
       │
       ▼
GenerateOutput
       │
       ▼
Done
```

## Feedback Loop

When the Verify step fails, it returns to the SynthesizeSchematic step and retries generation (up to 3 attempts). This iteratively improves the quality of AI-generated schematics.

## Step Details

### ParseRequirements

Converts natural language specification input into CircuitRequirements (summary, I/O, power voltage, constraints).

### GenerateBom

Generates a component list (part number, quantity, manufacturer) from CircuitRequirements.

### AnalyzeDatasheet

Downloads and parses datasheets for each IC, extracting pin assignments, electrical characteristics, and recommended circuits. Leverages the datasheet knowledge base from rag-conductor (§13.7).

### BuildKnowledgeGraph

Constructs a knowledge graph (IC nodes + pin nodes + edges) from datasheets.

### SynthesizeSchematic

Synthesizes schematics via LLM Chain-of-Thought 6 stages.

### Verify

Five-level multi-stage verification:
- Level 1: Syntax verification
- Level 2: ERC
- Level 3: KG-based verification (intra-pin)
- Level 4: KG-based verification (inter-pin)
- Level 5: Topology verification

## Related Documentation

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli binary specification
- [pcb/error_types.md](error_types.md) — pcb-conductor error codes
- [pcb/tool_adapter.md](tool_adapter.md) — AI-driven schematic design / KiCad adapter
- [pcb/config_schema.md](config_schema.md) — pcb.toml schema