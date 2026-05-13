# asic-conductor Build State Machine

**Target Conductor**: asic-conductor
**Source**: Design specification §6.5 (around lines 1868-1884)

## 13-State Build State Machine

| State | Progress | Description |
|-------|----------|-------------|
| `Idle` | 0% | Initial state |
| `PdkResolving` | 3% | Resolving PDK version and validating paths |
| `Synthesizing` | 10% | Executing logic synthesis (Yosys) |
| `Floorplanning` | 20% | Creating floorplan |
| `Placing` | 30% | Executing cell placement |
| `CTS` | 45% | Executing clock tree synthesis |
| `Routing` | 60% | Executing routing |
| `Extraction` | 70% | Executing parasitic extraction |
| `TimingSignoff` | 75% | Verifying timing signoff |
| `DRC` | 80% | Running design rule check |
| `LVS` | 90% | Running layout versus schematic verification |
| `GDSII` | 95% | Generating GDSII stream |
| `Success` | 100% | Build successful |

## State Transition Diagram

```
Idle
  │  build_start
  ▼
PdkResolving ─────── PDK not installed → Failed
  │
  ▼
Synthesizing ─────── Yosys synthesis failed → Failed
  │
  ▼
Floorplanning ────── Floorplan failed → Failed
  │
  ▼
Placing ──────────── Placement failed → Failed
  │
  ▼
CTS ──────────────── Clock tree synthesis failed → Failed
  │
  ▼
Routing ──────────── Routing failed → Failed
  │
  ▼
Extraction ────────── Parasitic extraction failed → Failed
  │
  ▼
TimingSignoff ─────── Timing violation → AI fix suggestion or Failed
  │
  ▼
DRC ───────────────── DRC violation → AI fix suggestion or Failed
  │
  ▼
LVS ───────────────── LVS mismatch → Failed
  │
  ▼
GDSII ─────────────── GDSII generation failed → Failed
  │
  ▼
Success
```

## AI Agent Integration on Failure

| Step | AI Integration | Description |
|---------|---------------|-------------|
| TimingSignoff | Automatic timing violation fix | Suggests constraint relaxation or buffer insertion |
| DRC | Automatic DRC violation fix | Generates layout fix patches based on violation patterns |
| Floorplanning | Floorplan optimization | Suggests improvements based on placement density and routing congestion |

## Integration with OpenLane 2 Step-based Execution

asic-conductor leverages OpenLane 2's Python-based Step-based Execution, enabling individual re-execution of each step. The `advance` command allows resuming from a specific step.

## Reproducibility Guarantee via asic.lock

On successful build, the PDK version, tool versions, and build parameters used are recorded in asic.lock, guaranteeing reproducibility in the same environment.

## Related Documentation

- [asic/binary_spec.md](binary_spec.md) — hestia-asic-cli binary specification
- [asic/error_types.md](error_types.md) — asic-conductor error codes
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter trait
- [../fpga/state_machines.md](../fpga/state_machines.md) — FPGA build state machine