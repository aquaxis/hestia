# debug-conductor Debug Protocols

**Target Conductor**: debug-conductor
**Source**: Design Specification §10.5 (around lines 2495-2528)

## JTAG TAP State Machine (§10.5)

adapter-jtag implements a TAP (Test Access Port) state machine compliant with IEEE 1149.1. State transitions are controlled by the TMS signal.

### TapState Enumeration

```rust
pub enum TapState {
    TestLogicReset, RunTestIdle,
    SelectDR, CaptureDR, ShiftDR, Exit1DR, PauseDR, Exit2DR, UpdateDR,
    SelectIR, CaptureIR, ShiftIR, Exit1IR, PauseIR, Exit2IR, UpdateIR,
}
```

### TAP State Machine Transitions

A 16-state finite automaton controlled by TMS=0 / TMS=1.

- `TestLogicReset` is always reachable by holding TMS=1 for 5 consecutive clock cycles (reset state)
- `RunTestIdle` is the idle state
- DR path: `SelectDR → CaptureDR → ShiftDR → Exit1DR → PauseDR → Exit2DR → UpdateDR`
- IR path: `SelectIR → CaptureIR → ShiftIR → Exit1IR → PauseIR → Exit2IR → UpdateIR`

## SWD Protocol (§10.6)

adapter-swd implements ARM Serial Wire Debug (2-wire: SWCLK / SWDIO).

### Request Types

| Request Type | Description | Target Registers |
|---------------|------|------------|
| `ReadDP` | Debug Port register read | DPIDR, CTRL/STAT, SELECT, etc. |
| `WriteDP` | Debug Port register write | SELECT, ABORT, etc. |
| `ReadAP` | Access Port register read | CSW, TAR, DRW, etc. |
| `WriteAP` | Access Port register write | CSW, TAR, DRW, etc. |

### SWD Packet Structure

```
[Start] [APnDP] [RnW] [Addr(2bit)] [Parity] [Stop] [Park] → [Trn] → [Data(32bit)] [Parity]
```

### SWD Notable Points

- Implements 2-wire (SWCLK/SWDIO) compared to JTAG (4-wire: TCK/TMS/TDI/TDO)
- Standard debug interface for ARM Cortex-M processors
- Supported by OpenOCD / pyOCD

## Protocol Decoders (§10.7)

debug-conductor includes the following built-in protocol decoders (sigrok / PulseView integration).

| Protocol | Decode Target | Configuration Parameters |
|-----------|------------|--------------|
| UART | Baud rate auto-detection, 8N1/7E1 and other frame settings | Baud rate, data bits, parity, stop bits |
| SPI | Mode 0-3, CPOL/CPHA settings | Clock polarity/phase, CS polarity |
| I2C | 7-bit/10-bit address, ACK/NACK analysis | Address mode |
| CAN | Standard/Extended ID, DLC, data field | Bit rate |
| LIN | Break/Sync/PID/Data/Checksum analysis | Baud rate |

## On-Chip Debug Integration

| ILA Type | Vendor | Connection Method |
|---------|---------|---------|
| Xilinx ILA | AMD/Xilinx | Via Vivado hw_server |
| Intel SignalTap | Intel/Altera | Via Quartus Signal Tap |
| Lattice Reveal | Lattice | Via Radiant Reveal |

## Waveform Formats

| Format | Description |
|------------|------|
| VCD | Value Change Dump (standard waveform format) |
| FST | Fast Signal Trace (compressed waveform format) |
| WASM | WASM-based waveform viewer (1 million samples, 60fps) |

## Related Documentation

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli binary specification
- [debug/state_machines.md](state_machines.md) — Session management state machine
- [debug/message_methods.md](message_methods.md) — debug.* method list
- [debug/error_types.md](error_types.md) — debug-conductor error codes