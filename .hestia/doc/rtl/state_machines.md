# rtl-conductor Build State Machine

**Target Conductor**: rtl-conductor
**Source**: Design Specification §4.3 (around lines 1280-1286)

## 7-State Build State Machine

```
Idle → Resolving → Linting → Compiling → Simulating → FormalChecking → Reporting → Done
                                                  ↓ (on failure)
                                              Failed → Diagnosing → Fix suggestion
```

## State Definitions

| State | Description | Input | Output |
|-------|-------------|-------|--------|
| Idle | Initial state | Build start command | — |
| Resolving | Resolving dependencies (adapter selection, toolchain verification) | rtl.toml | Resolved adapter information |
| Linting | Running Lint / format / static analysis | HDL source | LintReport (warnings/errors) |
| Compiling | Compiling (e.g., simulation build) | HDL source + testbench | Compiled simulation model |
| Simulating | Running simulation | Testbench + compiled model | SimReport (pass/fail, coverage) |
| FormalChecking | Running formal verification | Property definitions | FormalReport (proof results) |
| Reporting | Aggregating results and generating report | Results from each step | Integrated report |
| Done | Normal completion | — | All report output |
| Failed | Error occurred | Error information | Error details + fix suggestion |
| Diagnosing | Analyzing root cause | Error information | Fix suggestion (patch / configuration change proposal) |

## State Transition Rules

| Transition | Trigger | Condition |
|-----------|---------|-----------|
| Idle → Resolving | Build start command received | — |
| Resolving → Linting | Dependency resolution complete | All adapters available |
| Linting → Compiling | Lint complete | No fatal Lint errors |
| Linting → Failed | Lint failed | Fatal error detected |
| Compiling → Simulating | Compilation complete | Compilation successful |
| Compiling → Failed | Compilation failed | — |
| Simulating → FormalChecking | Simulation complete | Simulation successful or warnings only |
| Simulating → Failed | Simulation failed | Assertion violation, etc. |
| FormalChecking → Reporting | Formal verification complete | — |
| Reporting → Done | Report generation complete | — |
| Failed → Diagnosing | Diagnosis started | — |

## Parallel Execution

Simulating and FormalChecking are independent of each other and can run in parallel. However, the design specification defines sequential execution (Simulating → FormalChecking).

## Related Documentation

- [rtl/binary_spec.md](binary_spec.md) — hestia-rtl-cli binary specification
- [rtl/error_types.md](error_types.md) — RTL-specific error types
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter trait
- [rtl/handoff.md](handoff.md) — Downstream handoff