# rtl-conductor CLI Binary Specification

**Target Conductor**: rtl-conductor
**Source**: Design Specification §15 (around lines 3631-3730), §4 (around lines 1241-1397)

## Binary Name

`hestia-rtl-cli`

## Subcommands

| Subcommand | Description |
|------------|-------------|
| `init` | Generate rtl.toml template |
| `lint` | HDL source Lint / format / static analysis (Verilator/Verible, etc.) |
| `simulate` | Run simulation (`--tb <testbench>` / `--simulator verilator`) |
| `formal` | Run formal verification (SymbiYosys, `--properties <file>`) |
| `transpile` | Transpile between HDL languages (Chisel/SpinalHDL/Amaranth → Verilog) |
| `handoff` | Handoff to downstream conductor (`--target fpga` / `--target asic`) |
| `status` | Display build status and job information |

## Common Options (CommonOpts)

| Option | Value | Description |
|--------|-------|-------------|
| `--output` | `human` \| `json` | Output format (default: human) |
| `--timeout` | `<seconds>` | RPC timeout |
| `--registry` | `<path>` | agent-cli registry path |
| `--config` | `<path>` | Configuration file path |
| `--verbose` | — | Verbose logging |

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
hestia rtl init

# Run lint
hestia rtl lint

# Simulation
hestia rtl simulate --tb tb_alu --simulator verilator

# Formal verification
hestia rtl formal --properties properties.sv

# Downstream handoff
hestia rtl handoff --target fpga
```

## CLI Architecture

Rust-based client binary (`tokio` + `serde` + `clap`). Connects to the rtl-conductor agent-cli peer (peer name `rtl`) via agent-cli native IPC.

## Related Documentation

- [rtl/config_schema.md](config_schema.md) — rtl.toml configuration schema
- [rtl/message_methods.md](message_methods.md) — rtl.* method list
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter trait
- [rtl/handoff.md](handoff.md) — Downstream handoff
- [../ai/binary_spec.md](../ai/binary_spec.md) — Unified CLI specification