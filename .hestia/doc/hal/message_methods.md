# hal-conductor Message Methods

**Target Conductor**: hal-conductor
**Source**: Design specification §8.7 (around lines 2260-2262), §14 (around lines 3492-3630)

## Transport

All communication uses agent-cli native IPC. Peer name: `hal`.

## hal.* Method List

### Register Map Operations

| Method | Direction | Description |
|--------|-----------|-------------|
| `hal.parse.v1` | Request | Parse register definition files (SystemRDL / IP-XACT / TOML → RegisterMap) |
| `hal.validate.v1` | Request | Validate register map (address overlap, type consistency, bus boundary checks) |
| `hal.generate.v1` | Request | Generate code for specified language (C / Rust / Python / Markdown / SVD) |
| `hal.export.v1` | Request | Export for rtl/asic (SystemVerilog template output) |
| `hal.diff.v1` | Request | Show register map diff (comparison between two versions) |

### conductor-core Common

| Method | Direction | Description |
|--------|-----------|-------------|
| `system.health.v1` | Request | Health check |
| `system.readiness` | Request | Readiness check |

## Payload Examples

### hal.parse.v1 Request

```json
{
  "method": "hal.parse.v1",
  "params": {
    "input_format": "systemrdl",
    "sources": ["regs/core.rdl", "regs/peripheral.rdl"]
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### hal.generate.v1 Request

```json
{
  "method": "hal.generate.v1",
  "params": {
    "target_lang": "rust",
    "output_path": "build/hal/rust/soc-hal"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

### hal.diff.v1 Request

```json
{
  "method": "hal.diff.v1",
  "params": {
    "baseline": "build/hal/v1.0/registers.json",
    "current": "build/hal/v1.1/registers.json"
  },
  "id": "msg_2026-05-01T12:00:00Z_ghi789"
}
```

## Upstream/Downstream Integration

- **Upstream (rtl-conductor §4)**: Takes bus interface declarations defined by rtl-conductor as input and can export the register map in SystemRDL format. Triggered from rtl-conductor via the `hal.handoff` event
- **Downstream (apps-conductor §9)**: Generated C headers / Rust crates / Python modules are imported via apps-conductor's `[hal] import = "..."`
- **Cross-cutting (debug-conductor §10)**: debug-conductor reuses the same register map for live debugging register display and editing UI
- **Cross-cutting (asic/fpga conductor)**: SystemVerilog template output of register blocks can be passed directly to the corresponding conductor's `[sources]`

## Related Documentation

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli binary specification
- [hal/register_map.md](register_map.md) — Register map definition
- [hal/codegen.md](codegen.md) — Multi-language code generation
- [hal/state_machines.md](state_machines.md) — Build state machine
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification