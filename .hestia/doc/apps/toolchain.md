# apps-conductor Main Toolchain Adapters

**Target Conductor**: apps-conductor
**Source**: Design specification §9.5 (around lines 2356-2369)

## AppsToolAdapter Trait

```rust
#[async_trait]
pub trait AppsToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn target_arch(&self) -> &[TargetArch];     // arm-cortex-m / riscv32imac / xtensa-esp32
    fn supported_languages(&self) -> &[AppLanguage];  // C / C++ / Rust

    async fn build(&self, project: &AppProject) -> Result<Artifact, AdapterError>;
    async fn flash(&self, artifact: &Artifact, target: &Target) -> Result<(), AdapterError>;
    async fn test(&self, project: &AppProject, mode: TestMode) -> Result<TestReport, AdapterError>;
    async fn size_report(&self, artifact: &Artifact) -> Result<SizeReport, AdapterError>;
}
```

## Main Adapter List

### Cross-Compilers

| Adapter | Role | Target Language | Target Architecture |
|----------|------|----------------|--------------------|
| `arm-gcc` | ARM Cortex-M build | C / C++ | arm-cortex-m |
| `riscv-gcc` | RISC-V build | C / C++ | riscv32imac |
| `cargo-embed` | Rust embedded build | Rust | arm / riscv |
| `cargo-binutils` | Binary size analysis | Rust | All architectures |

### RTOS Build

| Adapter | Role | Target Language | Target RTOS |
|----------|------|----------------|------------|
| `west-zephyr` | Zephyr RTOS build | C | Zephyr |
| `freertos-builder` | FreeRTOS integration | C / C++ | FreeRTOS |
| `embassy-builder` | embassy-rs (async Rust) build | Rust | Embassy |

### Testing and Debugging

| Adapter | Role | Target Language | Description |
|----------|------|----------------|-------------|
| `qemu-system` | QEMU SIL testing | C / C++ / Rust | Emulation-based testing |
| `probe-rs` | Flash writing / RTT logging | Rust | Probe tool from the Rust ecosystem |
| `openocd-bridge` | OpenOCD integration | C / C++ | Used via debug-conductor |

## Crate Structure

```
crates/apps-conductor/
├── src/
│   ├── lib.rs              # Conductor main body
│   ├── adapter.rs          # AppsToolAdapter trait
│   ├── toolchain.rs        # Cross-compiler management
│   ├── rtos.rs             # RTOS integration
│   ├── linker.rs           # Linker script management
│   ├── target.rs           # Target definition
│   ├── hil.rs              # HIL / SIL test integration
│   └── fsm_states.rs       # Build state machine
```

## TargetArch Support List

| Architecture | Identifier | Description |
|-------------|-----------|-------------|
| ARM Cortex-M | `arm-cortex-m` | STM32 / nRF / LPC, etc. |
| RISC-V 32bit | `riscv32imac` | SiFive / ESP32-C, etc. |
| Xtensa ESP32 | `xtensa-esp32` | ESP32 / ESP32-S2 / S3 |

## Related Documentation

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli binary specification
- [apps/rtos.md](rtos.md) — RTOS support
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL testing
- [apps/config_schema.md](config_schema.md) — apps.toml schema