# rtl-conductor Configuration Schema

**Target Conductor**: rtl-conductor
**Source**: Design Specification §4.4 (around lines 1288-1310)

## rtl.toml — Unified Project Format

A file that declaratively defines RTL project settings, source definitions, adapter selections, and handoff targets.

### Sections

| Section | Required | Description |
|---------|----------|-------------|
| `[project]` | Required | Project basic settings (name, top module, language) |
| `[sources]` | Required | Source file definitions (RTL / testbench / shared constraints) |
| `[adapters]` | Optional | Adapter selection for each function |
| `[handoff]` | Optional | Handoff artifacts to downstream conductors |

### `[project]` Section

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `top` | string | Top module name |
| `language` | string | HDL language (`systemverilog` / `vhdl` / `chisel` / `spinalhdl` / `amaranth`) |

### `[sources]` Section

| Field | Type | Description |
|-------|------|-------------|
| `rtl` | string[] | RTL source files (glob supported) |
| `testbench` | string[] | Testbench files |
| `constraints_shared` | string[] | Shared constraint files (SDC, etc.) |

### `[adapters]` Section

| Field | Type | Description |
|-------|------|-------------|
| `lint` | string | Lint adapter name |
| `simulation` | string | Simulation adapter name |
| `formal` | string | Formal verification adapter name |

### `[handoff]` Section

| Field | Type | Description |
|-------|------|-------------|
| `fpga` | string[] | Artifacts to pass to fpga-conductor (§5) |
| `asic` | string[] | Artifacts to pass to asic-conductor (§6) |
| `hal_bus_decl` | string | Bus definition input for hal-conductor (§8) |

### Configuration Example

```toml
[project]
name = "core_v"
top = "Cv32e40p"
language = "systemverilog"

[sources]
rtl = ["src/**/*.sv"]
testbench = ["tb/**/*.sv"]
constraints_shared = ["constraints/timing_shared.sdc"]

[adapters]
lint = "verilator-lint"
simulation = "verilator"
formal = "symbiyosys"

[handoff]
fpga = ["build/synth_ready.sv"]
asic = ["build/asic_ready.sv"]
hal_bus_decl = "build/bus_iface.rdl"
```

## Related Documentation

- [rtl/binary_spec.md](binary_spec.md) — hestia-rtl-cli binary specification
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter trait
- [rtl/handoff.md](handoff.md) — Downstream handoff
- [../fpga/config_schema.md](../fpga/config_schema.md) — fpga.toml schema