# fpga-conductor Error Codes

**Target Conductor**: fpga-conductor
**Source**: Design Specification §14.3 (around lines 3565-3581)

## Error Code Range

fpga-conductor error codes use the range **-32200 through -32299**.

## Error Categories

### Synthesis

| Code | Name | Description |
|------|------|-------------|
| -32200 | SYNTHESIS_FAILED | RTL synthesis failed (Vivado / Quartus / Efinity / Yosys) |
| -32201 | SYNTHESIS_TIMEOUT | Synthesis timeout |
| -32202 | SYNTHESIS_INVALID_HDL | Non-synthesizable HDL detected |

### Implementation (Place-and-Route)

| Code | Name | Description |
|------|------|-------------|
| -32210 | IMPLEMENTATION_FAILED | Place-and-route failed |
| -32211 | PLACEMENT_FAILED | Cell placement failed |
| -32212 | ROUTING_FAILED | Routing failed (e.g., excessive congestion) |

### Bitstream

| Code | Name | Description |
|------|------|-------------|
| -32220 | BITSTREAM_GENERATION_FAILED | Bitstream generation failed |
| -32221 | BITSTREAM_INVALID | Generated bitstream is invalid |

### Timing

| Code | Name | Description |
|------|------|-------------|
| -32230 | TIMING_VIOLATION | Timing violation (WNS < 0 / TNS < 0) |
| -32231 | TIMING_ANALYSIS_FAILED | Timing analysis failed |

### Debug / On-Chip

| Code | Name | Description |
|------|------|-------------|
| -32240 | DEBUG_SESSION_FAILED | On-chip debug session failed |
| -32241 | ILA_CONFIGURATION_ERROR | ILA configuration error |

### HLS

| Code | Name | Description |
|------|------|-------------|
| -32245 | HLS_COMPILE_FAILED | HLS compilation failed |

### Device

| Code | Name | Description |
|------|------|-------------|
| -32250 | DEVICE_NOT_FOUND | Target device not found |
| -32251 | DEVICE_PROGRAM_FAILED | Device programming failed |
| -32252 | DEVICE_COMPATIBILITY_ERROR | Device compatibility error |

### Simulation

| Code | Name | Description |
|------|------|-------------|
| -32255 | SIMULATION_FAILED | Simulation failed |
| -32256 | SIMULATION_TIMEOUT | Simulation timeout |

### Constraints

| Code | Name | Description |
|------|------|-------------|
| -32260 | CONSTRAINT_PARSE_ERROR | Constraint file (XDC/SDC/PCF) parse error |
| -32261 | CONSTRAINT_CONVERSION_ERROR | Constraint format conversion error (XDC ⇔ SDC ⇔ PCF) |

### Adapter

| Code | Name | Description |
|------|------|-------------|
| -32270 | ADAPTER_NOT_FOUND | Specified adapter is not registered |
| -32271 | ADAPTER_VERSION_MISMATCH | Adapter API version mismatch |
| -32272 | ADAPTER_MANIFEST_INVALID | Invalid adapter manifest |

## Error Response Format

```json
{
  "error": {
    "code": -32230,
    "message": "Timing violation detected",
    "data": {
      "tool": "vivado",
      "exit_code": 1,
      "log_path": "/workspace/build/vivado_synth.log",
      "errors": [
        { "wns": -0.5, "tns": -3.2, "path": "clk -> ff1" }
      ],
      "retry_possible": true,
      "suggested_action": "Add pipeline registers or relax timing constraints"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123"
}
```

## Related Documentation

- [fpga/message_methods.md](message_methods.md) — fpga.* method list
- [fpga/state_machines.md](state_machines.md) — Build state machine
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter trait
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry