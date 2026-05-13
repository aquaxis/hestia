# Communication Specification (Agent Communication)

**Scope**: Hestia overall
**Source**: Design specification §2.3 (Communication Architecture), §14 (Interface Definition), §20 (agent-cli Endpoint Configuration)

---

## 1. Communication Architecture Overview

All conductors are AI agents launched as [`agent-cli`](https://github.com/aquaxis/agent-cli) processes. All communication between the frontend and between conductors is unified under **agent-cli native IPC**.

The legacy JSON-RPC 2.0 over Unix Domain Socket (`/var/run/hestia/*.sock`) has been deprecated. Connections are made via the `agent-cli send <peer> <payload>` API and shared registry (`$XDG_RUNTIME_DIR/agent-cli/`) for peer discovery.

```
┌─────────────────────────────────────────────────────────────────┐
│        Communication Protocol Stack (agent-cli single channel)  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ━━━ agent-cli native IPC (sole communication means) ━━━       │
│                                                                 │
│  All 9 conductors + frontend clients join as peers              │
│      Registry: $XDG_RUNTIME_DIR/agent-cli/  (permissions 0700) │
│      Discovery: agent-cli list                                  │
│      Send:       agent-cli send <peer> <payload>                │
│                  or REPL command /send <peer> <payload>          │
│                                                                 │
│      Peer names (= ConductorId strings):                         │
│        ai / rtl / fpga / asic / pcb /                           │
│        hal / apps / debug / rag                                  │
│                                                                 │
│      Frontend peer names: vscode / tauri / cli (arbitrary)      │
│                                                                 │
│  ━━━ Payload formats (coexist on the same channel) ━━━          │
│                                                                 │
│  (a) Structured message — JSON payload                           │
│      Follows method namespace conventions (§14 Messaging spec)   │
│                                                                 │
│  (b) Natural language message — plain text                       │
│      Free-form, CoT context sharing, agent-native coordination │
│                                                                 │
│  ━━━ Shared services layer (cross-cutting tools, as agent-cli peers) ━━━│
│                                                                 │
│  Shared service peer names: lsp / constraint-bridge / ip-manager /    │
│                       cicd / observability / waveform / mcp     │
│                                                                 │
│  ━━━ External adapters (outside agent-cli IPC boundary) ━━━    │
│                                                                 │
│  Remote adapters: gRPC (proto3)                                 │
│      Service: VendorAdapterService                               │
│      (gRPC used only at the boundary with external systems such as vendor tools) │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Payload Formats

### 2.1 Structured Request / Response / Notification

```json
// Request payload
{
  "method": "fpga.build.v1.synthesize",
  "params": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}

// Success Response payload
{
  "result": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}

// Error Response payload
{
  "error": { "code": -32200, "message": "...", "data": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}

// Notification payload (no id, no response)
{
  "method": "agent.status_update",
  "params": { ... },
  "trace_id": "trace_xyz789"
}

// Batch payload (responses in same order)
[ { "method":"...", "params":{}, "id":"msg_1" },
  { "method":"...", "params":{}, "id":"msg_2" } ]
```

- `id` follows the format `msg_{ISO8601 timestamp}_{random}`
- `trace_id` is a cross-workflow trace ID
- The legacy JSON-RPC 2.0 `"jsonrpc": "2.0"` field is not required

### 2.2 Payload Determination

The receiving agent-cli persona determines the payload type by inspecting the beginning:
- Starts with `{` → Parse as a JSON structured message and convert to a tool call
- Otherwise → Pass directly to the LLM as natural language

### 2.3 Payload Selection Guidelines

| Communication type | Recommended payload | Reason |
|---------|-------------|------|
| Structured operations from frontend | (a) Structured JSON | Type safety, error code conventions, SDK compatibility |
| Structured tool calls between conductors | (a) Structured JSON | Reproducibility, trace ID chaining, consistency with sled state persistence |
| Natural language coordination between conductors | (b) Natural language text | Free-form, CoT context sharing |
| Progress / CoT / thought process sharing | (b) Natural language text | Lightweight propagation, observability log integration |
| Error escalation | (b) Natural language aggregated to ai-conductor → (a) Structured notification to frontend |
| Event notifications | (a) Structured JSON (no id = notification) | Subscribable / filterable, immediate UI reflection |

---

## 3. Method Namespace

### 3.1 Naming Convention

`{domain}.{method_group}.{version_prefix}.{action}` (e.g., `fpga.build.v1.synthesize`)

The short form `{domain}.{action}` is equivalent (defaults to v1).

### 3.2 Versioning

- `ApiVersion { major, minor }`
- Adding required parameters, changing existing types, or removing methods requires a `major` bump
- Adding optional parameters / response fields is backward compatible
- Deprecation notice: `DeprecationNotice { deprecated_since, removal_scheduled, replacement }`

### 3.3 Domain List

| Domain | Examples |
|---------|----|
| `ai.*`   | `ai.spec.init` / `ai.spec.update` / `ai.spec.review` / `ai.exec` / `agent_spawn` / `agent_list` |
| `fpga.*` | `fpga.synthesize` / `fpga.implement` / `fpga.bitstream` / `fpga.simulate` / `fpga.program` |
| `asic.*` | `asic.synthesize` / `asic.floorplan` / `asic.place` / `asic.cts` / `asic.route` / `asic.gdsii` / `asic.drc` / `asic.lvs` |
| `pcb.*`  | `pcb.generate_schematic` / `pcb.run_drc` / `pcb.run_erc` / `pcb.generate_bom` / `pcb.place_components` / `pcb.route_traces` / `pcb.generate_output` / `pcb.ai_synthesize` / `pcb.status` |
| `debug.*`| `debug.connect` / `debug.disconnect` / `debug.program` / `debug.start_capture` / `debug.stop_capture` / `debug.read_signals` / `debug.set_trigger` / `debug.reset` / `debug.status` |
| `rag.*`  | `rag.ingest` / `rag.search` / `rag.cleanup` / `rag.status` |
| `meta.*` | `meta.dualBuild` / `meta.boardWithFpga` and other cross-Conductor workflows |
| `system.*` | `system.readiness` / `system.health` / `system.shutdown` |

---

## 4. Error Code Conventions

| Range | Domain |
|------|------|
| `-32700` | Parse Error (JSON payload parse failure) |
| `-32600` to `-32603` | Standard request errors (Invalid Request / Method not found / Invalid params / Internal) |
| `-32000` to `-32099` | HESTIA common (Timeout / NotFound / AlreadyExists / PermissionDenied / InvalidState etc.) |
| `-32100` to `-32199` | ai-conductor (Orchestration / Agent mgmt / Spec-driven / Version tracking / LLM) |
| `-32200` to `-32299` | fpga-conductor (Synthesis / Implementation / Bitstream / Timing / Debug / HLS / Device / Simulation / Constraints / Adapter) |
| `-32300` to `-32399` | asic-conductor (RTL Synth / Floorplan / Placement / CTS / Routing / etc.) |
| `-32400` to `-32499` | pcb-conductor (Schematic / DRC/ERC / BOM/Placement / Gerber / AI Synthesis / KG / Constraint Verify) |
| `-32500` to `-32599` | debug-conductor (JTAG / SWD / Session / Waveform / Programming / Signal / Trigger / Reset / Protocol) |
| `-32600` to `-32699` | rag-conductor (Ingest / PDF / Web / Quality gate / Chunk/Embed / Vector/Search / License/PII / Scheduler / Cache) |

Error response `data` should include `tool` / `exit_code` / `log_path` / `errors[]` / `retry_possible` / `suggested_action`.

---

## 5. conductor-core Common API

```rust
pub trait ConductorRpc {
    // Project management
    async fn project_open(&self, path: String) -> ProjectInfo;
    async fn project_targets(&self) -> Vec<Target>;
    async fn project_files(&self) -> FileTree;

    // Build
    async fn build_start(&self, target: String, steps: Vec<BuildStep>) -> BuildJobId;
    async fn build_cancel(&self, job_id: BuildJobId) -> ();
    async fn build_status(&self, job_id: BuildJobId) -> BuildStatus;

    // Reports
    async fn report_timing(&self, job_id: BuildJobId) -> TimingReport;
    async fn report_resource(&self, job_id: BuildJobId) -> ResourceReport;
    async fn report_messages(&self, job_id: BuildJobId) -> Vec<AnnotatedMessage>;

    // Programming
    async fn program_targets(&self) -> Vec<ProgramTarget>;
    async fn program_flash(&self, target: String, bitfile: String) -> ();

    // Toolchain
    async fn toolchain_list(&self) -> Vec<ToolInstall>;
    async fn toolchain_install(&self, id: String) -> InstallProgress;
    async fn toolchain_select(&self, target: String, version: String) -> ();

    // Agent
    async fn agent_status(&self) -> AgentSystemStatus;
    async fn agent_patch_list(&self) -> Vec<PatchProposal>;
    async fn agent_apply_patch(&self, patch_id: String) -> ();
    async fn agent_reject_patch(&self, patch_id: String, reason: String) -> ();

    // Container
    async fn container_list(&self) -> Vec<ContainerInfo>;
    async fn container_start(&self, id: String) -> ();
    async fn container_stop(&self, id: String) -> ();
    async fn container_update(&self, id: String) -> UpdateResult;

    // System
    async fn system_readiness(&self) -> ReadinessStatus;
    async fn system_health(&self) -> HealthStatus;
}
```

---

## 6. agent-cli Endpoint Configuration

### 6.1 Overview

Each conductor is launched as an agent-cli process. agent-cli is a standalone Rust CLI binary that provides Claude Code-equivalent tool/thinking capabilities. The backend LLM is configured in `config.toml::[agent_cli]`.

Four backend options are available:
- Anthropic Claude (default)
- OpenAI Codex
- Ollama (local)
- llama.cpp (OpenAI compatible)

### 6.2 `[agent_cli]` Schema

```toml
[agent_cli]
backend = "claude"                            # "claude" | "codex" | "ollama" | "llama_cpp"
binary_path = ""                              # Empty = $PATH resolution / full path can be specified
anthropic_base_url = ""                       # Empty = Anthropic official
anthropic_api_key_env = "ANTHROPIC_API_KEY"   # Environment variable name storing the API key
model = "claude-opus-4-7"                     # LLM model identifier
max_tokens = 4096                             # Response token limit
registry_dir = ""                             # Inter-agent IPC registry (empty = $XDG_RUNTIME_DIR/agent-cli)
```

### 6.3 Environment Variable Forwarding

1. Read `config.toml`
2. Retrieve the environment variable specified by `anthropic_api_key_env` from the host (unset → fail-fast)
3. If `anthropic_base_url` is non-empty, inject `ANTHROPIC_BASE_URL` into the child process
4. Inject the API key as `ANTHROPIC_API_KEY` into the child process
5. Launch `agent-cli` as a child process

### 6.4 Security Considerations

- **No plaintext API keys**: Never write API keys directly in `config.toml`
- **Environment variable only**: Specify the environment variable name via `anthropic_api_key_env`
- **Explicit error on unset**: Fail-fast before startup if the environment variable is missing
- **Logging masking**: Display only the length, not the actual API key value in logs
- **IPC registry**: Created with permissions 0700 to prevent impersonation by other users

### 6.5 Engine Abstraction (Phase 125)

`conductor_sdk::transport::AgentCliClient` uses `HESTIA_ENGINE_BINARY` env as the sole source of truth to select engine-specific transport:

| Engine | Default registry path | Send transport |
|--------|------------------|---------------|
| `agent-cli` (default) | `$XDG_RUNTIME_DIR/agent-cli/` | Unix socket round-trip |
| `claude-cli-shim` | `~/.local/share/claude-cli-shim/registry/` | `<engine_bin> send <peer> <text>` subprocess (returns synthesized OK response due to FIFO unidirectional) |

If `config.agent_cli_registry_dir` is explicitly specified, it takes highest priority. See `.aiprj/AI_PRJ_DESIGN.md` §10-§12 and Phase 125 commit `af52ffe` for details.

### 6.6 Concurrency Control (Phase 126)

The ai-conductor's `dispatch_to_conductor` loop and each conductor's `dispatch_coders.v1` handler cap concurrency via `conductor_sdk::concurrency::ConductorLimiter` (tokio::sync::Semaphore + acquire timeout). See [`user_guide.md`](user_guide.md) §3.12 for details.

---

## 7. Implementation Image

Each conductor is launched as a single process:

```bash
agent-cli run --persona-file ./.hestia/personas/<conductor>.md --name <conductor>
```

The persona file declares a "structured message handler tool" and a "natural language response tool." The frontend (VSCode / Tauri / CLI) also joins as an agent-cli peer and connects to ai-conductor via `agent-cli send ai <payload>`.

### Design Benefits

- Communication paths are consolidated to a single agent-cli IPC channel, simplifying operations and fault isolation
- All agents are equally discoverable via the peer model
- Structured calls and natural language coexist on the same channel
- Developers can directly interact with any conductor via `agent-cli list` / `agent-cli send`
- Each conductor's LLM backend can be switched individually or in bulk

---

## Related Documentation

- [architecture_overview.md](architecture_overview.md) — Architecture overview
- [glossary.md](glossary.md) — Glossary
- [common/agent_cli_messaging.md](common/agent_cli_messaging.md) — agent-cli messaging details
- [common/api_versioning.md](common/api_versioning.md) — API versioning details
- [common/error_registry.md](common/error_registry.md) — Error code registry