# hal-conductor Multi-language Code Generation

**Target Conductor**: hal-conductor
**Source**: Design specification §8 (around lines 2175-2280)

## Overview

hal-conductor auto-generates driver skeletons, register access APIs, and memory map definitions from a RegisterMap in multiple languages. Implemented in `codegen.rs`.

## Supported Output Languages

| Output Language | Identifier | Description |
|----------------|------------|-------------|
| C | `c` | C header file (register access macros, struct definitions) |
| Rust | `rust` | Rust crate (embedded-hal compatible driver) |
| Python | `python` | Python module (MMIO access wrapper) |
| Markdown | `markdown` | Register documentation |
| SVD | `svd` | CMSIS SVD XML (for debugger/IDE integration) |

## C Header Generation

Generated content:
- Register base address macros (`#define SOC_BASE 0x10000000`)
- Register offset macros (`#define REG_CTRL_OFFSET 0x00`)
- Register struct definitions (with bitfield support)
- Read/write helper macros (`REG_READ` / `REG_WRITE`)

Output path: specified in hal.toml `[outputs] c_header`

## Rust Crate Generation

Generated content:
- `embedded-hal` trait-compatible driver structs
- MMIO register access (`read()` / `write()` / `modify()`)
- Type-safe bitfield operations
- PAC (Peripheral Access Crate) format

Output path: specified in hal.toml `[outputs] rust_crate`

Related adapters:
- `peakrdl-rust`: SystemRDL → Rust (embedded-hal compatible)
- `svd2rust-bridge`: SVD → Rust (svd2rust compatible)

## Python Module Generation

Generated content:
- MMIO access class (`/dev/mem` or UIO-based)
- Register field properties (`@property` decorators)
- Enum mappings

Output path: specified in hal.toml `[outputs] python_module`

## SVD Generation

CMSIS SVD (System View Description) XML format. Used by debuggers and IDEs to display register information.

Generated content:
- `<peripheral>` elements (base address, size)
- `<register>` elements (offset, access rights, reset values)
- `<field>` elements (bit width, offset, enum values)

Output path: specified in hal.toml `[outputs] svd`

Related adapters:
- `cmsis-svd-gen`: Internal model → SVD XML

## Markdown Documentation Generation

Generated content:
- Register block overview
- Register table (address, name, access rights, reset values)
- Bitfield diagrams

Output path: specified in hal.toml `[outputs] documentation`

## Parallel Code Generation

When multiple output languages are specified, coder sub-agents (`hal-coder-c` / `hal-coder-rust` / `hal-coder-python` / `hal-coder-svd`) launch in parallel for each language and generate code simultaneously.

## Downstream Integration

### apps-conductor (§9)

Generated C headers / Rust crates / Python modules are imported via apps-conductor's `[hal] import = "..."`.

### debug-conductor (§10)

SVD files are reused by debug-conductor for live debugging register display and editing UI.

### asic-conductor / fpga-conductor

SystemVerilog templates exported via `export-rtl` can be passed directly to the corresponding conductor's `[sources]`.

## Related Documentation

- [hal/register_map.md](register_map.md) — Register map definition
- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli binary specification
- [hal/config_schema.md](config_schema.md) — hal.toml [outputs] section
- [../apps/config_schema.md](../apps/config_schema.md) — apps.toml [hal] section