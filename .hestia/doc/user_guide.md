# User Guide

**Scope**: Hestia overall
**Source**: Design specification §15 (CLI execution functions), §16 (Frontend)

---

## 1. Introduction

Hestia is an integrated hardware development environment where 9 Conductors (ai / rtl / fpga / asic / pcb / hal / apps / debug / rag) coordinate to orchestrate FPGA, ASIC, and PCB design flows. This guide explains how to use the CLI, VSCode Extension, and Tauri IDE.

---

## 2. Installation

### 2.1 Build Instructions

```bash
# Workspace (.hestia/tools/Cargo.toml, resolver = "2")
cd .hestia/tools

# All binaries (9 conductors + 10 CLIs)
cargo build --release

# Build specific conductor only
cargo build --release -p hestia-fpga-conductor

# Run tests
cargo test                          # All tests
cargo test -p hestia-fpga-conductor # Specific conductor
```

### 2.2 Daemon Startup Sequence

```bash
# Group 0 (serial, highest priority)
hestia-ai-conductor &
# Wait until system.health.v1 returns status="online"
hestia status --conductor ai

# Group 1 (8 in parallel)
hestia-rtl-conductor &
hestia-fpga-conductor &
hestia-asic-conductor &
hestia-pcb-conductor &
hestia-hal-conductor &
hestia-apps-conductor &
hestia-debug-conductor &
hestia-rag-conductor &

# Recommended: via systemd user units
systemctl --user start hestia-ai hestia-rtl hestia-fpga hestia-asic \
  hestia-pcb hestia-hal hestia-apps hestia-debug hestia-rag
```

---

## 3. CLI Usage

### 3.1 Unified Runner (hestia)

```bash
hestia init                     # Build .hestia/ structure
hestia start                    # Start all 9 conductors
hestia start fpga               # Start specified conductor only
hestia status                   # Show all conductor status
hestia kill                     # Stop all conductors and cleanup residual peers from
                                # agent-cli/claude-cli-shim registry (Phase 123)
hestia upgrade                  # cargo build --release from source → reinstall to ~/.local/bin
                                # (Phase 124)
hestia upgrade --no-pull        # Skip git pull and rebuild from current working tree
hestia --version                # Version display based on git describe via build.rs
                                # Example: "hestia 0.1.5-17-gaf88400" (Phase 127)
```

#### `hestia upgrade` Behavior (Phase 124 / 130)

1. Pull remote changes with `git pull --ff-only` (skippable with `--no-pull`)
2. Run `cargo build --release` in `.hestia/tools/` (**expanded to build all 20 binaries in Phase 130**)
3. Copy all **20 built binaries** (`hestia` / 9 conductors / 10 CLIs / `claude-cli-shim`) to `~/.local/bin/`
4. Display the new version with `hestia --version` after installation

Before Phase 130, only the `hestia` binary was installed, which meant fixes to conductors and libraries were not reflected. Phase 130 changed this to install all binaries, so `hestia upgrade` now synchronizes the entire system in one command.

#### `hestia --version` Version Synchronization (Phase 127)

`clis/hestia/build.rs` runs `git describe --tags --always --dirty=-dirty` at build time and injects it as the `HESTIA_BUILD_VERSION` env variable. `main.rs` displays it using the following fallback chain:

- `option_env!("HESTIA_BUILD_VERSION")` — `git describe` output when git is available
- `env!("CARGO_PKG_VERSION")` — `[workspace.package] version` when git is unavailable (e.g., `cargo install` distribution)

This ensures GitHub TAG and version display are automatically synchronized. At release time, `scripts/release.sh <X.Y.Z>` atomically performs Cargo.toml rewrite + commit + tag.

### 3.2 RTL Design Flow

```bash
hestia rtl init                                             # rtl.toml template
hestia rtl lint                                             # Lint with Verilator/Verible
hestia rtl simulate --tb tb_alu --simulator verilator
hestia rtl formal --properties properties.sv               # SymbiYosys
hestia rtl handoff --target fpga                           # Handoff to downstream
```

### 3.3 FPGA Design Flow

```bash
hestia fpga init                                            # fpga.toml template
hestia fpga build artix7                                   # Start build
hestia fpga status --job-id 1
hestia fpga report timing
hestia fpga simulate --tb tb_top --simulator iverilog
```

### 3.4 ASIC Design Flow

```bash
hestia asic init
hestia asic pdk install sky130A
hestia asic build --pdk sky130A
hestia asic advance --job-id 1                              # Advance 13 steps one at a time
```

### 3.5 PCB Design Flow

```bash
hestia pcb init
hestia pcb build --board-name "sensor-board"                # AI schematic synthesis
hestia pcb output kicad --output-dir ./out
hestia pcb output gerber --output-dir ./gb
```

### 3.6 HAL Generation Flow

```bash
hestia hal init                                             # hal.toml template
hestia hal parse regs/soc.rdl                               # SystemRDL register map parsing
hestia hal validate                                         # Address overlap and type consistency check
hestia hal generate c --output-dir build/hal/inc           # C header generation
hestia hal generate rust --output-dir build/hal/rust       # Rust crate generation
hestia hal generate svd --output build/hal/svd/soc.svd    # CMSIS SVD
hestia hal export-rtl --target rtl-conductor               # SystemRDL export
```

### 3.7 Application Development Flow

```bash
hestia apps init                                            # apps.toml template
hestia apps build --target thumbv7em-none-eabihf           # Cross-compilation
hestia apps test sil                                       # QEMU SIL test
hestia apps test hil --probe stlink-v3                     # Hardware HIL test
hestia apps size                                           # Binary size analysis
hestia apps flash --probe stlink-v3                        # Flash write
```

### 3.8 Debug Flow

```bash
hestia debug create STM32F407 --interface-type swd
hestia debug connect --session-id 1
hestia debug capture start --session-id 1 --duration-ms 1000
hestia debug program --board fpga_board --bitstream out.bit
```

### 3.9 RAG (Knowledge Search)

```bash
hestia rag ingest --source-id stm32_datasheet                # Ingest PDF/Web sources
hestia rag search "STM32F103 SPI pin configuration" --top-k 5
hestia rag cleanup                                            # Cleanup quarantine / old caches
```

### 3.10 AI Agent

```bash
hestia ai exec "Create a UART LED control circuit on Artix-7"  # Natural language job
hestia ai run --file .aiprj/instructions.md                  # Spec-driven execution
hestia ai container ls
hestia ai container create --conductor fpga --tool vivado:2025.2
hestia ai workflow run --workflow fpga-to-pcb-test-board
hestia ai review start --project ./my-project --target artix7
```

### 3.11 Common Options

`CommonOpts`: `--output (human|json)` / `--timeout` / `--registry` / `--config` / `--verbose`

### 3.12 Sub-agent Concurrency Control (Phase 126)

Configure sub-agent spawn limits via the `HESTIA_*` environment variables listed below. A 3-tier hierarchical Semaphore + acquire timeout prevents deadlocks while avoiding PC / LLM overload.

| Environment variable | Default | Role |
|---------|------|------|
| `HESTIA_GLOBAL_MAX_AGENTS` | 8 | ai-conductor `AgentManager` cap (1 slot reserved for reviewer) |
| `HESTIA_AI_DISPATCH_MAX` | 2 | ai-conductor `dispatch_to_conductor` concurrent execution limit |
| `HESTIA_PER_CONDUCTOR_MAX` | 4 | Per-conductor `dispatch_coders.v1` parallelism limit |
| `HESTIA_ACQUIRE_TIMEOUT_SECS` | 600 | Common acquire timeout for all limiters |

Set these in the environment of the parent process before running `hestia start`. Conductor processes inherit them at spawn time and read them via `ConductorLimiter::from_env` / `AgentManager::with_default_cap`. When unset, library defaults (above) apply.

> **Phase 131 alive cap semantics (enforced across all spawn paths)**:
> `per_conductor_max` is enforced as the "absolute upper limit on the number of currently alive target sub-agents"
> across **all paths** of `hestia spawn-subagent`.
>
> The enforcement point is `spawn_agent_cli` (the implementation body of `hestia spawn-subagent`).
> If the peer name has 3 or more segments in the `<conductor>-<role>-<module>` format,
> it uses `<conductor>-<role>-` as the cap prefix, queries the engine registry for the alive count,
> and if `alive >= cap`, it **refuses the spawn with `bail!`**. This ensures the cap is effective
> across all of the following paths:
>
> 1. `rtl.dispatch_coders.v1` / `apps.dispatch_coders.v1` RPC path
> 2. Path where persona LLM directly calls `hestia spawn-subagent`
> 3. Manual `hestia spawn-subagent --persona X --name Y` CLI execution
>
> For concurrent `hestia spawn-subagent` calls, `~/.local/share/hestia/spawn.lock`
> (via `flock(2)`) serializes them and prevents TOCTOU races.
>
> Peer names with 2 or fewer segments (e.g., `pcb-layout`, `ai-reviewer`) are assumed to be
> single-instance and are not subject to caps.
>
> Phase evolution history: Phase 126-128 (single-call cap) → Phase 129 (handler-level alive cap) →
> **Phase 131 (spawn all-paths alive cap, closing persona-based bypass)**.

**Deadlock Avoidance Mechanism:**

- **Fixed acquisition order (L1 → L2 → L3)**: Permits are acquired only in the order `AgentManager` global → ai-conductor dispatch → per-conductor cap, eliminating circular wait.
- **Acquire timeout**: On timeout expiration, record a `dispatch_acquire_timeout` error and proceed to the next step, aborting hold-and-wait.
- **Reviewer reserved slot**: Reserve 1 slot out of `global_max` for `ai-reviewer`, preventing starvation of Phase 77's auto-spawned ai-reviewer under cap limits.

**Usage Example** — Temporarily reduce parallelism to 1 for debugging:

```bash
HESTIA_PER_CONDUCTOR_MAX=1 HESTIA_AI_DISPATCH_MAX=1 \
  hestia ai exec "Write 5 RTL modules"
```

You can verify that the concurrent process count is within the cap with `pgrep -af 'agent-cli|claude-cli-shim' | wc -l`.

### 3.13 Exit Codes

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

---

## 4. VSCode Extension

### 4.1 Installation

Install the VSIX package `hestia-vscode` (publisher: `aquaxis`, engines.vscode >= 1.85.0).

### 4.2 Activation

Auto-activated on the following events:
- `onCommand`: `hestia.start|stop|status|ai|spec|fpga|debug|rag` and 30+ other commands
- `onView`: `hestia-conductor` / `agents` / `specs`
- `onLanguage`: `verilog`, `vhdl`, `systemverilog`, `xdc`, `pcf`, `toml`

### 4.3 Views

| View | Content |
|-------|------|
| ConductorStatusView | Status display for 9 conductors |
| AgentListView | Sub-agent list |
| SpecViewer | Specification viewer |
| DesignFlowView | Design flow visualization |
| LogViewer | Log display |

### 4.4 Configuration

Key configuration items (`hestia.*`):

| Configuration key | Type | Default | Content |
|---------|-----|-------|------|
| `agentCliRegistryDir` | string | `$XDG_RUNTIME_DIR/agent-cli/` | agent-cli registry |
| `autoConnect` | bool | `true` | Auto-connect on startup |
| `reconnectInterval` | number | `3000` | Reconnect interval (ms) |
| `requestTimeout` | number | `30000` | Request timeout (ms) |
| `ai.model` | string | `claude-sonnet-4-6` | LLM model selection |
| `ai.maxTokens` | number | `4096` | Maximum tokens |
| `ai.apiKeyEnv` | string | `ANTHROPIC_API_KEY` | API key environment variable name |
| `ai.baseUrl` | string | `""` | API endpoint URL |

### 4.5 Editor Features

- Monaco Editor integration: HDL highlighting, completion, diagnostics (via HDL LSP Broker)
- Waveform viewer: WASM rendering in WebView

---

## 5. Tauri Desktop App

### 5.1 Configuration

- Version: `0.1.0`
- Identifier: `dev.hestia.ide`
- Bundle targets: `deb`, `rpm`, `appimage`

### 5.2 Windows

| Window | Size | Purpose |
|-----------|-------|------|
| main | 1440x900 | Main IDE |
| waveform | 1200x600 | Waveform viewer |
| settings | 800x600 | Settings panel |

### 5.3 Security

CSP: `connect-src 'self' ipc: ws://localhost:*`

### 5.4 Shell Plugin

10 commands (`hestia` / `hestia-{ai,rtl,fpga,asic,pcb,hal,apps,debug,rag}-cli`) are executable via Tauri Shell.

---

## 6. UI Components (hestia-ui)

React + TypeScript component library:

| Component | Purpose |
|-------------|------|
| ConductorStatusCard | Conductor status display |
| AgentList | Sub-agent list |
| SpecViewer | Specification display |
| LogViewer | Log display |
| WaveformViewer | Waveform display |
| ConfigPanel | Settings panel |
| TaskProgress | Task progress |

Brand colors: Primary akane `#e84d2c`, secondary deep green `#2d8f5e`

---

## Related Documentation

- [architecture_overview.md](architecture_overview.md) — Architecture overview
- [agent_communication.md](agent_communication.md) — Communication specification
- [frontend/cli_clients.md](frontend/cli_clients.md) — CLI detailed specification
- [frontend/vscode_extension.md](frontend/vscode_extension.md) — VSCode extension details
- [frontend/tauri_ide.md](frontend/tauri_ide.md) — Tauri IDE details