# VSCode Configuration Schema

**Target Domain**: frontend — VSCode extension settings
**Source**: Design Specification §16.1

## Overview

The `hestia.*` configuration schema provided by the VSCode extension `hestia-vscode`. Users customize Hestia behavior via `settings.json`.

## Configuration Settings (hestia.*)

| Setting Key | Type | Default | Description |
|-------------|------|---------|-------------|
| `hestia.agentCliRegistryDir` | string | `$XDG_RUNTIME_DIR/agent-cli/` | agent-cli IPC registry directory (uses agent-cli default when empty) |
| `hestia.autoConnect` | boolean | true | Auto-connect on startup |
| `hestia.reconnectInterval` | number | 5000 | Reconnection interval (ms) |
| `hestia.requestTimeout` | number | 30000 | Request timeout (ms) |
| `hestia.ai.model` | string | `"claude-sonnet-4-6"` | AI model selection (`claude-sonnet-4-6` / `claude-opus-4-7` / `claude-haiku-4-5`) |
| `hestia.ai.maxTokens` | number | 4096 | AI response max token count |
| `hestia.ai.apiKeyEnv` | string | `"ANTHROPIC_API_KEY"` | API key environment variable name |
| `hestia.ai.baseUrl` | string | `""` | OpenAI-compatible API endpoint |

## ConductorId Mapping

Peer names (ConductorId) map 1:1 to configuration keys:

| ConductorId | Corresponding Conductor |
|-------------|------------------------|
| `ai` | ai-conductor |
| `rtl` | rtl-conductor |
| `fpga` | fpga-conductor |
| `asic` | asic-conductor |
| `pcb` | pcb-conductor |
| `hal` | hal-conductor |
| `apps` | apps-conductor |
| `debug` | debug-conductor |
| `rag` | rag-conductor |

## Configuration Example (settings.json)

```json
{
  "hestia.agentCliRegistryDir": "/run/user/1000/agent-cli",
  "hestia.autoConnect": true,
  "hestia.requestTimeout": 60000,
  "hestia.ai.model": "claude-opus-4-7",
  "hestia.ai.maxTokens": 8192,
  "hestia.ai.apiKeyEnv": "ANTHROPIC_API_KEY"
}
```

## Related Documentation

- [vscode_extension.md](vscode_extension.md) — VSCode extension
- [agent_cli_client.md](agent_cli_client.md) — agent-cli client specification
- [backend_switching.md](../common/backend_switching.md) — LLM backend switching