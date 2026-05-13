# ai-conductor Error Codes

**Target Conductor**: ai-conductor
**Source**: Design Specification §14.3 (around lines 3565-3581)

## Error Code Range

ai-conductor uses the error code range **-32100 to -32199**.

## Error Categories

### Orchestration

| Code | Name | Description |
|-------|------|------|
| -32100 | ORCHESTRATION_ERROR | General error during task routing or workflow execution |
| -32101 | TASK_DECOMPOSITION_FAILED | Task decomposition failed (intent incomprehension or dependency analysis error) |
| -32102 | WORKFLOW_EXECUTION_FAILED | Workflow (DAG) execution failed |
| -32103 | CONDUCTOR_UNREACHABLE | Downstream conductor unreachable (agent-cli IPC timeout) |
| -32104 | DAG_CYCLE_DETECTED | Circular dependency detected in DAG |

### Agent Management

| Code | Name | Description |
|-------|------|------|
| -32110 | AGENT_SPAWN_FAILED | Sub-agent launch failed |
| -32111 | AGENT_NOT_FOUND | Specified agent does not exist |
| -32112 | AGENT_COMMUNICATION_FAILED | Inter-agent communication failed (IPC error) |
| -32113 | AGENT_TIMEOUT | Agent response timeout |
| -32114 | MAX_AGENTS_EXCEEDED | Agent parallelism limit exceeded |

### Spec-Driven

| Code | Name | Description |
|-------|------|------|
| -32120 | SPEC_PARSE_ERROR | Specification parse failed (invalid REQ/CON/IF prefix) |
| -32121 | SPEC_VALIDATION_FAILED | DesignSpec validation failed (missing required constraints, etc.) |
| -32122 | SPEC_REVIEW_FAILED | Review session failed |
| -32123 | DESIGN_SPEC_CONFLICT | Contradiction detected between multiple specifications |

### Version Tracking

| Code | Name | Description |
|-------|------|------|
| -32130 | VERSION_INCOMPATIBLE | Semantic versioning incompatibility |
| -32131 | ROLLOUT_FAILED | Gradual rollout failed (Canary/Staging) |
| -32132 | ROLLBACK_FAILED | Automatic rollback failed |
| -32133 | UPGRADE_CHECK_FAILED | New version check failed (WatcherAgent) |

### LLM (Large Language Model)

| Code | Name | Description |
|-------|------|------|
| -32140 | LLM_BACKEND_UNAVAILABLE | LLM backend unreachable (Ollama / Anthropic / LM Studio / vLLM) |
| -32141 | LLM_INFERENCE_FAILED | LLM inference failed |
| -32142 | LLM_TIMEOUT | LLM response timeout |
| -32143 | TOOL_USE_EXECUTION_FAILED | Tool execution failed in Tool Use feature |

## Common Error Codes (HESTIA-wide)

| Range | Domain |
|------|------|
| -32700 | Parse Error (JSON payload parse failure) |
| -32600 to -32603 | Standard request errors (Invalid Request / Method not found / Invalid params / Internal) |
| -32000 to -32099 | HESTIA common (Timeout / NotFound / AlreadyExists / PermissionDenied / InvalidState, etc.) |

## Error Response Format

The `data` field should include:

| Field | Description |
|-----------|------|
| `tool` | Error source tool name |
| `exit_code` | Process exit code |
| `log_path` | Log file path |
| `errors[]` | Detailed error list |
| `retry_possible` | Whether retry is possible |
| `suggested_action` | Recommended action |

```json
{
  "error": {
    "code": -32101,
    "message": "Task decomposition failed",
    "data": {
      "tool": "task-router",
      "retry_possible": true,
      "suggested_action": "Specify target conductor explicitly"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

## Related Documentation

- [ai/message_methods.md](message_methods.md) — ai.* method list
- [ai/state_machines.md](state_machines.md) — Task state transitions
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry
- [../common/error_handling_strategy.md](../common/error_handling_strategy.md) — Error handling strategy