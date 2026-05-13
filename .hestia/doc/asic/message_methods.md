# asic-conductor Message Methods

**Target Conductor**: asic-conductor
**Source**: Design specification §14 (around lines 3492-3630), §6 (around lines 1761-1981)

## Transport

All communication uses agent-cli native IPC. Peer name: `asic`.

## asic.* Method List

### RTL-to-GDSII Flow

| Method | Direction | Description |
|---------|-----------|-------------|
| `asic.synthesize` | Request | Execute logic synthesis (Yosys) |
| `asic.floorplan` | Request | Create floorplan (OpenROAD) |
| `asic.place` | Request | Execute cell placement (RePlAce + OpenDP) |
| `asic.cts` | Request | Clock tree synthesis (TritonCTS) |
| `asic.route` | Request | Execute routing (FastRoute + TritonRoute) |
| `asic.gdsii` | Request | Generate GDSII stream |

### Signoff

| Method | Direction | Description |
|---------|-----------|-------------|
| `asic.drc` | Request | Design rule check (Magic / KLayout) |
| `asic.lvs` | Request | Layout versus schematic verification (Netgen / KLayout) |
| `asic.timing_signoff` | Request | Timing signoff (OpenSTA) |

### PDK Management

| Method | Direction | Description |
|---------|-----------|-------------|
| `asic.pdk.install` | Request | Install PDK (via volare) |
| `asic.pdk.list` | Request | List installed PDKs |

### AI Agent Integration

| Method | Direction | Description |
|---------|-----------|-------------|
| `asic.ai.timing_fix` | Request | Automatic timing violation fix suggestion |
| `asic.ai.drc_fix` | Request | Automatic DRC violation fix patch generation |
| `asic.ai.floorplan_optimize` | Request | Floorplan optimization suggestion |
| `asic.ai.pdk_migrate` | Request | PDK migration assistance |

### conductor-core Common

| Method | Direction | Description |
|---------|-----------|-------------|
| `system.health.v1` | Request | Health check |
| `system.readiness` | Request | Readiness check |

## Payload Examples

### asic.synthesize Request

```json
{
  "method": "asic.synthesize",
  "params": {
    "pdk": "sky130_fd_sc_hd",
    "strategy": "area",
    "abc_script": "resyn2"
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### asic.drc Request

```json
{
  "method": "asic.drc",
  "params": {
    "tool": "magic",
    "gds_path": "build/results/final.gds"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

## Related Documentation

- [asic/binary_spec.md](binary_spec.md) — hestia-asic-cli binary specification
- [asic/error_types.md](error_types.md) — asic-conductor error codes
- [asic/state_machines.md](state_machines.md) — ASIC build state machine
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter trait
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification