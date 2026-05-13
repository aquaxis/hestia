# pcb-conductor Error Codes

**Target Conductor**: pcb-conductor
**Source**: Design specification §14.3 (around lines 3565-3581)

## Error Code Range

pcb-conductor error codes use the range **-32400 to -32499**.

## Error Categories

### Schematic

| Code | Name | Description |
|------|------|-------------|
| -32400 | SCHEMATIC_GENERATION_FAILED | Schematic generation failed |
| -32401 | SCHEMATIC_PARSE_ERROR | Schematic parse error |
| -32402 | NETLIST_GENERATION_FAILED | Netlist generation failed |
| -32403 | SCHEMATIC_FORMAT_UNSUPPORTED | Unsupported schematic format |

### DRC / ERC

| Code | Name | Description |
|------|------|-------------|
| -32410 | DRC_FAILED | DRC execution failed |
| -32411 | DRC_VIOLATIONS_FOUND | DRC violations detected |
| -32412 | ERC_FAILED | ERC execution failed |
| -32413 | ERC_VIOLATIONS_FOUND | ERC violations detected (unconnected pins, power connections, driver conflicts, shorts) |

### BOM / Placement

| Code | Name | Description |
|------|------|-------------|
| -32420 | BOM_GENERATION_FAILED | BOM generation failed |
| -32421 | BOM_PART_NOT_FOUND | Component in BOM not found in library |
| -32422 | PLACEMENT_FAILED | Component placement failed |
| -32423 | PLACEMENT_DRC_ERROR | Post-placement DRC violation |

### Gerber / Output

| Code | Name | Description |
|------|------|-------------|
| -32430 | GERBER_GENERATION_FAILED | Gerber output failed |
| -32431 | DRILL_DATA_FAILED | Drill data generation failed |
| -32432 | OUTPUT_FORMAT_UNSUPPORTED | Unsupported output format |

### AI Synthesis

| Code | Name | Description |
|------|------|-------------|
| -32440 | AI_SYNTHESIS_FAILED | AI-driven schematic synthesis failed |
| -32441 | AI_COT_FAILED | Chain-of-Thought generation failed |
| -32442 | AI_LLM_UNAVAILABLE | LLM backend unavailable |

### Knowledge Graph

| Code | Name | Description |
|------|------|-------------|
| -32450 | KG_BUILD_FAILED | Knowledge graph build failed |
| -32451 | KG_DATASHEET_FETCH_FAILED | Datasheet fetch failed |
| -32452 | KG_NODE_RESOLUTION_FAILED | KG node resolution failed |

### Constraint Verification

| Code | Name | Description |
|------|------|-------------|
| -32460 | CONSTRAINT_VERIFY_SYNTAX | Level 1: Syntax verification failed |
| -32461 | CONSTRAINT_VERIFY_ERC | Level 2: ERC verification failed |
| -32462 | CONSTRAINT_VERIFY_KG_INTRA | Level 3: KG-based verification (intra-pin) failed |
| -32463 | CONSTRAINT_VERIFY_KG_INTER | Level 4: KG-based verification (inter-pin) failed |
| -32464 | CONSTRAINT_VERIFY_TOPOLOGY | Level 5: Topology verification failed |

## Related Documentation

- [pcb/message_methods.md](message_methods.md) — pcb.* method list
- [pcb/state_machines.md](state_machines.md) — PCB build steps
- [pcb/tool_adapter.md](tool_adapter.md) — AI-driven schematic design / KiCad adapter
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry