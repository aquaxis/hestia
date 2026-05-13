# asic-conductor CLI Binary Specification

**Target Conductor**: asic-conductor
**Source**: Design specification §15 (around lines 3631-3730), §6 (around lines 1761-1981)

## Binary Name

`hestia-asic-cli`

## Subcommands

| Subcommand | Description |
|-------------|------------|
| `init` | Generate asic.toml template |
| `build` | Execute RTL-to-GDSII full build |
| `pdk install <pdk>` | Install PDK (Sky130 / GF180MCU / IHP SG13G2) |
| `pdk list` | Display installed PDK list |
| `advance` | Advance build to next step (integrates with OpenLane 2 Step-based Execution) |
| `drc` | Run DRC (Design Rule Check) |
| `lvs` | Run LVS (Layout Versus Schematic verification) |
| `status` | Display build state and job status |

## Common Options (CommonOpts)

| Option | Value | Description |
|-----------|---|------------|
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
hestia asic init

# PDK install
hestia asic pdk install sky130_fd_sc_hd

# Full build
hestia asic build

# DRC only
hestia asic drc

# Resume from specific step
hestia asic advance --from placement
```

## CLI Architecture

Rust-based client binary (`tokio` + `serde` + `clap`). Connects to the asic-conductor agent-cli peer (peer name `asic`) via agent-cli native IPC. OpenLane 2 runs inside a Podman container.

## Related Documentation

- [asic/config_schema.md](config_schema.md) — asic.toml configuration schema
- [asic/message_methods.md](message_methods.md) — asic.* method list
- [asic/state_machines.md](state_machines.md) — ASIC build state machine
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter trait