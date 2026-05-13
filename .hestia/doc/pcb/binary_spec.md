# pcb-conductor CLI Binary Specification

**Target Conductor**: pcb-conductor
**Source**: Design specification §15 (around lines 3631-3730), §7 (around lines 1982-2174)

## Binary Name

`hestia-pcb-cli`

## Subcommand List

| Subcommand | Description |
|------------|-------------|
| `init` | Generate pcb.toml template |
| `build` | Execute full PCB build flow (requirements parse → BOM → schematic synthesis → verification → placement → routing → output) |
| `ai-synthesize` | Execute AI-driven schematic synthesis (LLM core) |
| `output kicad` | Output in KiCad format |
| `output gerber` | Output Gerber files |
| `output bom` | Output BOM (bill of materials) |
| `drc` | Run DRC (design rule check) |
| `erc` | Run ERC (electrical rule check) |
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
hestia pcb init

# AI-driven schematic synthesis
hestia pcb ai-synthesize --spec "STM32F103 + BME280 temperature/humidity sensor board"

# Run DRC / ERC
hestia pcb drc
hestia pcb erc

# Gerber output
hestia pcb output gerber
```

## CLI Architecture

Rust-based client binary (`tokio` + `serde` + `clap`). Connects to the pcb-conductor agent-cli peer (peer name `pcb`) via agent-cli native IPC.

## Related Documentation

- [pcb/config_schema.md](config_schema.md) — pcb.toml configuration schema
- [pcb/message_methods.md](message_methods.md) — pcb.* method list
- [pcb/state_machines.md](state_machines.md) — PCB build steps
- [pcb/tool_adapter.md](tool_adapter.md) — AI-driven schematic design / KiCad adapter