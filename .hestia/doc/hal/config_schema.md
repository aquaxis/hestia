# hal-conductor Configuration Schema

**Target Conductor**: hal-conductor
**Source**: Design specification §8.4 (around lines 2218-...)

## hal.toml — Unified Project Format

A file that declaratively defines HAL project settings, register definition sources, bus protocol, and output destinations.

### Section List

| Section | Required | Description |
|---------|----------|-------------|
| `[project]` | Required | Project basic settings (name, input format) |
| `[sources]` | Required | Register definition sources and memory map |
| `[bus]` | Required | Bus protocol, data width, and address width |
| `[outputs]` | Optional | File paths for each output language |

### `[project]` Section

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `input_format` | string | Input format (`systemrdl` / `ipxact` / `toml`) |

### `[sources]` Section

| Field | Type | Description |
|-------|------|-------------|
| `register_definitions` | string[] | Register definition files (glob supported, e.g., `regs/**/*.rdl`) |
| `memory_map` | string | Memory map configuration file path |

### `[bus]` Section

| Field | Type | Description |
|-------|------|-------------|
| `protocol` | string | Bus protocol (`axi4-lite` / `axi4` / `wishbone-b4` / `ahb-lite`) |
| `data_width` | integer | Data width in bits (e.g., 32) |
| `addr_width` | integer | Address width in bits (e.g., 32) |

### `[outputs]` Section

| Field | Type | Description |
|-------|------|-------------|
| `c_header` | string | C header output path |
| `rust_crate` | string | Rust crate output path |
| `python_module` | string | Python module output path |
| `documentation` | string | Markdown documentation output path |
| `svd` | string | SVD file output path |

### Configuration Example

```toml
[project]
name = "soc_hal"
input_format = "systemrdl"

[sources]
register_definitions = ["regs/**/*.rdl"]
memory_map = "config/memory_map.toml"

[bus]
protocol = "axi4-lite"
data_width = 32
addr_width = 32

[outputs]
c_header = "build/hal/inc/soc_hal.h"
rust_crate = "build/hal/rust/soc-hal"
python_module = "build/hal/python/soc_hal.py"
documentation = "build/hal/docs/registers.md"
svd = "build/hal/svd/soc_hal.svd"
```

## Related Documentation

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli binary specification
- [hal/register_map.md](register_map.md) — Register map definition
- [hal/codegen.md](codegen.md) — Multi-language code generation
- [../rtl/config_schema.md](../rtl/config_schema.md) — rtl.toml schema