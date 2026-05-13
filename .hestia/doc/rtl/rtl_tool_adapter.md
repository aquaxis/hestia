# rtl-conductor RtlToolAdapter Trait

**Target Conductor**: rtl-conductor
**Source**: Design Specification §4.2 (around lines 1262-1278), §4.5 (around lines 1312-1327)

## RtlToolAdapter Trait Definition

A unified interface that all RTL tool adapters must implement.

```rust
#[async_trait]
pub trait RtlToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_languages(&self) -> &[HdlLanguage];   // SystemVerilog / VHDL / Chisel / SpinalHDL / Amaranth
    fn capabilities(&self) -> RtlCapability;            // Lint | Sim | Formal | Transpile

    async fn lint(&self, project: &RtlProject) -> Result<LintReport, AdapterError>;
    async fn simulate(&self, project: &RtlProject, tb: &TestBench) -> Result<SimReport, AdapterError>;
    async fn formal_verify(&self, project: &RtlProject, props: &[Property])
        -> Result<FormalReport, AdapterError>;
    async fn transpile(&self, src: &Path, target_lang: HdlLanguage)
        -> Result<PathBuf, AdapterError>;
}
```

## RtlCapability

Capability flags supported by an adapter.

| Value | Description |
|-------|-------------|
| `Lint` | Lint / format / static analysis |
| `Sim` | Simulation (cycle-accurate / behavioral) |
| `Formal` | Formal verification (property-based) |
| `Transpile` | Transpilation between HDL languages |

## Supported Languages (HdlLanguage)

| Language | Identifier |
|----------|-----------|
| SystemVerilog | `systemverilog` |
| Verilog | `verilog` |
| VHDL | `vhdl` |
| Chisel | `chisel` |
| SpinalHDL | `spinalhdl` |
| Amaranth | `amaranth` |
| MyHDL | `myhdl` |

## Primary Adapter List

| Adapter | Role | Supported Languages | Capability |
|----------|------|---------------------|------------|
| `verilator-lint` | Lint | SystemVerilog / Verilog | Lint |
| `verible` | Format / Lint | SystemVerilog | Lint |
| `verilator` | Cycle-accurate simulation | SystemVerilog / Verilog | Sim |
| `iverilog` | Simulation | Verilog | Sim |
| `ghdl` | VHDL simulation | VHDL | Sim |
| `symbiyosys` | Formal verification (properties) | SystemVerilog | Formal |
| `riscof` | RISC-V ISA compliance | RV32I/M/A/F/D/C | Formal |
| `cocotb` | Python testbench | All languages | Sim |
| `chisel-bridge` | Chisel → Verilog | Chisel | Transpile |
| `spinalhdl-bridge` | SpinalHDL → Verilog | SpinalHDL | Transpile |
| `amaranth-bridge` | Amaranth → Verilog | Amaranth (Python) | Transpile |

## Crate Structure

```
crates/rtl-conductor/
├── src/
│   ├── adapter.rs      # RtlToolAdapter trait
│   ├── language.rs     # HDL language identification / transpilation management
│   ├── lint.rs         # Lint / format / static analysis
│   ├── simulation.rs   # Simulation integration
│   ├── formal.rs       # Formal verification integration
│   ├── repository.rs   # RTL module registry
│   └── handoff.rs      # Handoff to downstream conductors
```

## Related Documentation

- [rtl/config_schema.md](config_schema.md) — rtl.toml schema ([adapters] section)
- [rtl/state_machines.md](state_machines.md) — Build state machine
- [rtl/message_methods.md](message_methods.md) — rtl.* method list
- [../fpga/vendor_adapter.md](../fpga/vendor_adapter.md) — FPGA VendorAdapter trait