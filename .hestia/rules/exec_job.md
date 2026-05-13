---
name: Hestia Agent Execution Guidelines
description: Execution rules that hestia agents follow during the exec_job cycle. Adapted from `.aiprj/rules/exec_job.md` for the agent context (Phase 81 P-3).
---

# Hestia Agent Execution Guidelines (Phase 81)

This file defines the rules that hestia agents reference during the **exec_job cycle** (when receiving an execution request prompt from a superior). It is an independent entity in the hestia context, separate from `.aiprj/rules/exec_job.md` which is intended for project management AI.

---

## Article 1: Basis of Work

Agents obtain their work basis in the following priority order:

1. **Persona responsibilities**: The `name` / `description` fields in `.hestia/personas/<self>.md` indicating the scope of responsibilities
2. **Workspace state**: `<workspace>/{requirements,design,tasks}.md` (the three documents generated in the self-execution cycle) and `<workspace>/agent.log` (past interaction history)
3. **Superior prompt**: The `text` field of the most recently received `agent-cli send` message (all superior instructions are consolidated through this channel, Phase 89 terminology unification)

Project management AI documents such as `.aiprj/AI_PRJ_REQUIREMENTS.md` are **not referenced** (Phase 22 P-1 / Phase 81 P-3 rule).

---

## Article 2: Two-Tier Consistency of 3 Documents + Artifacts (Phase 81 Enhancement)

Agents maintain **two-tier consistency** between the 3 workspace documents and project artifacts under the project root:

**First tier (3-document consistency)**:
- `<workspace>/requirements.md` (requirements from received instructions)
- `<workspace>/design.md` (design decisions within the scope of responsibilities)
- `<workspace>/tasks.md` (implementation items and progress)

Agents verify that the content across these 3 documents is not contradictory. For example: if `requirements.md` mentions "UART loopback", then `design.md` should have the UART RX/TX FSM design, and `tasks.md` should have the corresponding implementation task.

**Second tier (artifact consistency)**:
- Register definitions in `hal/register_map.json` must be consistent with port signals in `rtl/<top>.sv`
- Pin constraints in `fpga/constraints/<top>.xdc` must be consistent with top-level I/O in `rtl/<top>.sv`
- Lint results in `sim/lint_report.json` must be used to judge quality gates on the `rtl/` side

If inconsistency is detected, agents **report to the superior with a halt reason** and do not attempt to fix it on their own (responsibility boundary / Phase 53 AI persona correction). Skipping the 3 documents is also treated as inconsistency (Phase 91 mandatory compliance).

---

**Phase 92 Clarification**: The first-tier 3 documents (`requirements.md` / `design.md` / `tasks.md`) are **per-agent exclusive** and exist independently under `.hestia/workspaces/<peer>/`. Sharing or cross-referencing other agents' 3 documents is prohibited (per-agent specification).

---

## Article 3: Progress Recording

Agents do not explicitly record progress via fs_write. Instead:

- agent-cli's structured JSONL logs (`~/.local/share/agent-cli/logs/<peer>/*.jsonl`) automatically record thinking / tool_call / tool_result
- `hestia mirror <peer>` path (Phase 49) mirrors summary lines to `<workspace>/agent.log` in real time
- Aggregate JSON (`<root>/.hestia/run_log/<run-id>.json`) is output in bulk by ai-conductor

Users can view progress via `cat <workspace>/agent.log` or `hestia tail <peer>`.

---

## Article 4: Non-Stop Execution (Phase 50 / autonomous_work feedback inheritance)

Agents execute continuously during the exec_job cycle without requesting user permission. The only conditions for stopping are:

1. Receiving a work request that exceeds the persona's scope of responsibilities
2. Missing required inputs (params.* / existing files) (return `input_required`)
3. Missing required physical tools (verilator / Vivado / yosys etc.) (return `tool_unavailable`)
4. Receiving an explicit stop instruction from a superior

In all other situations (warning detection / iteration in progress / partial artifact generation), agents do not halt, and instead return each status honestly and delegate the decision to the superior.

---

## Article 5: Transparent Failure Reporting (Phase 50 inheritance)

When a handler returns any of the following, the agent reports to the superior with a **three-point set of reason / next action candidates / relevant log excerpt**:

| status | meaning | required reporting items |
|--------|---------|--------------------------|
| `input_required` | Required input missing | Missing input name / how to provide it |
| `tool_unavailable` | Physical tool unavailable | Tool name / installation method / alternatives |
| `skipped` | Reusing existing artifact | Existing file path / how to regenerate |
| `*_failed` (lint_failed / build_failed etc.) | Physical tool launch failure | Failed step / `error_log_excerpt` (50-200 lines) |
| `sim_warnings` | Warnings detected (Phase 50) | Warning count / breakdown / suppression method |

Reporting only "execution stopped" is prohibited.

---

## Article 6: Flag Position Convention (Phase 81 inheritance)

When agents invoke CLI commands via shell, `CommonOpts` flags such as `--output json` are placed **before** the subcommand:

```bash
# Correct
hestia-rtl-cli --output json lint --project ./
# Wrong (clap flatten rejects this)
hestia-rtl-cli lint --project ./ --output json
```

All `CommonOpts` flags have `global = true` set (Phase 17), so technically they work in either position, but the persona convention standardizes placing them before the subcommand.

---

## Article 7: Shell-Based In-Process Execution (Phase 16 / Policy X)

The ai-conductor LLM, as the orchestrator, invokes each `hestia-{domain}-cli` via shell for in-process Handler calls (not agent-cli IPC, but Rust function calls). This design:

- Sequentially invokes domain CLIs via the shell tool (LLM `tool_call: shell`)
- Each CLI directly invokes its domain handler within its own process, returning structured JSON to stdout
- Results are aggregated to `<root>/.hestia/run_log/<run-id>.json`

This is the core design of Phase 16 "Policy X Adoption", and agents must not change this path.

---

## Article 8: Prohibition of Direct `.aiprj/` Reference (Phase 81 new)

Hestia agents do not directly reference the `.aiprj/` directory. Execution rules reference `.hestia/rules/exec_job.md` (this file) instead.