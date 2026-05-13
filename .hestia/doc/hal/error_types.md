# hal-conductor Error Types

**Target Conductor**: hal-conductor
**Source**: Design specification §8 (around lines 2175-2280), §14.3 (around lines 3565-3581)

## Error Categories

hal-conductor uses HESTIA common error codes (-32000 to -32099) and standard request errors (-32600 to -32603). HAL-specific errors are sub-classified within the common range.

## Parse Errors

| Error | Description |
|-------|-------------|
| PARSE_FORMAT_UNSUPPORTED | Unsupported input format |
| PARSE_SYNTAX_ERROR | SystemRDL / IP-XACT / TOML syntax error |
| PARSE_FILE_NOT_FOUND | Register definition file not found |
| PARSE_SCHEMA_MISMATCH | Schema version mismatch |

## Validation Errors

| Error | Description |
|-------|-------------|
| VALIDATION_ADDRESS_OVERLAP | Address overlap detected |
| VALIDATION_BUS_BOUNDARY_VIOLATION | Bus boundary violation (e.g., register spanning a 32-bit boundary) |
| VALIDATION_TYPE_MISMATCH | Type mismatch (e.g., field width exceeds register width) |
| VALIDATION_ACCESS_CONFLICT | Access rights conflict (e.g., initial value on a write-only field) |
| VALIDATION_RESERVED_FIELD_WRITE | Write definition on a reserved field |

## Code Generation Errors

| Error | Description |
|-------|-------------|
| CODEGEN_TARGET_UNSUPPORTED | Unsupported output language |
| CODEGEN_TEMPLATE_ERROR | Template processing error |
| CODEGEN_OUTPUT_WRITE_ERROR | Output file write error |
| CODEGEN_RUST_CRATE_BUILD_FAILED | Rust crate build failure |

## Diff Errors

| Error | Description |
|-------|-------------|
| DIFF_BASELINE_NOT_FOUND | Baseline version not found |
| DIFF_INCOMPATIBLE_VERSIONS | Incompatible versions for comparison |

## Bus Protocol Errors

| Error | Description |
|-------|-------------|
| BUS_PROTOCOL_UNSUPPORTED | Unsupported bus protocol |
| BUS_WIDTH_MISMATCH | Data/address width mismatch with register definitions |

## Build State Machine Errors

When a failure occurs at any build state (Parsing / Validating / Generating / Reporting), the machine transitions to the `Failed` state, and `Diagnosing` generates a fix proposal.

## Related Documentation

- [hal/message_methods.md](message_methods.md) — hal.* method list
- [hal/state_machines.md](state_machines.md) — Build state machine
- [hal/register_map.md](register_map.md) — Register map definition
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry