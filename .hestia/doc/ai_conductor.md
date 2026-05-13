# ai-conductor Overview — Meta-Orchestrator

**Scope**: ai-conductor (meta-orchestrator)
**Source**: Design specification §3 (lines 745-1240)

---

## Overview

ai-conductor is the top-level conductor in HESTIA, serving as the sole entry point from the frontend (VSCode / Tauri / CLI). It orchestrates the 8 subordinate conductors (rtl / fpga / asic / pcb / hal / apps / debug / rag) and uses AI to orchestrate the entire hardware development process.

ai-conductor itself is launched as an agent-cli process (peer name `ai`), and all communication with downstream conductors uses agent-cli native IPC.

---

## Core Functions

ai-conductor provides the following four core functions.

| Function | Role |
|------|------|
| **Task decomposition and routing** | Understands natural language or structured instructions from the frontend, decomposes tasks, and routes them to the appropriate subordinate conductor (task-router) |
| **Health check** | Periodically polls all conductors (default 30-second interval), aggregating Online / Offline / Degraded / Upgrading states. Automatically restarts (max 3 times) or escalates to the frontend on failure |
| **Skill management** | Registers specialized skills (HDL generation, constraint generation, testbench generation, etc.) as plugins in SkillRegistry and provides them to subordinate conductor agent-cli personas |
| **Container management** | Automatic Containerfile generation, build, differential update, provisioning, and registry management based on `container.toml` declarations (only when container execution is selected) |

---

## Auxiliary Functions

| Function | Description |
|------|------|
| Sustainable upgrade | Automated tool version upgrades via WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent |
| DAG-based workflow | Cross-conductor pipelines via topological sort (Kahn), with state persistence in sled |
| Spec-driven development | Generates DesignSpec from natural language specifications (`REQ:`/`CON:`/`IF:` prefixes) → automated design data generation |
| LLM backend switching | Supports switching between Anthropic / Ollama / LM Studio / vLLM |
| **Sub-agent concurrency control** (Phase 126) | Caps concurrency at global / ai-dispatch / per-conductor levels via a 3-tier hierarchical Semaphore + acquire timeout. Reserved slot for reviewer prevents starvation of auto-spawned ai-reviewer. See [`user_guide.md`](user_guide.md) §3.12 for details |

---

## ConductorManager

The core structure that manages the lifecycle of all conductors. It identifies the 8 subordinate conductors with the `ConductorId` enum and tracks their state using the `ConductorStatus` enum (Online / Offline / Degraded / Upgrading).

```rust
pub struct ConductorManager {
    conductors: Arc<RwLock<HashMap<ConductorId, ConductorInfo>>>,
    pub config: OrchestratorConfig,
}
```

---

## Task Routing Flow

1. **Intent understanding**: Natural language → design task type classification / Structured JSON → direct classification via method namespace
2. **Task decomposition**: Single conductor → dispatch directly / Cross-conductor → delegate to workflow-engine and DAG-ify / Specification-based → generate DesignSpec via spec-driven
3. **Routing**: Route to the appropriate conductor via conductor-router using `agent-cli send <peer> <payload>`

### Dispatch Concurrency (Phase 126)

Each step in the dispatch loop within `AiHandler::handle_exec` acquires a permit from the L2 limiter (`HESTIA_AI_DISPATCH_MAX`, default 2, `tokio::sync::Semaphore`-based) before executing `spawn_conductor_on_demand` + `dispatch_to_conductor`. If a permit cannot be acquired within `HESTIA_ACQUIRE_TIMEOUT_SECS` (default 600), the step is recorded with a `dispatch_acquire_timeout` error and the next step proceeds (aborting hold-and-wait to detect deadlocks).

`AgentManager` (in the internal `multi-agent` crate) was also replaced in Phase 126 with a `conductor_sdk::concurrency::ConductorLimiter`-based implementation, capping globally at `HESTIA_GLOBAL_MAX_AGENTS` (default 8), with 1 slot reserved for `ai-reviewer`.

---

## Startup Sequence

- **Group 0**: ai-conductor (highest priority, serial startup)
- **Group 1**: rtl / fpga / asic / pcb / hal / apps / debug / rag (8 in parallel, after ai readiness confirmed)

---

## Sub-agents

| Sub-agent | Peer name | Role | Multiplicity |
|----------------|---------|------|-------|
| planner | ai-planner | Task decomposition and execution planning (DAG-ification, dispatch strategy) | 1 (N in parallel under high load) |
| designer | ai-designer | Overall specification (DesignSpec, HW/SW integration high-level design) creation | 1 |

---

## Related Documentation

- [master_agent_design.md](master_agent_design.md) — ai-conductor detailed design (full version)
- [rtl_conductor.md](rtl_conductor.md) — RTL design flow orchestrator
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator
- [pcb_conductor.md](pcb_conductor.md) — PCB design flow orchestrator
- [hal_conductor.md](hal_conductor.md) — HAL generation orchestrator
- [apps_conductor.md](apps_conductor.md) — Application software development orchestrator
- [debug_conductor.md](debug_conductor.md) — Debug environment orchestrator
- [rag_conductor.md](rag_conductor.md) — Knowledge base orchestrator