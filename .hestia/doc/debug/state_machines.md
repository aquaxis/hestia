# debug-conductor Session Management State Machine

**Target Conductor**: debug-conductor
**Source**: Design Specification §10.3 (around lines 2450-2475)

## Session Management State Machine

```
Idle → Connecting → Connected → Running → Paused → Capturing → Disconnected
                                                                     │
                                                               Error ←─ Any state
```

## State Definitions

| State | Description |
|------|------|
| Idle | Session not started |
| Connecting | Connecting to debug probe (JTAG / SWD) |
| Connected | Connection established, accepting commands |
| Running | Target executing |
| Paused | Target paused (breakpoint hit / manual pause) |
| Capturing | Waveform capture in progress |
| Disconnected | Disconnected |
| Error | Error occurred (can transition from any state) |

## State Transition Rules

| Transition | Trigger | Description |
|------|---------|------|
| Idle → Connecting | `debug.connect` | Probe connection initiated |
| Connecting → Connected | Connection successful | JTAG / SWD connection established |
| Connecting → Error | Connection failed | Probe not detected, driver error, etc. |
| Connected → Running | `debug.run` | Target execution started |
| Connected → Disconnected | `debug.disconnect` | Disconnected |
| Running → Paused | `debug.pause` / `breakpointHit` | Paused |
| Running → Capturing | `debug.startCapture` | Capture started |
| Paused → Running | `debug.run` / `debug.stepOver` / `debug.stepInto` | Resumed |
| Capturing → Running | `debug.stopCapture` | Capture completed |
| Any → Error | Error occurred | Probe disconnection, communication error, etc. |
| Error → Idle | Recovery | Session recreated |
| Disconnected → Idle | Resource release | — |

## Debug Flow

```
Debug session start
    │
    ▼ Debug probe detection
    │  ├── JTAG: via OpenOCD (USB device access)
    │  └── SWD: via OpenOCD / pyOCD
    │
    ├─── On-chip debug
    │    ├── ILA (Xilinx) — via Vivado hw_server
    │    ├── SignalTap (Intel) — via Quartus Signal Tap
    │    └── Reveal (Lattice) — via Radiant Reveal
    │
    ├─── Logic analyzer
    │    ├── sigrok — Generic logic analyzer framework
    │    └── PulseView — GUI waveform viewer
    │
    └─── Waveform capture, display, and analysis
         ├── Waveform saving in VCD / FST format
         └── WASM-based waveform viewer (1 million samples, 60fps)
```

## ResetType

```rust
pub enum ResetType {
    Hardware,  // Hardware reset (using SRST/TRST pins)
    Software,  // Software reset (register write)
    System,    // System reset (entire processor)
}
```

## Related Documentation

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli binary specification
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD protocols
- [debug/message_methods.md](message_methods.md) — debug.* method list
- [debug/error_types.md](error_types.md) — debug-conductor error codes