# apps-conductor CLI Binary Specification

**Target Conductor**: apps-conductor
**Source**: Design specification §15 (around lines 3631-3730), §9 (around lines 2281-2400)

## Binary Name

`hestia-apps-cli`

## Subcommands

| Subcommand | Description |
|-------------|-----------|
| `init` | Generate apps.toml template |
| `build` | Execute cross-compilation build |
| `flash` | Write firmware to target device |
| `test sil` | Run SIL (Software-in-the-Loop) test (QEMU) |
| `test hil` | Run HIL (Hardware-in-the-Loop) test (physical device + debug-conductor) |
| `test qemu` | Run QEMU test |
| `size` | Display binary size report |
| `debug` | Start debug session (bridging with debug-conductor §10) |
| `status` | Display build state and job status |

## Common Options (CommonOpts)

| Option | Value | Description |
|-----------|---|-----------|
| `--output` | `human` \| `json` | Output format (default: human) |
| `--timeout` | `<seconds>` | RPC timeout |
| `--registry` | `<path>` | agent-cli registry path |
| `--config` | `<path>` | Configuration file path |
| `--verbose` | — | Verbose log output |

## Exit Codes

| Exit Code | Meaning |
|-----------|--------|
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
hestia apps init

# Build
hestia apps build

# QEMU test
hestia apps test qemu

# HIL test
hestia apps test hil

# Flash write
hestia apps flash

# Size report
hestia apps size
```

## CLI Architecture

Rust-based client binary (`tokio` + `serde` + `clap`). Connects to the apps-conductor agent-cli peer (peer name `apps`) via agent-cli native IPC.

## Related Documentation

- [apps/config_schema.md](config_schema.md) — apps.toml configuration schema
- [apps/message_methods.md](message_methods.md) — apps.* method list
- [apps/toolchain.md](toolchain.md) — Main adapters
- [apps/rtos.md](rtos.md) — RTOS support
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL testing