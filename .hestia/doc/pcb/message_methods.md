# pcb-conductor Message Methods

**Target Conductor**: pcb-conductor
**Source**: Design specification §14 (around lines 3492-3630), §7 (around lines 1982-2174)

## Transport

All communication uses agent-cli native IPC. Peer name: `pcb`.

## pcb.* Method List

### Schematic Design

| Method | Direction | Description |
|--------|-----------|-------------|
| `pcb.generate_schematic` | Request | KiCad netlist output (`kicad-cli sch export netlist`) |
| `pcb.ai_synthesize` | Request | AI-driven schematic synthesis (LLM core, Chain-of-Thought 6 stages) |

### Verification

| Method | Direction | Description |
|--------|-----------|-------------|
| `pcb.run_drc` | Request | Run DRC (`kicad-cli pcb drc`) |
| `pcb.run_erc` | Request | Run ERC (`kicad-cli sch erc`) |

### BOM / Placement

| Method | Direction | Description |
|--------|-----------|-------------|
| `pcb.generate_bom` | Request | Generate BOM (`kicad-cli sch export bom`) |
| `pcb.place_components` | Request | Component placement data output (`kicad-cli pcb export pos`) |
| `pcb.route_traces` | Request | Routing and drill data output (`kicad-cli pcb export drill`) |

### Manufacturing Output

| Method | Direction | Description |
|--------|-----------|-------------|
| `pcb.generate_output` | Request | Gerber output (`kicad-cli pcb export gerbers`) |
| `pcb.status` | Request | Get build status |

### conductor-core Common

| Method | Direction | Description |
|--------|-----------|-------------|
| `system.health.v1` | Request | Health check |
| `system.readiness` | Request | Readiness check |

## Payload Examples

### pcb.ai_synthesize Request

```json
{
  "method": "pcb.ai_synthesize",
  "params": {
    "spec": "STM32F103 + BME280 + USB Type-C temperature/humidity sensor board",
    "input_format": "natural_language",
    "output_format": "kicad"
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### pcb.run_drc Request

```json
{
  "method": "pcb.run_drc",
  "params": {
    "pcb_file": "output/motor_controller.kicad_pcb"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

## Related Documentation

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli binary specification
- [pcb/error_types.md](error_types.md) — pcb-conductor error codes
- [pcb/state_machines.md](state_machines.md) — PCB build steps
- [pcb/tool_adapter.md](tool_adapter.md) — AI-driven schematic design / KiCad adapter
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification