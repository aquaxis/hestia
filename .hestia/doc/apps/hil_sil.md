# apps-conductor HIL/SIL Testing

**Target Conductor**: apps-conductor
**Source**: Design specification §9 (around lines 2281-2400)

## Test Mode Overview

| Mode | Description | Execution Environment | Speed | Accuracy |
|-------|------------|----------------------|-------|----------|
| SIL | Software-in-the-Loop | QEMU emulation | Fast | Functional verification level |
| HIL | Hardware-in-the-Loop | Physical device + debug-conductor | Real-time | Cycle-accurate |
| QEMU | QEMU test only | QEMU standalone | Fast | Functional verification level |

## SIL (Software-in-the-Loop) Testing

Software-level testing using QEMU emulation.

### Features

- No hardware required (completed within QEMU)
- Fast execution (can be faster than real-time)
- Integrable into CI/CD pipelines
- Suitable for functional correctness verification

### Supported Targets

| Target | QEMU Command |
|-----------|-------------|
| ARM Cortex-M | `qemu-system-arm -machine <board>` |
| RISC-V 32bit | `qemu-system-riscv32 -machine <board>` |

### Test Flow

```
1. Launch QEMU (load firmware ELF)
2. Execute test scenario (serial output / GDB connection)
3. Determine result (exit code / output string / memory state)
4. Generate test report
```

### apps-conductor Integration

- Adapter: `qemu-system`
- apps.toml: `[test] mode = "sil"` or `mode = "qemu"`
- GDB remote debugging support

## HIL (Hardware-in-the-Loop) Testing

Hardware-level testing combining physical hardware with debug-conductor (§10).

### Features

- Testing on physical hardware (true execution environment)
- Cycle-accurate verification
- Real peripheral and interrupt behavior verification
- Timing constraint validation

### Test Flow

```
1. Connect probe via debug-conductor (ST-Link / J-Link, etc.)
2. Write firmware (probe-rs / OpenOCD)
3. Execute on target and acquire RTT logs
4. Execute test scenario (real peripheral operations: GPIO / UART / SPI, etc.)
5. Determine result and generate test report
```

### apps-conductor Integration

- Adapter: `probe-rs` / `openocd-bridge`
- apps.toml: `[test] mode = "hil"` / `probe = "stlink-v3"`
- Debug session management in coordination with debug-conductor (§10)

## Cross-Testing (QEMU + Cycle-Accurate Co-Simulation)

Hybrid testing combining QEMU with a cycle-accurate simulator. Functional verification (QEMU) and timing verification (cycle-accurate) are performed simultaneously.

## Test Report

| Item | Description |
|------|-------------|
| Test name | Test case identifier |
| Result | PASS / FAIL / SKIP |
| Execution time | Time required for test execution |
| Coverage | Code coverage (gcov / llvm-cov) |
| Logs | RTT logs / serial output |
| Memory usage | Stack usage / heap usage |

## CI/CD Integration

SIL tests can be run automatically on GitHub Actions / GitLab CI via the shared services layer / CI/CD API (§13). HIL tests require physical device connection, so they run on local or dedicated CI runners.

## Related Documentation

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli binary specification
- [apps/state_machines.md](state_machines.md) — Build state machine
- [apps/toolchain.md](toolchain.md) — Main adapters
- [apps/rtos.md](rtos.md) — RTOS support
- [../debug/binary_spec.md](../debug/binary_spec.md) — debug-conductor CLI