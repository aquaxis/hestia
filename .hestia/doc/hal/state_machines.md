# hal-conductor Build State Machine

**Target Conductor**: hal-conductor
**Source**: Design specification §8.3 (around lines 2210-2216)

## 5-State Build State Machine

```
Idle → Parsing → Validating → Generating → Reporting → Done
                          ↓ (bus boundary violation / address overlap / type mismatch)
                      Failed → Diagnosing → Fix proposal
```

## State Definitions

| State | Description | Input | Output |
|-------|-------------|-------|--------|
| Idle | Initial state | Build start command | — |
| Parsing | Parsing register definition files | SystemRDL / IP-XACT / TOML | RegisterMap |
| Validating | Validating register map | RegisterMap | ValidationReport |
| Generating | Generating multi-language code | RegisterMap + output language specification | C header / Rust crate / Python module / SVD |
| Reporting | Generating result report | Results from each step | Integrated report |
| Done | Normal completion | — | All artifacts output |
| Failed | Error occurred | Error information | Error details + fix proposal |
| Diagnosing | Analyzing root cause | Error information | Fix proposal |

## State Transition Rules

| Transition | Trigger | Condition |
|------------|---------|-----------|
| Idle → Parsing | Parse start | — |
| Parsing → Validating | Parse complete | No syntax errors |
| Parsing → Failed | Parse failure | Syntax errors detected |
| Validating → Generating | Validation complete | No address overlap, type mismatches, or bus boundary violations |
| Validating → Failed | Validation failure | Violations detected |
| Generating → Reporting | Generation complete | All output languages generated successfully |
| Generating → Failed | Generation failure | Error in some output generation |
| Reporting → Done | Report complete | — |
| Failed → Diagnosing | Diagnosis start | — |

## Failure Patterns

| Failure State | Cause | Fix Proposal |
|---------------|-------|--------------|
| Parsing failure | Syntax error | Identify error location and suggest fixes |
| Validating failure | Bus boundary violation | Suggest address alignment adjustments |
| Validating failure | Address overlap | Suggest merging overlapping registers or reassigning addresses |
| Validating failure | Type mismatch | Suggest field width adjustments |
| Generating failure | Template error | Suggest template fixes |

## Multi-language Parallel Generation

When multiple output languages (C / Rust / Python / SVD) are specified, the Generating step can execute generation for each language in parallel (coder sub-agents launch in parallel per language).

## Related Documentation

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli binary specification
- [hal/error_types.md](error_types.md) — HAL-specific error types
- [hal/register_map.md](register_map.md) — Register map definition
- [hal/codegen.md](codegen.md) — Multi-language code generation