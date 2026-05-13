# apps-conductor Configuration Schema

**Target Conductor**: apps-conductor
**Source**: Design specification §9.4 (around lines 2325-2354)

## apps.toml — Unified Project Format

A file that declaratively defines the configuration, toolchain, RTOS, memory layout, HAL integration, and test settings for an application firmware project.

### Sections

| Section | Required | Description |
|-----------|----------|------------|
| `[project]` | Required | Project basic configuration |
| `[toolchain]` | Required | Compiler and version specification |
| `[rtos]` | Optional | RTOS kernel and version |
| `[memory]` | Required | Memory layout (Flash / RAM) |
| `[hal]` | Optional | HAL module import |
| `[test]` | Optional | Test mode and probe settings |

### `[project]` Section

| Field | Type | Description |
|-----------|---|------------|
| `name` | string | Project name |
| `language` | string | Language (`c` / `cpp` / `rust`) |
| `target` | string | Target triple (e.g., `thumbv7em-none-eabihf`) |

### `[toolchain]` Section

| Field | Type | Description |
|-----------|---|------------|
| `compiler` | string | Compiler (`arm-none-eabi-gcc` / `riscv32-unknown-elf-gcc` / `cargo`) |
| `version` | string | Version |

### `[rtos]` Section

| Field | Type | Description |
|-----------|---|------------|
| `kernel` | string | RTOS kernel (`freertos` / `zephyr` / `embassy-rs` / `bare-metal`) |
| `version` | string | RTOS version |

### `[memory]` Section

| Field | Type | Description |
|-----------|---|------------|
| `flash_origin` | integer | Flash start address |
| `flash_length` | string | Flash size (e.g., `256K`) |
| `ram_origin` | integer | RAM start address |
| `ram_length` | string | RAM size (e.g., `64K`) |
| `linker_script` | string | Linker script path |

### `[hal]` Section

| Field | Type | Description |
|-----------|---|------------|
| `import` | string | Import path for hal-conductor (§8) output |

### `[test]` Section

| Field | Type | Description |
|-----------|---|------------|
| `mode` | string | Test mode (`sil` / `hil` / `qemu`) |
| `probe` | string | Debug probe (e.g., `stlink-v3`, via debug-conductor §10) |

### Configuration Example

```toml
[project]
name = "sensor_node_fw"
language = "rust"
target = "thumbv7em-none-eabihf"

[toolchain]
compiler = "arm-none-eabi-gcc"
version = "14.2.1"

[rtos]
kernel = "embassy-rs"
version = "0.4"

[memory]
flash_origin  = 0x08000000
flash_length  = "256K"
ram_origin    = 0x20000000
ram_length    = "64K"
linker_script = "memory.x"

[hal]
import = "build/hal/rust/soc-hal"

[test]
mode  = "hil"
probe = "stlink-v3"
```

## Related Documentation

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli binary specification
- [apps/toolchain.md](toolchain.md) — Main adapters
- [apps/rtos.md](rtos.md) — RTOS support
- [../hal/config_schema.md](../hal/config_schema.md) — hal.toml schema