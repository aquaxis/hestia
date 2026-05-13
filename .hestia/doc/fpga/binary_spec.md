# fpga-conductor CLI Binary Specification

**Target Conductor**: fpga-conductor
**Source**: Design Specification §15 (around lines 3631-3730), §5 (around lines 1398-1760)

## Binary Name

`hestia-fpga-cli`

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `init` | Generate fpga.toml template |
| `build <target>` | Full build for specified target (synthesis → place-and-route → bitstream) |
| `synthesize` | Run synthesis only |
| `implement` | Run place-and-route only |
| `bitstream` | Generate bitstream only |
| `simulate` | Run simulation |
| `program` | Write bitstream to FPGA |
| `report timing` | Show timing report |
| `report resource` | Show resource utilization report |
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
hestia fpga init

# Full build
hestia fpga build artix7

# Synthesis only
hestia fpga synthesize

# Timing report
hestia fpga report timing --job-id 1

# Program bitstream
hestia fpga program --target artix7_dev
```

## CLI Architecture

Rust client binary (`tokio` + `serde` + `clap`). Connects to the fpga-conductor agent-cli peer (peer name `fpga`) via agent-cli native IPC.

## Related Documentation

- [fpga/config_schema.md](config_schema.md) — fpga.toml configuration schema
- [fpga/message_methods.md](message_methods.md) — fpga.* method list
- [fpga/state_machines.md](state_machines.md) — Build state machine
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter trait