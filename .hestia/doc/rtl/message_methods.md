# rtl-conductor Message Method List

**Target Conductor**: rtl-conductor
**Source**: Design Specification §4.7 (around lines 1332-1334), §14 (around lines 3492-3630)

## Transport

All communication is unified via agent-cli native IPC. Peer name: `rtl`. Payloads are structured JSON (starting with `{`) or natural language text.

## rtl.* Method List

### Lint

| Method | Direction | Description |
|--------|-----------|-------------|
| `rtl.lint.v1` | Request | Execute HDL source Lint / format / static analysis |
| `rtl.lint.v1.format` | Request | Execute code formatting (Verible, etc.) |

### Simulation

| Method | Direction | Description |
|--------|-----------|-------------|
| `rtl.simulate.v1` | Request | Run simulation (testbench specified, cycle-accurate / behavioral simulation) |

### Formal Verification

| Method | Direction | Description |
|--------|-----------|-------------|
| `rtl.formal.v1` | Request | Run formal verification (property-based, SymbiYosys) |

### Transpilation

| Method | Direction | Description |
|--------|-----------|-------------|
| `rtl.transpile.v1` | Request | Transpile between HDL languages (Chisel/SpinalHDL/Amaranth → Verilog/VHDL) |

### Handoff

| Method | Direction | Description |
|--------|-----------|-------------|
| `rtl.handoff.v1` | Request | Pass artifacts to downstream conductors (fpga / asic / hal) |

## conductor-core Common API

rtl-conductor also implements common methods from the `ConductorRpc` trait.

| Method | Description |
|--------|-------------|
| `system.health.v1` | Health check (Online / Offline / Degraded / Upgrading) |
| `system.readiness` | Readiness check |

## Payload Examples

### rtl.lint.v1 Request

```json
{
  "method": "rtl.lint.v1",
  "params": {
    "project": "core_v",
    "adapter": "verilator-lint",
    "flags": ["--warn-no-UNDRIVEN"]
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### rtl.handoff.v1 Request

```json
{
  "method": "rtl.handoff.v1",
  "params": {
    "target": "fpga",
    "artifacts": ["build/synth_ready.sv"]
  },
  "id": "msg_2026-05-01T12:00:00Z_def456",
  "trace_id": "trace_xyz789"
}
```

## Related Documentation

- [rtl/binary_spec.md](binary_spec.md) — hestia-rtl-cli binary specification
- [rtl/error_types.md](error_types.md) — RTL-specific error types
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter trait
- [rtl/handoff.md](handoff.md) — Downstream handoff
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification