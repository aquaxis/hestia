# debug-conductor CLI Binary Specification

**Target Conductor**: debug-conductor
**Source**: Design Specification §15 (around lines 3631-3730), §10 (around lines 2401-2550)

## Binary Name

`hestia-debug-cli`

## Subcommand List

| Subcommand | Description |
|-------------|------|
| `create` | Create debug session |
| `connect` | Connect to target device (JTAG / SWD) |
| `disconnect` | Disconnect from target device |
| `program` | Firmware programming (SVF / JAM / probe-rs / OpenOCD) |
| `capture start` | Start waveform capture |
| `capture stop` | Stop waveform capture |
| `signals read` | Read signals |
| `trigger set` | Set trigger condition |
| `reset` | Target reset (Hardware / Software / System) |
| `status` | Display session status and connection state |

## Common Options (CommonOpts)

| Option | Value | Description |
|-----------|---|------|
| `--output` | `human` \| `json` | Output format (default: human) |
| `--timeout` | `<seconds>` | RPC timeout |
| `--registry` | `<path>` | agent-cli registry path |
| `--config` | `<path>` | Configuration file path |
| `--verbose` | — | Verbose log output |

## Exit Codes

| Exit Code | Meaning |
|-----------|------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## Local Only

debug-conductor is **local only** (USB probe access, §2.2). Execution inside containers is not supported due to USB device access constraints.

## CLI Usage Examples

```bash
# Session creation and connection
hestia debug create
hestia debug connect --probe stlink-v3

# Firmware programming
hestia debug program --firmware build/sensor_node_fw.bin

# Waveform capture
hestia debug capture start --signals "clk,data"
hestia debug capture stop

# Reset
hestia debug reset --type hardware
```

## Related Documentation

- [debug/config_schema.md](config_schema.md) — debug-conductor configuration
- [debug/message_methods.md](message_methods.md) — debug.* method list
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD protocols
- [debug/state_machines.md](state_machines.md) — Session management state machine