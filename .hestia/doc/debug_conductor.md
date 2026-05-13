# Debug Environment Orchestrator

**Domain**: debug-conductor
**Source**: Design Specification §10 (around lines 2401-2550)

---

## Overview

debug-conductor is an orchestrator that integrates and manages hardware debug environments. It provides unified management of target connections via JTAG / SWD protocols, on-chip debug (ILA / SignalTap / Reveal), logic analyzers (sigrok), and waveform capture (VCD / FST), delivering a session-based debug workflow. debug-conductor is **local only** (USB probe access required), and all sub-agents also run locally.

---

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

---

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

---

## Session Management State Machine

```
Idle → Connecting → Connected → Running → Paused → Capturing → Disconnected
                                                                     │
                                                               Error ←─ Any state
```

| State | Description |
|------|------|
| Idle | Session not started |
| Connecting | Connecting to debug probe |
| Connected | Connection established, accepting commands |
| Running | Target executing |
| Paused | Target paused |
| Capturing | Waveform capture in progress |
| Disconnected | Disconnected |
| Error | Error occurred |

```rust
pub enum ResetType {
    Hardware,  // Hardware reset (using SRST/TRST pins)
    Software,  // Software reset (register write)
    System,    // System reset (entire processor)
}
```

---

## Published Methods

| Method | Direction | Description |
|---------|------|------|
| `connect` | Request | Connect to target device |
| `disconnect` | Request | Disconnect from target device |
| `reset` | Request | Reset target device |
| `setBreakpoint` | Request | Set breakpoint (Source/Address/Symbol 3 modes) |
| `removeBreakpoint` | Request | Remove breakpoint |
| `run` | Request | Execute target program |
| `pause` | Request | Pause |
| `stepOver` / `stepInto` | Request | Step execution |
| `readMemory` / `writeMemory` | Request | Memory read/write |
| `startCapture` / `stopCapture` | Request | Start/stop waveform capture |
| `sessionStateChanged` | Notification | Session state change notification |
| `breakpointHit` | Notification | Breakpoint hit notification |
| `captureComplete` | Notification | Capture complete notification |

---

## JTAG TAP State Machine

adapter-jtag implements a TAP state machine compliant with IEEE 1149.1. State transitions are controlled by the TMS signal.

```rust
pub enum TapState {
    TestLogicReset, RunTestIdle,
    SelectDR, CaptureDR, ShiftDR, Exit1DR, PauseDR, Exit2DR, UpdateDR,
    SelectIR, CaptureIR, ShiftIR, Exit1IR, PauseIR, Exit2IR, UpdateIR,
}
```

---

## SWD Protocol

adapter-swd implements ARM Serial Wire Debug (2-wire: SWCLK / SWDIO).

| Request Type | Description | Target |
|---------------|------|------|
| `ReadDP` | Debug Port register read | DPIDR, CTRL/STAT, SELECT, etc. |
| `WriteDP` | Debug Port register write | SELECT, ABORT, etc. |
| `ReadAP` | Access Port register read | CSW, TAR, DRW, etc. |
| `WriteAP` | Access Port register write | CSW, TAR, DRW, etc. |

---

## Protocol Decoders

debug-conductor includes the following built-in protocol decoders.

| Protocol | Decode Target |
|-----------|------------|
| UART | Baud rate auto-detection, 8N1/7E1 and other frame settings |
| SPI | Mode 0-3, CPOL/CPHA settings |
| I2C | 7-bit/10-bit address, ACK/NACK analysis |
| CAN | Standard/Extended ID, DLC, data field |
| LIN | Break/Sync/PID/Data/Checksum analysis |

---

## Sub-Agent Structure

debug-conductor has 5 sub-agent types: **planner / designer / session_manager / analyzer / programmer**, which share debug session management, waveform analysis, and firmware programming. Each sub-agent is launched as an independent agent-cli process and coordinates with the debug-conductor main body (peer name `debug`) via `agent-cli send <peer>` IPC. Since debug-conductor is **local only** (USB probe access), all sub-agents also run locally.

| Sub-Agent | Peer Name | Role | Multiplicity |
|----------------|---------|------|-------|
| **planner** | `debug-planner` | Debug planning (test point selection, trigger conditions, capture depth) | 1 |
| **designer** | `debug-designer` | Verification scenario specification (signal definition, state transition verification items, expected waveforms) | 1 |
| **session_manager** | `debug-session` | Debug session management (JTAG / SWD connection, OpenOCD/pyOCD control, breakpoint/watchpoint setup) | 1 (can be parallel per target) |
| **analyzer** | `debug-analyzer` | Waveform analysis / protocol decode / logic analyzer aggregation (sigrok/PulseView, ILA/SignalTap/Reveal) | 1 |
| **programmer** | `debug-programmer` | Firmware programming (probe-rs, OpenOCD, SVF / JAM) | 1 |

**Flow**: planner → designer → session_manager (connection) → programmer (programming) → analyzer (execution + analysis).

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator
- [apps_conductor.md](apps_conductor.md) — Application software development orchestrator