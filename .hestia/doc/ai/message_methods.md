# ai-conductor Message Method List

**Target Conductor**: ai-conductor
**Source**: Design Specification §14 (around lines 3492-3630), §3 (around lines 745-1240)

## Transport

All communication is unified via agent-cli native IPC. Messages are sent with `agent-cli send <peer> <payload>`. If the payload starts with `{`, it is interpreted as structured JSON; otherwise, it is treated as natural language.

## ai.* Method List

### Spec-Driven Development

| Method | Direction | Description |
|---------|------|------|
| `ai.spec.init` | Request | Initialize specification session. Start DesignSpec generation from natural language specification |
| `ai.spec.update` | Request | Update an existing DesignSpec. Append requirements, constraints, and interfaces using REQ/CON/IF prefixes |
| `ai.spec.review` | Request | Start specification review. Returns review results + modification suggestions |

### Execution and Control

| Method | Direction | Description |
|---------|------|------|
| `ai.exec` | Request | Direct execution of natural language or structured instructions. task-router performs intent understanding → task decomposition → routing |

### Agent Management

| Method | Direction | Description |
|---------|------|------|
| `agent_spawn` | Request | Launch a new sub-agent (planner/designer/coder-N/tester) |
| `agent_list` | Request | Get list of registered sub-agents. Equivalent to agent-cli list |
| `agent.status_update` | Notification | Agent state change notification (no id, no response) |

### Container Management

| Method | Direction | Description |
|---------|------|------|
| `container.list` | Request | Get container list |
| `container.start` | Request | Start a container |
| `container.stop` | Request | Stop a container |
| `container.create` | Request | Generate and build a Containerfile from container.toml |
| `container.update` | Request | Differential update of container image |

### Workflow

| Method | Direction | Description |
|---------|------|------|
| `meta.dualBuild` | Request | Parallel build across multiple conductors (DAG: fpga.build ‖ asic.synth → meta.collect) |
| `meta.boardWithFpga` | Request | Cross-conductor workflow (e.g., FPGA + PCB integration) |
| `meta.handoff` | Notification | Inter-conductor handoff event (rtl → fpga/asic, etc.) |

### System Common

| Method | Direction | Description |
|---------|------|------|
| `system.readiness` | Request | Check ai-conductor readiness state (`{ ready: bool }`) |
| `system.health` | Request | Health check (Online / Offline / Degraded / Upgrading) |
| `system.shutdown` | Request | Shut down ai-conductor |
| `agent.alert` | Notification | Alert notification to frontend (e.g., on consecutive health check failures) |

## Payload Format

### Request

```json
{
  "method": "ai.spec.init",
  "params": { "spec_text": "...", "format": "natural" },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Success Response

```json
{
  "result": { "design_spec": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Error Response

```json
{
  "error": { "code": -32100, "message": "...", "data": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

## Method Namespace Convention

`{domain}.{method_group}.{version_prefix}.{action}` (e.g., `fpga.build.v1.synthesize`). The short form `{domain}.{action}` is equivalent (v1 default).

- `ApiVersion { major, minor }`
- Compatibility: Adding required parameters, changing existing types, or removing methods requires a `major` bump
- Deprecation notice: `DeprecationNotice { deprecated_since, removal_scheduled, replacement }`

## Related Documentation

- [ai/binary_spec.md](binary_spec.md) — hestia-ai-cli binary specification
- [ai/error_types.md](error_types.md) — ai-conductor error codes
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine details
- [ai/skills_system.md](skills_system.md) — SkillSystem details
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification