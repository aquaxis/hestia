# hal-conductor Register Map Definition

**Target Conductor**: hal-conductor
**Source**: Design specification §8 (around lines 2175-2280)

## Overview

hal-conductor reads register definitions from SystemRDL / IP-XACT / custom TOML schemas and converts them into an internal representation (RegisterMap). Multi-language code generation, validation, and diff display are performed based on this RegisterMap.

## Input Formats

| Format | Identifier | Description |
|--------|------------|-------------|
| SystemRDL | `systemrdl` | IEEE 1685 compliant register description language |
| IP-XACT | `ipxact` | IEEE 1685 XML-based IP description format |
| TOML | `toml` | Hestia custom TOML-based definition |

## HalToolAdapter Trait

```rust
#[async_trait]
pub trait HalToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_inputs(&self) -> &[RegisterFormat];   // SystemRDL / IP-XACT / TOML
    fn supported_outputs(&self) -> &[OutputLang];      // C / Rust / Python / Markdown / SVD

    async fn parse(&self, src: &Path) -> Result<RegisterMap, AdapterError>;
    async fn validate(&self, map: &RegisterMap) -> Result<ValidationReport, AdapterError>;
    async fn generate(&self, map: &RegisterMap, target: OutputLang, out: &Path)
        -> Result<PathBuf, AdapterError>;
}
```

## RegisterMap Model

Internal representation of a register map (defined in `register_map.rs`).

### Core Data Structures

- **RegisterMap**: Collection of register blocks (base address, bus protocol)
- **RegisterBlock**: Group of registers (offset, size)
- **Register**: Individual register (address, width, access rights, reset value)
- **RegisterField**: Bit field (bit width, offset, access rights, enum values)

### Validation Checks

| Check Item | Description |
|------------|-------------|
| Address overlap | Whether address ranges of multiple registers overlap |
| Bus boundary | Whether registers are aligned to bus width boundaries (e.g., 32-bit) |
| Type consistency | Whether field width exceeds register width |
| Access rights | Whether RW/RO/WO/RESERVED combinations are contradictory |
| Memory map | Consistency with memory map definitions |

## Memory Map Management

`memory_map.rs` manages address spaces and detects overlaps. It verifies the placement of multiple register blocks and maintains address space consistency.

## Bus Protocol Definitions

Supported bus protocols are defined in `bus_protocol.rs`.

| Protocol | Identifier | Description |
|----------|------------|-------------|
| AXI4-Lite | `axi4-lite` | Lightweight AXI for low-throughput applications |
| AXI4 | `axi4` | High-performance memory-mapped interface |
| Wishbone B4 | `wishbone-b4` | Open-source SoC bus |
| AHB-Lite | `ahb-lite` | ARM high-bandwidth bus |

## Major Adapters

| Adapter | Role | Input | Output |
|---------|------|-------|--------|
| `peakrdl` | SystemRDL multi-language generation | SystemRDL | C / Markdown / HTML |
| `peakrdl-rust` | Rust driver generation | SystemRDL | Rust (embedded-hal compatible) |
| `ipyxact` | IP-XACT parsing | IP-XACT XML | Internal register model |
| `csr2regs` | CSR register generation | TOML / YAML | C / SystemVerilog |
| `cmsis-svd-gen` | CMSIS SVD generation | Internal model | SVD XML |
| `svd2rust-bridge` | SVD → Rust crate | SVD | Rust (svd2rust compatible) |

## Related Documentation

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli binary specification
- [hal/codegen.md](codegen.md) — Multi-language code generation
- [hal/config_schema.md](config_schema.md) — hal.toml schema
- [hal/state_machines.md](state_machines.md) — Build state machine