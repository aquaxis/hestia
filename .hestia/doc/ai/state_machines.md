# ai-conductor Task State Transitions

**Target Conductor**: ai-conductor
**Source**: Design Specification §3.3.1 (around lines 916-955), §3.3.2 (around lines 957-1010)

## Task Processing State Transitions

ai-conductor's task-router goes through the following state transitions from receiving a frontend instruction to completing dispatch to downstream conductors.

```
[Received] → Intent Classification → Task Decomposition → Routing → Completed
                            │                │
                            │                └→ Failed (conductor unreachable)
                            │
                            └→ Decomposition failed
```

### State Definitions

| State | Description |
|------|------|
| Received | Instruction received from frontend (natural language or structured JSON) |
| IntentClassified | Intent classification complete (design task type determined: fpga build / asic synth / pcb route, etc.) |
| Decomposed | Task decomposition complete (single conductor / cross-conductor DAG / specification-based) |
| Routed | Dispatch to downstream conductor complete (`agent-cli send <peer> <payload>`) |
| Completed | Result aggregation complete → notify frontend |
| Failed | Processing failed (escalation / retry determination) |

### Branching Patterns

**Single conductor completion:**
```
Received → IntentClassified → Decomposed → Routed → Completed
```

**Cross-conductor:**
```
Received → IntentClassified → Decomposed → Delegated to WorkflowEngine → DAG execution → Completed
```

**Specification-based:**
```
Received → IntentClassified → DesignSpec generated via SpecDriven → DAG creation → WorkflowEngine execution → Completed
```

## Health Check State Transitions

health-checker polls all conductors at regular intervals (default 30 seconds) and updates ConductorStatus.

```
           ┌─────────────────────────────────────┐
           │                                     │
           ▼                                     │
  Online ──→ Offline ──→ Auto restart (max 3) ─────┘
    │           │                       │
    │           │                  3 consecutive failures
    ▼           │                       │
  Degraded     │                       ▼
    │           │              Frontend notification
    │           │           (agent.alert.v1)
  Upgrading     │
    │           │
    └───────────┘
```

### ConductorStatus

| State | Description | Transition Trigger |
|------|------|------------|
| Online | Running normally | "online" response within 3 seconds |
| Offline | Stopped | Timeout (3 seconds) |
| Degraded | Degraded state (some features restricted) | "degraded" response |
| Upgrading | Upgrading in progress | "upgrading" response |

### Actions on State Change

| Transition | Action |
|------|----------|
| Online → Offline / Degraded | Observability log + auto restart attempt (max 3) |
| 3 consecutive failures | Frontend notification (`agent.alert.v1`) |
| Upgrading → Online | Success notification to upgrade-manager |
| Any → Persist state history to sled | §19 Observability integration |

## Workflow Execution State Transitions

Step state transitions for DAG-based execution by WorkflowEngine.

| StepStatus | Description |
|-----------|------|
| Pending | Dependency steps not yet completed |
| Ready | Dependency steps completed, executable |
| Running | Currently executing |
| Completed | Successfully completed |
| Failed | Failed |
| Skipped | Skipped (due to dependency step failure, etc.) |

### Diamond Dependency Example

```
        [A: FPGA Synthesis]
       /              \
[B: ASIC Synthesis]    [C: PCB Design]
       \              /
        [D: Integration Verification]
```

A → B, A → C, B → D, C → D. After A completes, B and C run in parallel; D runs after both B and C complete.

## Startup Orchestration

| Group | Conductor | Startup Method |
|-------|-----------|---------|
| Group 0 | ai-conductor | Serial, highest priority |
| Group 1 | rtl / fpga / asic / pcb / hal / apps / debug / rag | 8 parallel (after ai readiness confirmed) |

## Related Documentation

- [ai/message_methods.md](message_methods.md) — ai.* method list
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine details
- [ai/agent_hierarchy.md](agent_hierarchy.md) — Sub-agent hierarchy
- [../common/conductor_startup.md](../common/conductor_startup.md) — Startup sequence details