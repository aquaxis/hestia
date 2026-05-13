# WASM Waveform Viewer

**Domain**: common — Waveform Display
**Source**: Design Specification §13.2

## Overview

A waveform viewer capable of streaming-parse VCD / FST / GHW / EVCD formats. The `waveform-core` crate is built as both `cdylib` and `rlib`; browsers load it via WebWorker + SharedArrayBuffer, while Tauri / VSCode WebView uses the same crate directly. The target is 60fps at 1 million sample display.

## Supported Formats

| Format | Identifier | Description |
|------------|-------|------|
| VCD | `Vcd` | Value Change Dump (standard)|
| FST | `Fst` | Fast Signal Trace (compressed)|
| GHW | `Ghw` | GHDL Waveform |
| EVCD | `Evcd` | Extended VCD |

## Key Types

### WaveformFormat

```rust
pub enum WaveformFormat {
    Vcd,
    Fst,
    Ghw,
    Evcd,
}
```

### Signal

```rust
pub struct Signal {
    pub id: String,
    pub full_name: String,
    pub display_name: String,
    pub bit_width: u32,
    pub signal_type: SignalType,
    pub scope: String,
}

pub enum SignalType {
    Wire,
    Reg,
    Integer,
    Real,
}
```

### SignalValue

```rust
pub enum SignalValue {
    Logic(char),              // '0' / '1' / 'X' / 'Z'
    Vector { bits: String, hex: String },
    Real(f64),
    String(String),
}
```

## Build Configuration

The `waveform-core` crate is built with two crate types:

| crate-type | Purpose |
|-----------|------|
| `cdylib` | WASM compilation (for browsers)|
| `rlib` | Rust library (for Tauri / VSCode WebView)|

## Rendering Paths

### Browser Path

```
waveform-core (cdylib -> WASM)
  -> Loaded in WebWorker
  -> Shared with main thread via SharedArrayBuffer
  -> 60fps rendering via Canvas / WebGL
```

### Tauri / VSCode WebView Path

```
waveform-core (rlib)
  -> Directly linked
  -> Native rendering
```

## Performance Targets

| Metric | Target |
|------|-------|
| Display sample count | 1 million samples |
| Frame rate | 60fps |
| Streaming parse | Incremental reading without loading entire file into memory |

## Integration Targets

- VSCode extension: WASM rendering within WebView (§16.1)
- Tauri IDE: Native rendering (§16.2)
- debug-conductor: Waveform data provider

## Crate Structure

```
waveform-core/
├── Cargo.toml         # crate-type = ["cdylib", "rlib"]
└── src/
    ├── lib.rs          # Public API
    ├── vcd.rs          # VCD parser
    ├── fst.rs          # FST parser
    ├── ghw.rs          # GHW parser
    ├── evcd.rs         # EVCD parser
    └── render.rs       # Rendering abstraction
```

## Related Documents

- [hdl_lsp_broker.md](hdl_lsp_broker.md) — HDL LSP Broker
- [observability.md](observability.md) — Monitoring