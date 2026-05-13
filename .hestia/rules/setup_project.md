---
name: Hestia Agent Setup Guidelines
description: Rules that hestia agents (each conductor / sub-agent) follow during workspace setup. Adapted from `.aiprj/rules/setup_project.md` for the agent context (Phase 81 P-3).
---

# Hestia Agent Setup Guidelines (Phase 81)

This file defines the rules that hestia agents (ai-conductor / 9 domain conductors / 50+ sub-agent personas) reference during the **setup phase at startup**. It is an independent entity separate from `.aiprj/rules/setup_project.md` which is intended for project management AI, and is designed so that the hestia runtime functions even in environments without `.aiprj/`.

---

## Article 1: Input Acquisition

Agents acquire inputs in the following priority order at startup:

1. Prompt received from the superior conductor via `agent-cli send` (all superior instructions come through the peer prompt)
2. `<workspace>/{requirements,design,tasks}.md` (if already generated in a setup_ai cycle, as input for consistency verification)
3. The persona's own scope of responsibilities (domain responsibilities indicated by the `name` field)

(The previous fs_read-based acquisition path has been abolished by Phase 89 terminology unification; inputs are now organized into the above 3 categories.)

If input is empty or missing, the agent transitions to an idle state and waits for instructions from the superior. It must not exit with an error.

---

## Article 2: Artifact Creation Scope (Phase 81 - 3-Document Compliance Mandatory)

Agents create **3 documents + artifacts within the workspace and under the project root** via fs_write, within the scope of their persona responsibilities. Phase 81 makes 3-document compliance mandatory at all levels.

| Persona level | Primary creation targets (3 documents + artifact two-tier) |
|------------|------------------------------|
| ai-conductor | (1) `<workspace>/{requirements,design,tasks}.md` 3 documents / (2) `<root>/.hestia/run_log/<run-id>.json` aggregate / `<workspace>/agent.log` |
| domain conductor (rtl/fpga/asic/pcb/hal/apps/debug/rag) | (1) `<workspace>/{requirements,design,tasks}.md` 3 documents / (2) `<root>/<domain>/...` artifacts (e.g., rtl/<top>.sv) / `<workspace>/agent.log`. Task creation and management is handled directly by this conductor (domain planner abolished in Phase 91) |
| sub-agent (designer/coder/tester/...) | (1) `<workspace>/{requirements,design,tasks}.md` 3 documents / (2) Design / implementation / test artifacts for the assigned module |

Writing to the `.aiprj/` directory is prohibited (project management AI's exclusive domain). Skipping the 3 documents is prohibited (Phase 81 mandatory compliance).

---

**Phase 92 Clarification**: Each persona level's 3 documents (`requirements.md` / `design.md` / `tasks.md`) are **exclusive to that agent's workspace directory** (`.hestia/workspaces/<peer>/`) and are independent from other agents' 3 documents. Shared placement (under the project root like `<root>/requirements.md` or cross-referencing between agents) is prohibited. For example, `ai/requirements.md` and `rtl-designer/requirements.md` are managed as separate files with separate content.

---

## Article 3: Template Embedding Prohibition (Phase 81 inheritance)

Domain-specific templates (HDL / constraints / TCL / register maps, etc.) must not be embedded in handler source code under `.hestia/tools/`. The LLM (ai-conductor / designer etc.) dynamically generates them via `fs_write`, and handlers are responsible only for invoking physical tools (verilator / Vivado / yosys etc.).

For details, refer to the "Absolute Rules" section of `.hestia/personas/ai.md` and the Phase 42/47 persona rules.

---

## Article 4: Self-Execution Loop (Phase 57b/68/71 inheritance)

Agents judge and execute the following 4 cycles on their own:

| Cycle | Trigger | Reference rules |
|---------|------|---------|
| setup_ai | Immediately after peer startup | This file (`.hestia/rules/setup_project.md`) |
| update_ai | Superior prompt received + content diff detected in `<workspace>/requirements.md` (Phase 89 terminology unification)| `.hestia/rules/update_project.md` |
| exec_job | Execution request prompt received from superior | `.hestia/rules/exec_job.md` |
| close_ai | Session termination notification | `.hestia/rules/close_ai.md` (planned for Phase 82+) |

The judgment and branching logic for each cycle is described in the "Self-execution rules for `.hestia/rules/` at startup" section within each persona.

---

## Article 5: Progress Visibility (Phase 49 mirror inheritance)

Agent activity (thinking / tool_call / tool_result / peer_prompt / assistant) is written to agent-cli's structured JSONL. `hestia start` spawns `hestia mirror <peer>` as a detached helper, allowing users to observe activity in real time via `cat .hestia/workspaces/<peer>/agent.log`.

Agents do not need to perform explicit log output -- agent-cli's structured events reach the workspace agent.log via the mirror path.

---

## Article 6: Transparent Failure Reporting (Phase 50 inheritance)

When a handler returns `input_required` / `tool_unavailable` / `skipped` / `*_failed`, the agent reports to the superior with **reasons at a granularity that enables the next decision**. These are included inline in the aggregate JSON's `halted_reason` field and each step's `error_log_excerpt`.

Reporting only "execution stopped" is prohibited. Information granularity sufficient for the user to determine the next action is required.

---

## Article 7: Prohibition of Direct `.aiprj/` Reference (Phase 81 new)

Hestia agents (personas / handlers / Rust source code) must not directly reference the `.aiprj/` directory. Only `.hestia/rules/` (including this file `.hestia/rules/setup_project.md`) is within the reference scope.

Exception: The project management AI (the Claude Code session running at the top level of this repository) continues to use `.aiprj/`, but this is outside the scope of these rules since it is not a hestia agent.