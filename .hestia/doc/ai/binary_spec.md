# ai-conductor CLI Binary Specification

**Target Conductor**: ai-conductor
**Source**: Design Specification §15 (around lines 3631-3730), §3 (around lines 745-1240)

## Binary Name

`hestia-ai-cli`

## Subcommand List

| Subcommand | Description |
|-------------|------|
| `exec` | Execute natural language instructions directly on agent-cli |
| `run --file <path>` | Execute a workflow YAML file |
| `agent ls` | List registered sub-agents |
| `container ls` | List containers |
| `container start <id>` | Start a container |
| `container stop <id>` | Stop a container |
| `container create` | Generate and build a Containerfile from container.toml |
| `workflow run <yaml>` | Execute a DAG-based workflow (via §3.5 WorkflowEngine) |
| `review start` | Start a specification review session (§3.6 SpecDriven) |

## Common Options (CommonOpts)

| Option | Value | Description |
|-----------|---|------|
| `--output` | `human` \| `json` | Output format (default: human) |
| `--timeout` | `<seconds>` | RPC timeout |
| `--registry` | `<path>` | agent-cli registry path (default: `$XDG_RUNTIME_DIR/agent-cli/`) |
| `--config` | `<path>` | Configuration file path |
| `--verbose` | — | Enable verbose logging |

## Exit Codes

| Exit Code | Meaning |
|-----------|------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## CLI Architecture

Rust-based client binary (`tokio` + `serde` + `clap`). Connects to the corresponding conductor's agent-cli peer (peer name `ai`) via agent-cli native IPC (`agent-cli send <peer> <payload>`). Can execute full workflows without a frontend.

## Usage Examples

```bash
# List agents
hestia-ai-cli agent ls

# Create and start a container
hestia-ai-cli container create
hestia-ai-cli container start vivado-build

# Run a workflow
hestia-ai-cli run --file workflow/fpga_to_asic.yaml

# Start a specification review
hestia-ai-cli review start --spec spec/dsp_core.md
```

## Related Documentation

- [ai/config_schema.md](config_schema.md) — container.toml / upgrade.toml configuration schemas
- [ai/message_methods.md](message_methods.md) — ai.* method list
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine details
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification
- [../frontend/cli_clients.md](../frontend/cli_clients.md) — CLI client common specification