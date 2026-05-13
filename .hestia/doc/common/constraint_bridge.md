# Constraint Bridge

**Domain**: common — Constraint Conversion
**Source**: Design Specification §13.3

## Overview

A constraint file cross-conversion engine. Using `ConstraintModel` as an intermediate representation, conversion between N format types is possible. This reduces the traditional N x N conversion approach to N + M parsers/generators.

## Supported Formats

| Format | Target | Extension |
|------------|------|--------|
| XDC | Xilinx (Vivado) | `.xdc` |
| PCF | iCE40 (nextpnr) | `.pcf` |
| SDC | Synopsys (OpenSTA / Vivado) | `.sdc` |
| Efinity XML | Efinix (Efinity) | `.xml` |
| QSF | Intel (Quartus) | `.qsf` |
| UCF | Legacy Xilinx (ISE) | `.ucf` |

## Intermediate Representation: ConstraintModel

### ConstraintFormat

```rust
pub enum ConstraintFormat {
    Xdc,
    Pcf,
    Sdc,
    // Others are extensible
}
```

### Key Structures

```rust
pub struct ClockConstraint {
    pub name: String,
    pub period_ns: f64,
    pub waveform: Option<String>,
    pub target_pins: Vec<String>,
}

pub struct PinConstraint {
    pub port_name: String,
    pub pin_id: String,
    pub io_standard: Option<String>,
    pub drive_strength: Option<String>,
    pub slew_rate: Option<String>,
    pub differential_pair: Option<String>,
}

pub struct TimingConstraint {
    pub kind: TimingKind,
    pub from_clock: String,
    pub to_clock: String,
    pub delay_ns: f64,
}

pub struct PlacementConstraint {
    pub instance: String,
    pub site: String,
}

pub struct RawConstraint {
    pub format: ConstraintFormat,
    pub text: String,
}
```

## Conversion Flow

```
Input format (XDC / PCF / SDC / ...) -> Parser -> ConstraintModel -> Generator -> Output format
```

- N format types -> N parsers + N generators = 2N modules
- Reduces the traditional N x N = N^2 conversion functions

## Supported Constraint Coverage

- Pin assignments (PORT -> PIN mapping)
- I/O standards (LVCMOS33 / LVDS, etc.)
- Drive strength (mA specification)
- Slew rate (FAST / SLOW)
- Differential pairs (p/n pair constraints)
- Clock constraints (period / frequency / waveform)
- Multicycle paths
- False paths
- Timing exceptions

## Crate Structure

```
constraint-bridge/
├── Cargo.toml
└── src/
    ├── lib.rs              # ConstraintModel, conversion dispatch
    ├── parsers/
    │   ├── xdc.rs          # XDC parser
    │   ├── pcf.rs          # PCF parser
    │   ├── sdc.rs          # SDC parser
    │   ├── efinity.rs      # Efinity XML parser
    │   ├── qsf.rs          # QSF parser
    │   └── ucf.rs          # UCF parser
    └── generators/
        ├── xdc.rs          # XDC generator
        ├── pcf.rs          # PCF generator
        ├── sdc.rs          # SDC generator
        ├── efinity.rs      # Efinity XML generator
        ├── qsf.rs          # QSF generator
        └── ucf.rs          # UCF generator
```

## Related Documents

- [hdl_lsp_broker.md](hdl_lsp_broker.md) — HDL LSP Broker
- [ip_manager.md](ip_manager.md) — IP Manager
- [observability.md](observability.md) — Monitoring