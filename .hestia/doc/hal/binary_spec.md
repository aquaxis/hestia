# hal-conductor CLI Binary Specification

**Target Conductor**: hal-conductor
**Source**: Design specification §15 (around lines 3631-3730), §8 (around lines 2175-2280)

## Binary Name

`hestia-hal-cli`

## Subcommand List

| Subcommand | Description |
|------------|-------------|
| `init` | Generate hal.toml template |
| `parse` | Parse register definition files (SystemRDL / IP-XACT / TOML) |
| `validate` | Validate register map (address overlap, type consistency, bus boundary checks) |
| `generate c` | Generate C header file |
| `generate rust` | Generate Rust crate (embedded-hal compatible) |
| `generate python` | Generate Python module |
| `generate svd` | Generate CMSIS SVD file |
| `export-rtl` | Export SystemVerilog template output (for rtl/asic/fpga conductor) |
| `diff` | Show register map diff |
| `status` | Show build status and job progress |

## Common Options (CommonOpts)

| Option | Value | Description |
|--------|-------|-------------|
| `--output` | `human` \| `json` | Output format (default: human) |
| `--timeout` | `<seconds>` | RPC timeout |
| `--registry` | `<path>` | agent-cli registry path |
| `--config` | `<path>` | Configuration file path |
| `--verbose` | — | Enable verbose logging |

## Exit Codes

| Exit Code | Meaning |
|-----------|---------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## CLI Usage Examples

```bash
# Initialize
hestia hal init

# Parse register definitions
hestia hal parse

# Validate
hestia hal validate

# Generate C header
hestia hal generate c

# Generate Rust crate
hestia hal generate rust

# Show diff
hestia hal diff --baseline v1.0 --current v1.1
```

## CLI Architecture

Rust-based client binary (`tokio` + `serde` + `clap`). Connects to the hal-conductor agent-cli peer (peer name `hal`) via agent-cli native IPC.

## Related Documentation

- [hal/config_schema.md](config_schema.md) — hal.toml configuration schema
- [hal/message_methods.md](message_methods.md) — hal.* method list
- [hal/register_map.md](register_map.md) — Register map definition
- [hal/codegen.md](codegen.md) — Multi-language code generation