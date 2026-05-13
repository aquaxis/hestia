# Sub-Agent Lifecycle

**Domain**: common — Agent Management
**Source**: Design Specification §3.3, §4, §13.7.7, §20.5

## Overview

Each conductor dynamically starts and stops sub-agents, managing their liveness via `agent-cli list`. Sub-agents operate as independent agent-cli processes and collaborate with their parent conductor via `agent-cli send <peer>` IPC.

## Sub-Agent Startup and Shutdown

### Startup Command

```bash
agent-cli run \
    --persona-file ./.hestia/personas/<peer>.md \
    --name <peer> \
   
```

### Shutdown Conditions

- After assigned task completion and verification
- Termination instruction from parent conductor
- Idle timeout (default: 300 seconds)
- Abnormal termination (detected by health-checker §3.3.2)

### Liveness Management

```bash
agent-cli list    # Get list of active peers
```

The parent conductor periodically (every 30 seconds) runs `agent-cli list` to confirm sub-agent liveness.

## Representative Sub-Agent Configurations

### rtl-conductor

| Sub-agent | Peer Name | Multiplicity | Dynamic Startup |
|----------------|---------|-------|---------|
| planner | `rtl-planner` | 1 | Resident |
| designer | `rtl-designer` | 1 | Resident |
| coder | `rtl-coder-{module}` | N | Dynamic, one per module |
| tester | `rtl-tester` | 1 | Resident |

### fpga-conductor

| Sub-agent | Peer Name | Multiplicity |
|----------------|---------|-------|
| planner | `fpga-planner` | 1 |
| designer | `fpga-designer` | 1 |
| synthesizer | `fpga-synthesizer` | 1 |
| implementer | `fpga-implementer` | 1 |
| tester | `fpga-tester` | 1 |
| programmer | `fpga-programmer` | 1 |

### rag-conductor

| Sub-agent | Peer Name | Multiplicity |
|----------------|---------|-------|
| planner | `rag-planner` | 1 |
| designer | `rag-designer` | 1 |
| ingest | `rag-ingest-{source}` | N (parallel per source)|
| search | `rag-search` | 1 (N under high load)|
| quality_gate | `rag-quality` | 1 |
| archivist | `rag-archivist` | 1 (N under high load)|

## Scaling Policy

| Item | Policy |
|------|---------|
| Resident agents | Synchronized with conductor lifetime (1 instance)|
| Dynamic agents | Start and stop per task count |
| Maximum parallelism | 16 parallel (queue when exceeded)|
| Resource release | Terminate agent-cli process after task completion |

## Workspace

Each sub-agent has a dedicated workspace at `.hestia/workspaces/<peer>/`:

```
.hestia/workspaces/<peer>/
├── requirements.md     # Written by the agent itself via fs_write during setup_ai/update_ai cycle (renamed in Phase 89)
├── design.md           # Same (renamed in Phase 89)
├── tasks.md            # Same, for detailed tasks / DAG (renamed in Phase 89 / state log separated to task_status.md in Phase 107)
├── task_status.md      # Written by the agent itself during exec_job cycle for status only (new in Phase 107)
├── agent.log           # Auto-recorded via agent-cli mirror (Phase 49)
└── (work artifacts)
```

**Important conventions**:

- `<workspace>/instruction.md` placeholder must **not** be generated (deprecated in Phase 92). Instructions are received only via peer prompt
- Startup conventions are referenced from project root `.hestia/rules/{setup_project,update_project,exec_job}.md` (Phases 81-92)
- `.aiprj/` is the exclusive domain of project management AI and must not be created in sub-agent workspaces (Phase 91 convention / runtime integration in Phase 102)
- The 3 documents (`requirements.md` / `design.md` / `tasks.md`) are per-agent, not shared (clarified in Phase 92)
- **Phase 107: Responsibility separation between `tasks.md` and `task_status.md`**
  - `tasks.md` = written by the agent itself during setup_ai/update_ai cycle for detailed tasks / DAG, immutable during exec_job
  - `task_status.md` = written by the agent itself during exec_job cycle for the status of assigned tasks ("Not Started", "In Progress", "Completed", "Blocked")
  - Resolves the conflict where Phase 106 incorrectly judged `task.md` (from Phase 103 = state log) and `tasks.md` (from Phase 89 = 3 documents) as "semantically identical" and merged them, causing `tasks.md` DAG to be full-overwritten by status updates

## Health Check Target

All sub-agents are included in ai-conductor's health-checker (§3.3.2) targets. `system.health.v1` is polled every 30 seconds, with automatic restart on anomaly (max 3 times).

## Liveness Monitoring Loop (Phase 108)

When `hestia start ai` is executed, alongside the agent-cli LLM peer and resident sub-agents (ai-designer / ai-reviewer), the **`hestia monitor-daemon`** child process is automatically spawned (same pattern as `mirror` helper). The monitoring daemon operates independently of the ai-conductor LLM peer, polling the liveness of subordinate sub-agents + starting domain conductors at 30-second intervals.

### Monitoring Targets

- ai-designer / ai-reviewer (resident sub-agents)
- Among rtl / fpga / asic / pcb / hal / apps / debug / rag, those that are started
- `ai` itself is excluded from monitoring targets (no self-monitoring)

### Classification Logic

| State | Treatment in monitoring loop |
|-----|------------------|
| BUSY    | Running |
| WAITING | Running |
| STARTING| Running (prevents false restart immediately after startup) |
| IDLE    | Treated as stopped |
| ERROR   | Treated as stopped |
| UNKNOWN | Treated as stopped |
| Process absent | Treated as stopped |

"All stopped" means **all monitored targets** are treated as stopped **within the same cycle**. If even one is running, wait until the next cycle.

### Pending Task Detection

Read `<workspace>/<peer>/task_status.md` via fs_read for each peer. If any line has a status of "Not Started", "In Progress", or "Blocked", it is considered to have pending tasks. If `task_status.md` is absent, it is treated as having no pending tasks (prioritize false suppression / avoid false restarts).

### Resume Instruction

Send `agent-cli send <peer> "<instruction text>"` to instruct the target peer to resume according to the "Resume Work" section of its persona. The instruction text should include a directive to fs_read `<workspace>/<peer>/{tasks.md, task_status.md}` and resume from unfinished tasks. After sending the instruction, observe a **60-second cooldown** with no additional sends.

### Related CLI Commands

- `hestia monitor` (Phase 108): CLI for human users that periodically updates and displays liveness status (`--interval N` for update interval, `--once` for single output, `--all` to show SKILLS column). Operates independently of ai-conductor's monitoring loop and does not send resume instructions.
- `hestia status`: Existing single-output command. Equivalent to `hestia monitor --once`.
- `hestia monitor-daemon`: (Internal / hidden) Automatically spawned by `hestia start ai`. Killed along with agent-cli / mirror via `hestia kill` with SIGKILL.

### Environment Variables

| Variable | Default | Range | Purpose |
|------|------|------|------|
| `HESTIA_MONITOR_INTERVAL_SECS` | 30 | 5..=600 | Monitoring cycle |
| `HESTIA_MONITOR_COOLDOWN_SECS` | 60 | 0..=600 | Cooldown after resume instruction |
| `HESTIA_MONITOR_DISABLED` | unset | `1` disables monitoring loop | For verification / debug |

### Implementation

- `.hestia/tools/clis/hestia/src/monitor.rs` — Pure functions (`is_all_stopped` / `resolve_monitor_targets` / `parse_task_status` / `has_pending_tasks` / `summarize_statuses`, etc.) + `run_monitor_daemon()` / `run_monitor()` + 22 unit tests.
- `.hestia/tools/clis/hestia/src/main.rs` — `Commands::Monitor` / `Commands::MonitorDaemon` / child process spawn in `start_conductor("ai")` / addition of `hestia monitor-daemon` to `KILL_PATTERNS`.

## Auto-Termination Logic (Phase 109)

The monitoring daemon evaluates the following auto-termination logic within the same cycle, in addition to the Phase 108 "resume instruction" logic. The evaluation order is **(1) sub-agent termination -> (2) conductor termination -> (3) existing resume instruction**, in 3 stages.

### Sub-Agent Termination

If all lines in a peer's `<workspace>/<peer>/task_status.md` are "Completed" and the peer's status is IDLE / ERROR / UNKNOWN, send SIGTERM to that peer (graceful termination). If the process is still alive after `HESTIA_MONITOR_TERMINATE_GRACE_SECS` (default 10 seconds, clamped to 0..=60) grace period, escalate to SIGKILL.

The following are both included as targets:

- Static sub-agents: `ai-designer` / `ai-reviewer`
- Dynamic sub-agents: `<domain>-*` pattern (e.g., `rtl-coder-uart` / `asic-signoff` / `hal-designer`)

### Domain Conductor Termination (Order Guarantee)

A domain conductor is terminated only when all of the following are satisfied:

1. All lines in that conductor's `task_status.md` are "Completed".
2. No dynamic sub-agents (`<domain>-*` peers) for that conductor exist in `agent-cli list`.

For order guarantee, conductors are not terminated while sub-agents remain. Sub-agents are terminated in (1) -> disappear from `agent-cli list` in the next cycle -> conductors are terminated in (2) in a subsequent cycle, spanning 2 cycles.

### ai-conductor Exclusion

The peer name `ai` is excluded from this logic. Termination is only via explicit user `hestia stop ai` or `hestia kill`.

### Duplicate Spawn Prevention (Phase 109 related)

`spawn_agent_cli` and `start_conductor` check `agent-cli list` immediately before spawning, and if the target peer name is already registered, they log a warning and skip. The `hestia monitor-daemon` child process spawn also prevents duplicates via `pgrep -f "hestia monitor-daemon"`. This prevents peer duplication from multiple `hestia start ai` executions (such as the ai-reviewer x 5 situation observed in the Phase 108 smoke test) from recurring.

### Environment Variables (Phase 109)

| Variable | Default | Range | Purpose |
|------|------|------|------|
| `HESTIA_MONITOR_TERMINATE_GRACE_SECS` | 10 | 0..=60 | SIGTERM -> SIGKILL escalation grace seconds |

### Implementation (Phase 109)

- `.hestia/tools/clis/hestia/src/monitor.rs` — Added `classify_peer` / `peer_tasks_all_complete` / `conductors_ready_to_terminate` / `is_terminable_status` / `terminate_peer` / `pgrep_agent_cli_pids`, added `MonitorTarget.parent_conductor` field, extended `run_monitor_daemon` to 3-stage processing. 16 new unit tests added.
- `.hestia/tools/clis/hestia/src/main.rs` — Added `registered_peer_names` pure function / `peer_already_registered` / `monitor_daemon_already_running` helpers, added duplicate checks to 3 paths: `spawn_agent_cli` / `start_conductor` / monitor-daemon spawn. 6 new unit tests added.

## Rescue Logic (Phase 110)

When a peer that received a `agent-cli send` via the Phase 108 "all stopped + pending tasks -> batch resume instruction" path has not transitioned to a running state within the specified time, and the number of unfinished tasks in `task_status.md` has not changed, the monitoring daemon executes the following rescue path.

### Rescue Procedure

1. **Immediate SIGKILL**: Kill all PIDs extracted via `pgrep -f "agent-cli run.*--name <peer>"` with SIGKILL (unlike the SIGTERM -> grace -> SIGKILL approach in Phase 109, this is an immediate kill)
2. **Wait for deregistration**: Poll `agent-cli list` for up to 10 seconds until the peer disappears
3. **Persona name resolution**: Derive persona file name from peer name
   - `<peer>.md` directly (e.g., `ai` -> `ai.md`, `rtl` -> `rtl.md`)
   - `<domain>-coder-<module>` -> `<domain>-coder.md` (dynamic sub-agent)
   - `asic-signoff` -> `asic-signoff-checker.md` (known exception, HD-033)
4. **Re-spawn**: Restart the peer via `spawn_agent_cli` (passes duplicate check)
5. **Wait for registry registration**: Up to 15 seconds
6. **Send Update Project instruction**: Send via `agent-cli send <peer>` with instructions to fs_read `<root>/.hestia/rules/update_project.md` + follow conventions + reference `tasks.md` / `task_status.md` + resume unfinished tasks

### Rescue Suppression (Infinite Loop Prevention)

| Control | Default | Environment Variable | Range |
|------|------|---------|------|
| Normal peer timeout | 120 seconds | `HESTIA_MONITOR_RESCUE_TIMEOUT_SECS` | 30..=600 |
| ai-conductor timeout | 180 seconds | `HESTIA_MONITOR_AI_RESCUE_TIMEOUT_SECS` | 60..=600 |
| Post-rescue cooldown | 300 seconds | `HESTIA_MONITOR_RESCUE_COOLDOWN_SECS` | 60..=3600 |
| Per-peer attempt limit | 3 times | `HESTIA_MONITOR_RESCUE_MAX_ATTEMPTS` | 1..=10 |

After reaching the limit, subsequent rescue attempts for that peer are stopped with only a warning log (waiting for human intervention).

### ai-conductor Rescue

The monitoring daemon (`hestia monitor-daemon`) is an **independent process** started as a child of the ai-conductor, so even when ai-conductor is unresponsive, the monitoring daemon itself continues to run. This means ai-conductor is also included as a rescue target.

- Monitoring target classification: Handled by the new `MonitorKind::AiConductor` variant (`classify_peer("ai")` changed to return Some in Phase 110)
- Phase 109 auto-termination target: Excluded (SIGTERM on task completion does not apply)
- Phase 108 batch resume instruction target: Included
- Phase 110 rescue target: Included (default timeout 180s)
- ai-conductor persona name resolution: `resolve_persona_for_peer("ai")` -> `Some("ai")` resolving to `.hestia/personas/ai.md`

### `hestia status` STATUS Column Extension (Phase 110)

Extended the `AgentStatus` enum with new display variants `THINK` / `WAIT`:

| Variant | Display | Meaning |
|------|------|------|
| `Idle` | `IDLE` | Last event is `assistant` (response completed) |
| `Busy` | `BUSY` | Last event is `tool_call` / `tool_result` and recent (tool executing) |
| `Think` | `THINK` | Last event is `thinking` and recent (thinking, new in Phase 110) |
| `Waiting` | `WAIT` | Last event is `user` (user prompt received, assistant not yet responding, formerly `WAITING`) |
| `Error` | `ERROR` | Last `tool_result` has `ok=false` |
| `Starting` | `STARTING` | jsonl not yet generated / just started |
| `Unknown` | `UNKNOWN` | Parse failure |

In the monitoring loop, `Think` is treated the same as `Busy` / `Waiting` / `Starting` as **running** (excluded from "stopped" in `is_all_stopped`, treated as running in `is_terminable_status` / `needs_rescue`). Add `THINK: N` count to the `hestia monitor` summary line.

### Exclusivity with Existing Phase 108 / 109

| State | Applied phase |
|------|----------|
| Tasks complete + IDLE (DomainConductor / Subagent) | (1) / (2) SIGTERM |
| Tasks complete + IDLE (AiConductor) | Do nothing (explicit stop only) |
| Pending tasks + IDLE + resume not sent | (4) Batch resume instruction |
| Pending tasks + IDLE + resume sent + within timeout | Do nothing (re-evaluate next cycle) |
| Pending tasks + IDLE + timeout exceeded + within limit | (3) rescue |
| BUSY / THINK / WAIT | Do nothing (treated as running) |

The evaluation order within a single monitoring loop cycle is **(1) -> (2) -> (3) -> (4)** (Phase 110 (3) rescue inserted into existing Phase 108 / 109).

### Implementation (Phase 110)

- `.hestia/tools/clis/hestia/src/monitor.rs` — Added `MonitorKind::AiConductor`, `ResumeAttempt` / `RescueAttempt` structs, `needs_rescue` / `rescue_allowed` / `count_pending_tasks` / `record_resume` / `resolve_persona_for_peer` / `build_rescue_message` pure functions, `kill_peer_now` / `wait_for_deregistration` / `rescue_peer` async functions, 4 environment variables + clamp helpers, added Phase 110 (3) rescue evaluation block to `run_monitor_daemon`. 28 new unit tests.
- `.hestia/tools/clis/hestia/src/main.rs` — Added `AgentStatus::Think`, changed `Waiting::as_str` to `"WAIT"`, added thinking branch to `derive_status_from_log`, made `registered_peer_names` / `spawn_agent_cli` `pub(crate)` (called from monitor.rs). Existing test modifications + 2 new tests.
- `.hestia/personas/ai.md` — Added Phase 110 responsibilities (4 items) / prohibitions (4 items).

## Related Documents

- [backend_switching.md](backend_switching.md) — LLM backend switching
- [health_check_orchestration.md](health_check_orchestration.md) — Health checks
- [conductor_startup.md](conductor_startup.md) — Daemon startup order