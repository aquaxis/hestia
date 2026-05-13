# debug-conductor Configuration Schema

**Target Conductor**: debug-conductor
**Source**: Design Specification §10 (around lines 2401-2550)

## debug-conductor Configuration

debug-conductor is a local-only conductor that requires access to USB debug probes.

## Crate Structure

```
debug-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona, main.rs
│   ├── project-model/              # debug.toml parser
│   ├── plugin-registry/            # Tool registration and resolution
│   ├── adapter-jtag/               # JTAG debug (OpenOCD integration)
│   ├── adapter-swd/                # SWD debug
│   ├── adapter-ila/                # On-chip debug integration
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── xilinx_ila.rs       # Xilinx ILA
│   │       ├── intel_signaltap.rs  # Intel SignalTap
│   │       └── lattice_reveal.rs   # Lattice Reveal
│   ├── waveform-capture/           # Waveform capture
│   ├── protocol-analyzer/          # Protocol analysis (sigrok integration)
│   └── podman-runtime/             # Container management
├── debug-cli/                      # Rust CLI
└── conductor-sdk/
```

## Probe Configuration

| Setting | Type | Description |
|---------|---|------|
| probe_type | string | Probe type (`stlink` / `jlink` / `cmsis-dap` / `ftdi`) |
| interface | string | Interface (`jtag` / `swd`) |
| clock_speed_khz | integer | Clock speed (kHz) |
| target_device | string | Target device name |

## On-Chip Debug Configuration

| Setting | Type | Description |
|---------|---|------|
| ila_type | string | ILA type (`xilinx_ila` / `intel_signaltap` / `lattice_reveal`) |
| trigger_position | integer | Trigger position |
| sample_depth | integer | Sample depth |
| signals | string[] | Capture signal list |

## Waveform Capture Configuration

| Setting | Type | Description |
|---------|---|------|
| format | string | Waveform format (`vcd` / `fst`) |
| max_samples | integer | Maximum number of samples |
| viewer | string | Viewer (`pulseview` / `wasm`) |

## Reset Types

| Type | Description |
|------|------|
| Hardware | Hardware reset (using SRST/TRST pins) |
| Software | Software reset (register write) |
| System | System reset (entire processor) |

## Related Documentation

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli binary specification
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD protocols
- [debug/state_machines.md](state_machines.md) — Session management state machine
- [../apps/hil_sil.md](../apps/hil_sil.md) — HIL/SIL testing