# apps-conductor Error Types

**Target Conductor**: apps-conductor
**Source**: Design specification §9 (around lines 2281-2400), §14.3 (around lines 3565-3581)

## Error Categories

apps-conductor uses HESTIA common error codes (-32000 to -32099) and request standard errors (-32600 to -32603). Apps-specific errors are subdivided within the common range.

## Build Errors

| Error | Description |
|-------|-------------|
| CROSS_COMPILATION_FAILED | Cross-compilation failure (arm-gcc / riscv-gcc / cargo) |
| TOOLCHAIN_NOT_FOUND | Specified toolchain not found |
| TOOLCHAIN_VERSION_MISMATCH | Toolchain version mismatch |
| COMPILATION_ERROR | Source code compilation error |
| LINK_ERROR | Link error (unresolved symbols, duplicate definitions, etc.) |

## Memory Errors

| Error | Description |
|-------|-------------|
| MEMORY_OVERFLOW_FLASH | Flash region overflow |
| MEMORY_OVERFLOW_RAM | RAM region overflow |
| LINKER_SCRIPT_ERROR | Linker script error |
| SIZE_CHECK_FAILED | Binary size check failure |

## RTOS Errors

| Error | Description |
|-------|-------------|
| RTOS_NOT_FOUND | Specified RTOS not installed |
| RTOS_VERSION_INCOMPATIBLE | RTOS version incompatibility |
| FREERTOS_CONFIG_ERROR | FreeRTOS configuration error |
| ZEPHYR_WEST_FAILED | Zephyr west command failure |

## Flash Errors

| Error | Description |
|-------|-------------|
| FLASH_FAILED | Flash write failure |
| PROBE_NOT_FOUND | Debug probe not found |
| PROBE_CONNECTION_ERROR | Probe connection error |
| TARGET_NOT_RESPONDING | Target device not responding |

## Test Errors

| Error | Description |
|-------|-------------|
| TEST_FAILED | Test execution failure |
| QEMU_LAUNCH_FAILED | QEMU launch failure |
| HIL_CONNECTION_FAILED | HIL test connection failure |
| TEST_TIMEOUT | Test timeout |

## HAL Integration Errors

| Error | Description |
|-------|-------------|
| HAL_IMPORT_NOT_FOUND | HAL module import destination not found |
| HAL_VERSION_MISMATCH | HAL version mismatch |

## Build State Machine Errors

When a failure occurs in any state during a build (Resolving / Compiling / Linking / SizeChecking / Flashing / Testing), the state machine transitions to `Failed` and generates a fix suggestion in `Diagnosing`.

## Related Documentation

- [apps/message_methods.md](message_methods.md) — apps.* method list
- [apps/state_machines.md](state_machines.md) — Build state machine
- [apps/toolchain.md](toolchain.md) — Main adapters
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry