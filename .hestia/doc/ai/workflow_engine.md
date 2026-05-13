# ai-conductor WorkflowEngine Details

**Target Conductor**: ai-conductor
**Source**: Design Specification §3.5 (around lines 1052-1077), §1.3.5 (around lines 148-177)

## Overview

WorkflowEngine is a DAG (Directed Acyclic Graph)-based cross-conductor pipeline engine. It determines execution order via topological sort using Kahn's algorithm and persists state with sled.

## Crate Structure

```
workflow-engine/
└── src/
    ├── lib.rs      # WorkflowEngine body
    ├── dag.rs      # DAG definition and execution
    └── pipeline.rs # Cross-conductor pipeline
```

## WorkflowStep Structure

```rust
pub struct WorkflowStep {
    pub id: String,              // Step ID
    pub name: String,            // Step name
    pub conductor: String,       // Target conductor (peer name)
    pub method: String,          // agent-cli message method to execute (§14)
    pub params: Option<Value>,   // Parameters
    pub depends_on: Vec<String>, // Dependency step IDs (DAG structure)
    pub status: StepStatus,      // Current state
}
```

## StepStatus States

| State | Description |
|------|------|
| Pending | Dependency steps not yet completed |
| Ready | Dependency steps completed, executable |
| Running | Currently executing |
| Completed | Successfully completed |
| Failed | Failed |
| Skipped | Skipped (due to dependency step failure, etc.) |

## DAG Definition and Topological Sort

Kahn's algorithm resolves dependencies and executes steps from those whose dependencies are satisfied. Steps with satisfied dependencies can run in parallel.

### Workflow Definition Example (YAML format)

```yaml
steps:
  - id: fpga_synth
    conductor: fpga
    method: build/start
    params: { target: artix7 }
  - id: pcb_design
    conductor: pcb
    method: build/start
    depends_on: [fpga_synth]
  - id: debug_setup
    conductor: debug
    method: connect
    depends_on: [fpga_synth, pcb_design]
```

## Diamond Dependency Pattern

Branching and merging patterns are supported.

```
        [A: FPGA Synthesis]
       /              \
[B: ASIC Synthesis]    [C: PCB Design]
       \              /
        [D: Integration Verification]
```

After A completes, B and C run in parallel. D runs after both B and C complete.

## sled Persistence

Workflow execution state (each StepStatus) is persisted to sled (a Rust-native embedded key-value store). This enables:

- Execution state restoration after process restart
- Interruption and resumption of long-running workflows
- Execution history traceability

## Cross-Conductor Pipeline

WorkflowEngine automates coordination across multiple conductors via the `meta.*` method group.

| Method | Description |
|---------|------|
| `meta.dualBuild` | Parallel build across multiple conductors (e.g., fpga.build ‖ asic.synth → meta.collect) |
| `meta.boardWithFpga` | FPGA + PCB integration workflow |
| `meta.handoff` | Inter-conductor handoff event (rtl → fpga/asic, etc.) |

## Execution Flow

```
1. Define workflow as DAG (YAML / structured JSON)
2. WorkflowEngine determines execution order via topological sort
3. Execute steps in order as dependencies are satisfied (parallel execution supported)
4. Each step sends a structured message to the target conductor's agent-cli peer
5. Diamond dependency patterns (branch → merge) are supported
6. Inter-step messages propagate as structured JSON payloads or natural language text
```

## Related Documentation

- [ai/state_machines.md](state_machines.md) — Task state transitions
- [ai/agent_hierarchy.md](agent_hierarchy.md) — Sub-agent hierarchy
- [ai/message_methods.md](message_methods.md) — ai.* / meta.* method list
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli messaging specification