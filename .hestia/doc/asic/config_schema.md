# asic-conductor Configuration Schema

**Target Conductor**: asic-conductor
**Source**: Design specification §6.9 (around lines 1930-1958)

## asic.toml — Unified Project Format

A file that declaratively defines the configuration, PDK specification, synthesis settings, and placement and routing settings for an ASIC project.

### Sections

| Section | Required | Description |
|-----------|----------|------------|
| `[project]` | Required | Project basic configuration |
| `[target]` | Required | PDK and clock period specification |
| `[synthesis]` | Optional | Logic synthesis settings |
| `[placement]` | Optional | Placement settings |
| `[cts]` | Optional | Clock tree synthesis settings |
| `[routing]` | Optional | Routing settings |

### `[project]` Section

| Field | Type | Description |
|-----------|---|------------|
| `name` | string | Project name |
| `version` | string | Version |
| `rtl_files` | string[] | RTL source files (glob supported) |
| `top` | string | Top module name |

### `[target]` Section

| Field | Type | Description |
|-----------|---|------------|
| `pdk` | string | PDK name (e.g., `sky130_fd_sc_hd` / `gf180mcu_fd_sc_mcu7t5v0` / `ihp_sg13g2`) |
| `clock_period_ns` | float | Clock period (nanoseconds) |

### `[synthesis]` Section

| Field | Type | Description |
|-----------|---|------------|
| `flatten` | boolean | Enable flattening |
| `abc_script` | string | ABC technology mapping script (e.g., `resyn2`) |
| `strategy` | string | Synthesis strategy (`area` / `speed` / `balanced`) |

### `[placement]` Section

| Field | Type | Description |
|-----------|---|------------|
| `target_density` | float | Target placement density (0.0 to 1.0) |

### `[cts]` Section

| Field | Type | Description |
|-----------|---|------------|
| `max_skew_ns` | float | Maximum clock skew (nanoseconds) |

### `[routing]` Section

| Field | Type | Description |
|-----------|---|------------|
| `min_layer` | string | Minimum routing layer (e.g., `met1`) |
| `max_layer` | string | Maximum routing layer (e.g., `met5`) |

### Configuration Example

```toml
[project]
name = "my-asic-project"
version = "0.1.0"
rtl_files = ["src/*.v"]
top = "top_module"

[target]
pdk = "sky130_fd_sc_hd"
clock_period_ns = 10.0

[synthesis]
flatten = true
abc_script = "resyn2"
strategy = "area"

[placement]
target_density = 0.6

[cts]
max_skew_ns = 0.5

[routing]
min_layer = "met1"
max_layer = "met5"
```

## Supported PDKs

| PDK | Process | Provider | Use Case |
|-----|---------|----------|----------|
| Sky130 | 130nm CMOS | SkyWater Technology | Digital and mixed-signal, most stable |
| GF180MCU | 180nm CMOS | GlobalFoundries | MCU-oriented, high reliability |
| IHP SG13G2 | 130nm BiCMOS | IHP | High-speed analog and RF design |

## Related Documentation

- [asic/binary_spec.md](binary_spec.md) — hestia-asic-cli binary specification
- [asic/state_machines.md](state_machines.md) — ASIC build state machine
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter trait
- [../rtl/config_schema.md](../rtl/config_schema.md) — rtl.toml schema