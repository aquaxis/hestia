# fpga-conductor Message Methods

**Target Conductor**: fpga-conductor
**Source**: Design Specification §14 (around lines 3492-3630), §5 (around lines 1398-1760)

## Transport

All communication uses agent-cli native IPC. Peer name: `fpga`.

## fpga.* Method List

### Build Flow

| Method | Direction | Description |
|--------|-----------|-------------|
| `fpga.synthesize` | Request | RTL synthesis (calls adapter.synthesize) |
| `fpga.implement` | Request | Place-and-route (calls adapter.implement) |
| `fpga.bitstream` | Request | Bitstream generation (calls adapter.generate_bitstream) |
| `fpga.simulate` | Request | Run simulation |

### Programming

| Method | Direction | Description |
|--------|-----------|-------------|
| `fpga.program` | Request | Write bitstream to FPGA (debug-conductor §10 integration) |

### Reports

| Method | Direction | Description |
|--------|-----------|-------------|
| `fpga.build.v1.start` | Request | Start full build (specify target) |
| `fpga.build.v1.cancel` | Request | Cancel build |
| `fpga.build.v1.status` | Request | Get build status |

### conductor-core Common

| Method | Direction | Description |
|--------|-----------|-------------|
| `system.health.v1` | Request | Health check |
| `system.readiness` | Request | Readiness check |
| `project_open` | Request | Open project |
| `project_targets` | Request | List targets |
| `report_timing` | Request | Timing report |
| `report_resource` | Request | Resource report |
| `report_messages` | Request | Build messages list |

## Payload Examples

### fpga.build.v1.start Request

```json
{
  "method": "fpga.build.v1.start",
  "params": {
    "target": "artix7_dev",
    "steps": ["synthesize", "implement", "bitstream"]
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### system.health.v1 Response

```json
{
  "result": {
    "status": "online",
    "uptime_secs": 12345,
    "tools_ready": ["vivado", "yosys"],
    "load": { "cpu_pct": 12, "mem_mb": 512 },
    "active_jobs": 3,
    "last_error": null
  },
  "id": "hc_2026-05-01T12:00:00Z_abc123"
}
```

## Related Documentation

- [fpga/binary_spec.md](binary_spec.md) — hestia-fpga-cli binary specification
- [fpga/error_types.md](error_types.md) — fpga-conductor error codes
- [fpga/state_machines.md](state_machines.md) — Build state machine
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter trait
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification