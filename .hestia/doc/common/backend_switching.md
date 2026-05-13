# LLM Backend Switching / Engine Switching

**Domain**: common — Peer-driven Engine
**Source**: Design Specification §20 / Phase 113

## Overview

HESTIA's peer-driven system has two levels of switching:

1. **Engine Switching (Phase 113)** — Choose between `agent-cli` or `claude-cli-shim` as the binary that drives peers. Specified in the `[engine]` section of `.hestia/config.toml`.
2. **LLM Backend Switching** — When the engine is `agent-cli`, select from 4 backend LLM types (via the `[agent_cli]` section). When using the `claude-cli-shim` engine, Claude Code (Anthropic API) is the sole fixed backend.

## Engine (Phase 113)

| Engine | `[engine] type` value | Binary | Purpose |
|--------|--------------------|----------|------|
| agent-cli (default) | `"agent_cli"` or unset | `agent-cli` | Supports 4 LLM backends, legacy behavior |
| claude-cli-shim | `"claude_cli_shim"` | `claude-cli-shim` | Wrapper that holds Claude Code (`claude` CLI) as a subprocess, Plan C |

```toml
[engine]
# "agent_cli" (default, backward compatible) | "claude_cli_shim"
type = "agent_cli"
binary = ""           # When omitted, default path based on type
registry_path = ""    # When omitted, engine default (~/.local/share/<engine>/registry)
log_path = ""         # When omitted, engine default (~/.local/share/<engine>/logs)
```

When `[engine]` is unset, `type = "agent_cli"` is the default, fully compatible with legacy behavior.

When `type = "claude_cli_shim"` is selected, hestia spawns `claude-cli-shim run`, and the shim internally holds `claude --input-format stream-json --output-format stream-json --print` as a subprocess. The registry/log are recorded in a separate directory using an agent-cli compatible schema.

## Supported Backends (agent-cli engine only)

| Backend | `backend` value | Characteristics |
|------------|-------------|------|
| Anthropic Claude | `"claude"` | Default. High-accuracy Tool Use |
| OpenAI Codex | `"codex"` | OpenAI API compatible |
| Ollama | `"ollama"` | Local execution, offline support |
| llama.cpp | `"llama_cpp"` | OpenAI-compatible endpoint |

**Note**: When `type = "claude_cli_shim"`, this table is irrelevant (it connects directly to the Anthropic API via Claude Code).

## `[agent_cli]` Schema

```toml
[agent_cli]
backend = "claude"                            # "claude" | "codex" | "ollama" | "llama_cpp"
binary_path = ""                              # Empty = $PATH resolution / full path can be specified
anthropic_base_url = ""                       # Empty = Official Anthropic / URL for OpenAI-compatible API
anthropic_api_key_env = "ANTHROPIC_API_KEY"   # Host environment variable name storing the API key
model = "claude-opus-4-7"                     # LLM model identifier
max_tokens = 4096                             # Default response token limit
registry_dir = ""                             # agent-cli IPC registry (empty = $XDG_RUNTIME_DIR/agent-cli)
```

## Rust Types

```rust
pub struct AgentCliSection {
    pub backend: String,            // default: "claude"
    pub binary_path: String,        // default: ""
    pub anthropic_base_url: String, // default: ""
    pub anthropic_api_key_env: String, // default: "ANTHROPIC_API_KEY"
    pub model: String,             // default: "claude-opus-4-7"
    pub max_tokens: u32,          // default: 4096
    pub registry_dir: String,     // default: ""
}
```

## Environment Variable Forwarding (FR-CFG-07)

1. Read `config.toml` (`HestiaConfig::from_toml_file`)
2. Retrieve the environment variable specified by `anthropic_api_key_env` from the host (fail-fast if unset/empty)
3. If `anthropic_base_url` is non-empty, inject `ANTHROPIC_BASE_URL` into the subprocess
4. Inject the API key as `ANTHROPIC_API_KEY` into the subprocess
5. Spawn agent-cli subprocess via `tokio::process::Command::spawn`

Helper: `AgentCliSection::build_env() -> Result<Vec<(String, String)>, AgentCliEnvError>`

## Security Considerations

- **No plaintext API keys**: Do not write keys directly in `config.toml`
- **Environment variables only**: Resolve from secret backends such as 1Password CLI / direnv / systemd EnvironmentFile / GPG
- **Fail-fast when unset**: Fail before startup with `AgentCliEnvError::MissingApiKeyEnv`
- **Log output masking**: Display length only in the format `ANTHROPIC_API_KEY=<set, len=N>`
- **Registry permissions**: `0700` to prevent peer discovery by other users

## Usage Examples

### Anthropic Claude (default)

```toml
[agent_cli]
backend = "claude"
anthropic_api_key_env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-7"
max_tokens = 4096
```

### Ollama (local)

```toml
[agent_cli]
backend = "ollama"
anthropic_base_url = "http://localhost:11434/v1/"
anthropic_api_key_env = "OLLAMA_API_KEY"
model = "glm-5.1:cloud"
max_tokens = 8192
```

### OpenAI Codex / llama.cpp / LM Studio

- **Codex**: `backend = "codex"` + `model = "gpt-4.1"` + `anthropic_base_url = "https://api.openai.com/v1/"`
- **llama.cpp**: `backend = "llama_cpp"` + `anthropic_base_url = "http://localhost:8080/v1/"`
- **LM Studio**: `backend = "llama_cpp"` + `anthropic_base_url = "http://localhost:1234/v1/"`

### claude-cli-shim engine (Phase 113, Claude Code wrapper)

```toml
[engine]
type = "claude_cli_shim"
# binary = "/home/hidemi/.local/bin/claude-cli-shim"   # When omitted, PATH resolution
# registry_path = "/custom/path"                       # Specify only when sharing
# log_path = "/custom/path"
```

Prerequisites:
- `claude` CLI installed (`which claude`)
- `ANTHROPIC_API_KEY` set in environment variables
- `claude-cli-shim` binary exists in PATH (after `cargo build`, `target/debug/claude-cli-shim`)

When cross-engine communication is needed, set `[agent_cli]` `[engine] registry_path` to a shared path to synchronize the registry between agent-cli and shim (be mindful of peer name collisions).

## Test Strategy

8 unit tests + 3 integration tests under `project-model::config`:

1. `agent_cli_section_defaults` — Default value verification
2. `agent_cli_section_parses_with_defaults_when_omitted` — Default completion when omitted
3. `agent_cli_section_round_trip_with_custom_values` — TOML round-trip with Ollama settings
4. `default_template_includes_agent_cli` — Default template inclusion verification
5. `build_env_anthropic_official_default` — Inject verification with empty base_url
6. `build_env_ollama_includes_base_url` — 2 inject verifications with Ollama settings
7. `build_env_missing_api_key_returns_error` / `build_env_empty_api_key_returns_error` — Fail-fast verification
8. `backend_enum_parse` — 4 backend type parse verification

## Related Documents

- [agent_cli_messaging.md](agent_cli_messaging.md) — agent-cli messaging specification
- [sub_agent_lifecycle.md](sub_agent_lifecycle.md) — Sub-agent startup and shutdown management
- [error_registry.md](error_registry.md) — Error code conventions