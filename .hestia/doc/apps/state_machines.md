# apps-conductor Build State Machine

**Target Conductor**: apps-conductor
**Source**: Design specification §9.3 (around lines 2317-2323)

## 8-State Build State Machine

```
Idle → Resolving → Compiling → Linking → SizeChecking → Flashing → Testing → Done
                                       ↓ (memory exceeded / link error / test failed)
                                   Failed → Diagnosing → Fix suggestion
```

## State Definitions

| State | Description | Main Processing |
|------|-------------|-----------------|
| Idle | Initial state | — |
| Resolving | Resolving toolchain, RTOS, and HAL dependencies | Version verification, path validation |
| Compiling | Executing cross-compilation | Compilation via arm-gcc / riscv-gcc / cargo |
| Linking | Executing linking | Applying linker script, generating binary |
| SizeChecking | Checking binary size | Verifying Flash/RAM usage, determining if within limits |
| Flashing | Writing to flash | Writing to device via probe-rs / OpenOCD |
| Testing | Executing tests | SIL (QEMU) / HIL (physical device) / unit tests |
| Done | Normal completion | Test report and coverage output |
| Failed | Error occurred | Error details + fix suggestion |
| Diagnosing | Analyzing cause | Fix suggestion (memory optimization, linker settings change, etc.) |

## State Transition Rules

| Transition | Trigger | Condition |
|-----------|---------|-----------|
| Idle → Resolving | Build started | — |
| Resolving → Compiling | Dependency resolution complete | All toolchains available |
| Resolving → Failed | Dependency resolution failed | Toolchain not found, etc. |
| Compiling → Linking | Compilation complete | Compilation successful |
| Compiling → Failed | Compilation failed | — |
| Linking → SizeChecking | Linking complete | Linking successful |
| Linking → Failed | Linking failed | Unresolved symbols, etc. |
| SizeChecking → Flashing | Size check passed | Flash/RAM usage within limits |
| SizeChecking → Failed | Size check failed | Memory overflow |
| Flashing → Testing | Flash write complete | — |
| Flashing → Failed | Flash failed | Probe connection error, etc. |
| Testing → Done | Testing complete | All tests passed (or warnings only) |
| Testing → Failed | Testing failed | Test failure |
| Failed → Diagnosing | Diagnosis started | — |

## Test Modes

| Mode | Description | Execution Environment |
|-------|-------------|---------------------|
| SIL | Software-in-the-Loop | QEMU emulation |
| HIL | Hardware-in-the-Loop | Physical device + debug-conductor (§10) |
| QEMU | QEMU test only | QEMU standalone |

## Related Documentation

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli binary specification
- [apps/error_types.md](error_types.md) — Apps-specific error types
- [apps/toolchain.md](toolchain.md) — Main adapters
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL testing