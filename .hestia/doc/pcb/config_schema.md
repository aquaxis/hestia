# pcb-conductor Configuration Schema

**Target Conductor**: pcb-conductor
**Source**: Design specification §7.6 (around lines 2157-...)

## pcb.toml — Unified Project Format

A file that declaratively defines PCB project settings, board definitions, layer configuration, AI design settings, and output settings.

### Section List

| Section | Required | Description |
|---------|----------|-------------|
| `[project]` | Required | Project basic settings |
| `[board]` | Required | Board dimensions and layer count |
| `[[layers]]` | Required | Layer definitions (signal/power/GND) |
| `[design]` | Optional | AI design settings |
| `[output]` | Optional | Output settings |

### `[project]` Section

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `version` | string | Version |
| `board_name` | string | Board name |

### `[board]` Section

| Field | Type | Description |
|-------|------|-------------|
| `layer_count` | integer | Number of layers |
| `width_mm` | float | Board width (mm) |
| `height_mm` | float | Board height (mm) |

### `[[layers]]` Section

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Layer name (e.g., `F.Cu`, `In1.Cu`, `B.Cu`) |
| `type` | string | Layer type (`signal` / `power` / `ground`) |

### `[design]` Section

| Field | Type | Description |
|-------|------|-------------|
| `input_format` | string | Input format (`natural_language` / `skidl` / `kicad_sch`) |
| `ai_enabled` | boolean | Enable AI-driven schematic design |

### `[output]` Section

| Field | Type | Description |
|-------|------|-------------|
| `format` | string | Output format (`kicad` / `altium` / `gerber`) |
| `output_dir` | string | Output directory |

### Configuration Example

```toml
[project]
name = "my-pcb-project"
version = "0.1.0"
board_name = "motor_controller"

[board]
layer_count = 4
width_mm = 100
height_mm = 80

[[layers]]
name = "F.Cu"
type = "signal"

[[layers]]
name = "In1.Cu"
type = "power"

[[layers]]
name = "In2.Cu"
type = "ground"

[[layers]]
name = "B.Cu"
type = "signal"

[design]
input_format = "natural_language"
ai_enabled = true

[output]
format = "kicad"
output_dir = "output/"
```

## Related Documentation

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli binary specification
- [pcb/state_machines.md](state_machines.md) — PCB build steps
- [pcb/tool_adapter.md](tool_adapter.md) — AI-driven schematic design / KiCad adapter
- [../fpga/config_schema.md](../fpga/config_schema.md) — fpga.toml schema