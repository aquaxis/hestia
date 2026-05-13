# asic-conductor Error Codes

**Target Conductor**: asic-conductor
**Source**: Design specification §14.3 (around lines 3565-3581)

## Error Code Range

asic-conductor error codes use the range **-32300 to -32399**.

## Error Categories

### RTL Synthesis

| Code | Name | Description |
|-------|------|------------|
| -32300 | SYNTHESIS_FAILED | Yosys logic synthesis failure |
| -32301 | SYNTHESIS_TIMEOUT | Synthesis timeout |
| -32302 | RTL_READ_ERROR | RTL read error |
| -32303 | TECH_MAPPING_FAILED | Technology mapping failure (ABC) |

### Floorplan

| Code | Name | Description |
|-------|------|------------|
| -32310 | FLOORPLAN_FAILED | Floorplan creation failure |
| -32311 | PDN_GENERATION_FAILED | Power distribution network generation failure |
| -32312 | IO_PLACEMENT_FAILED | I/O pin placement failure |
| -32313 | MACRO_PLACEMENT_FAILED | Macro placement failure |

### Placement

| Code | Name | Description |
|-------|------|------------|
| -32320 | PLACEMENT_FAILED | Cell placement failure |
| -32321 | DENSITY_EXCEEDED | Placement density exceeded |
| -32322 | OVERFLOW_ERROR | Placement overflow |

### CTS (Clock Tree Synthesis)

| Code | Name | Description |
|-------|------|------------|
| -32330 | CTS_FAILED | Clock tree synthesis failure |
| -32331 | CTS_SKEW_VIOLATION | Skew violation |
| -32332 | BUFFER_INSERTION_FAILED | Buffer insertion failure |

### Routing

| Code | Name | Description |
|-------|------|------------|
| -32340 | GLOBAL_ROUTING_FAILED | Global routing failure |
| -32341 | DETAILED_ROUTING_FAILED | Detailed routing failure |
| -32342 | CONGESTION_DETECTED | Routing congestion detected |
| -32343 | DRC_VIOLATION_ROUTING | Routing DRC violation |

### Extraction / Timing

| Code | Name | Description |
|-------|------|------------|
| -32350 | EXTRACTION_FAILED | Parasitic extraction failure (OpenRCX) |
| -32351 | TIMING_SIGNOFF_FAILED | Timing signoff failure (WNS < 0) |

### DRC / LVS / Signoff

| Code | Name | Description |
|-------|------|------------|
| -32360 | DRC_FAILED | DRC check failure (Magic / KLayout) |
| -32361 | LVS_FAILED | LVS check failure (Netgen) |
| -32362 | DRC_VIOLATIONS_FOUND | DRC violations detected |
| -32363 | LVS_MISMATCH_FOUND | LVS mismatch detected |

### GDSII / PDK

| Code | Name | Description |
|-------|------|------------|
| -32370 | GDSII_GENERATION_FAILED | GDSII stream generation failure |
| -32371 | PDK_NOT_INSTALLED | PDK not installed |
| -32372 | PDK_VERSION_MISMATCH | PDK version mismatch |

## Error Response Format

```json
{
  "error": {
    "code": -32351,
    "message": "Timing signoff failed",
    "data": {
      "tool": "opensta",
      "exit_code": 1,
      "log_path": "/workspace/build/timing.log",
      "errors": [
        { "wns": -0.3, "tns": -1.5, "path": "clk -> data_out" }
      ],
      "retry_possible": true,
      "suggested_action": "Add buffer insertion or relax clock period"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123"
}
```

## Related Documentation

- [asic/message_methods.md](message_methods.md) — asic.* method list
- [asic/state_machines.md](state_machines.md) — ASIC build state machine
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter trait
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry