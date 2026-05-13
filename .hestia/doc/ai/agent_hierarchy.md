# ai-conductor Sub-Agent Hierarchy

**Target Conductor**: ai-conductor
**Source**: Design Specification §3.10 (around lines 1189-1239)

## Overview

ai-conductor has two types of sub-agents to assist with §3.3.1 task routing and §3.6 SpecDriven. Each sub-agent is launched as an independent agent-cli process (§20) and coordinates with the ai-conductor body (peer name `ai`) via `agent-cli send <peer>` IPC (§2.3).

## Sub-Agent List

| Sub-Agent | peer Name | Role | Multiplicity | Persona File |
|----------------|---------|------|-------|-----------------|
| **planner** | `ai-planner` | Decomposition and execution planning of frontend instructions (DAG creation, dependency analysis, dispatch strategy to downstream conductors) | 1 (N parallel instances possible under high load) | `.hestia/personas/ai-planner.md` |
| **designer** | `ai-designer` | Creation of overall specifications based on frontend instructions (DesignSpec, HW/SW integration high-level design, inter-conductor coordination contracts) | 1 | `.hestia/personas/ai-designer.md` |

## Coordination Flow

```
[Frontend (VSCode/Tauri/CLI)]
       │
       │ agent-cli send ai <payload>
       ▼
[ai-conductor (peer "ai")]
       │
       ├── Internal delegation 1: agent-cli send ai-planner '{"method":"plan.v1.create",...}'
       │       │
       │       ▼
       │   [planner sub-agent]
       │       ↓ Plan response (DAG / Step list / downstream conductor assignment)
       │
       ├── Internal delegation 2: agent-cli send ai-designer '{"method":"design.v1.create",...}'
       │       │
       │       ▼
       │   [designer sub-agent]
       │       ↓ DesignSpec response (high-level spec / inter-conductor coordination contract)
       │
       ▼
[ai-conductor: integrates planner + designer output → dispatch via conductor-router]
```

## Launch Commands

```bash
# Launch planner / designer simultaneously with ai-conductor startup via agent-cli
agent-cli run --persona-file ./.hestia/personas/ai-planner.md  --name ai-planner  &
agent-cli run --persona-file ./.hestia/personas/ai-designer.md --name ai-designer &
```

## Scaling and Lifecycle

- Both planner and designer are **resident** processes, started and stopped in sync with the ai-conductor lifecycle
- Under high load, multiple planner instances can be launched (peer names `ai-planner-1`, `ai-planner-2`, ...)
- Discoverable via `agent-cli list`, and included in health-checker (§3.3.2) targets

## RAG Integration

When rag-conductor is running (responds `online` to `system.health.v1`), task-router retrieves past similar task cases via `rag.search_similar.v1` and injects them into the planner dispatch context (§13.7.8 self-learning loop).

## ConductorManager and ConductorId

ai-conductor manages 9 downstream conductors (self + 8 downstream) via ConductorManager.

| ConductorId | peer Name | Target |
|-------------|---------|------|
| Ai | `ai` | self (for loopback health check) |
| Rtl | `rtl` | RTL upstream (§4) |
| Fpga | `fpga` | FPGA (§5) |
| Asic | `asic` | ASIC (§6) |
| Pcb | `pcb` | PCB (§7) |
| Hal | `hal` | HAL generation (§8) |
| Apps | `apps` | App FW (§9) |
| Debug | `debug` | Debug (§10) |
| Rag | `rag` | RAG (§13.7) |

## Related Documentation

- [ai/state_machines.md](state_machines.md) — Task state transitions
- [ai/skills_system.md](skills_system.md) — SkillSystem details
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine details
- [../common/sub_agent_lifecycle.md](../common/sub_agent_lifecycle.md) — Sub-agent lifecycle