# rtl-conductor Error Types

**Target Conductor**: rtl-conductor
**Source**: Design Specification §14.3 (around lines 3565-3581), §4 (around lines 1241-1397)

## Error Categories

rtl-conductor uses HESTIA common error codes (-32000 to -32099) and request standard errors (-32600 to -32603). RTL-specific errors are subdivided within the common range.

## Lint Errors

| Error | Description |
|-------|-------------|
| LINT_ADAPTER_NOT_FOUND | Specified Lint adapter not registered |
| LINT_EXECUTION_FAILED | Lint tool execution failure (Verilator/Verible, etc. process error) |
| LINT_PARSE_ERROR | Lint output parse failure |
| LINT_VIOLATIONS_FOUND | Lint violations detected (warnings/errors) |

## Simulation Errors

| Error | Description |
|-------|-------------|
| SIM_ADAPTER_NOT_FOUND | Specified simulation adapter not registered |
| SIM_COMPILATION_FAILED | Testbench / RTL compilation failure |
| SIM_RUNTIME_ERROR | Simulation runtime error (assertion failure, etc.) |
| SIM_TIMEOUT | Simulation execution timeout |
| SIM_TESTBENCH_NOT_FOUND | Specified testbench does not exist |

## Formal Verification Errors

| Error | Description |
|-------|-------------|
| FORMAL_ADAPTER_NOT_FOUND | Formal verification adapter not registered |
| FORMAL_PROOF_FAILED | Formal verification property proof failure |
| FORMAL_TIMEOUT | Formal verification timeout |
| FORMAL_PROPERTY_INVALID | Invalid property definition |

## Transpilation Errors

| Error | Description |
|-------|-------------|
| TRANSPILE_UNSUPPORTED_LANGUAGE | Unsupported source/target language |
| TRANSPILE_COMPILATION_FAILED | Source compilation failure for transpilation |
| TRANSPILE_OUTPUT_ERROR | Transpilation output error |

## Handoff Errors

| Error | Description |
|-------|-------------|
| HANDOFF_TARGET_UNKNOWN | Unknown handoff target (not fpga/asic/hal) |
| HANDOFF_ARTIFACT_MISSING | Specified artifact does not exist |
| HANDOFF_DOWNSTREAM_UNREACHABLE | Downstream conductor unreachable |

## Build State Machine Errors

When a failure occurs at any build state (Linting / Compiling / Simulating / FormalChecking / Reporting), it transitions to the `Failed` state and generates a fix suggestion via `Diagnosing`.

## Common Error Code Reference

| Range | Domain |
|-------|--------|
| -32700 | Parse Error |
| -32600 to -32603 | Request standard errors |
| -32000 to -32099 | HESTIA common (Timeout / NotFound / AlreadyExists / PermissionDenied / InvalidState, etc.) |

## Related Documentation

- [rtl/message_methods.md](message_methods.md) — rtl.* method list
- [rtl/state_machines.md](state_machines.md) — Build state machine
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter trait
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry