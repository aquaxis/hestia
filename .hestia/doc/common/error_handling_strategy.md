# Error Handling Strategy

**Domain**: common — Error Handling
**Source**: Design Specification §18.9, §14.3

## Overview

HESTIA uses Rust's `thiserror` and `anyhow` error handling ecosystem, selecting between them based on use case. Library crates define type-safe errors with `thiserror`, while binary crates use `anyhow` for flexible error handling. This aligns with the error code conventions (§14.3).

## thiserror / anyhow Separation Policy

| Use Case | Crate | Choice | Reason |
|------|---------|------|------|
| Library | `conductor-sdk`, `adapter-core`, `project-model`, etc. | `thiserror` | Callers can branch on error type; type-safe |
| Binary | `hestia-fpga-conductor`, `hestia-ai-cli`, etc. | `anyhow` | Simplified error propagation; bulk handling at top level |

## Error Type Design Patterns

### Library Side (thiserror)

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConductorError {
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    #[error("Build failed: exit code {exit_code}")]
    BuildFailed { exit_code: i32 },

    #[error("Timeout after {secs}s")]
    Timeout { secs: u64 },

    #[error("JSON-RPC error {code}: {message}")]
    Rpc { code: i32, message: String },
}
```

### Binary Side (anyhow)

```rust
fn main() -> anyhow::Result<()> {
    let config = HestiaConfig::from_toml_file(path)?;
    // Simple propagation with ? operator
    conductor.run().await?;
    Ok(())
}
```

## Alignment with Error Code Conventions

When converting library `thiserror` types to structured message error responses (§14.3), errors are mapped to error codes:

```rust
impl From<ConductorError> for ErrorResponse {
    fn from(err: ConductorError) -> Self {
        match err {
            ConductorError::ToolNotFound { .. } => ErrorResponse { code: -32209, .. },
            ConductorError::BuildFailed { .. }  => ErrorResponse { code: -32201, .. },
            ConductorError::Timeout { .. }      => ErrorResponse { code: -32001, .. },
            ConductorError::Rpc { code, .. }    => ErrorResponse { code, .. },
        }
    }
}
```

## Error Response data Field Convention

All error responses must include the following in `data`:

| Field | Type | Description |
|---------|-----|------|
| `tool` | string | Originating tool name |
| `exit_code` | int | Process exit code |
| `log_path` | string | Log file path |
| `errors[]` | array | Error detail list |
| `retry_possible` | bool | Whether retry is possible |
| `suggested_action` | string | Recommended action |

## Related Documents

- [error_registry.md](error_registry.md) — Error code complete listing
- [agent_message.md](agent_message.md) — Message payload format
- [observability.md](observability.md) — Monitoring and logging