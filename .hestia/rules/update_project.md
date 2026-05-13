---
name: Hestia Agent Update Guidelines
description: Update rules that hestia agents follow during the update_ai cycle. Adapted from `.aiprj/rules/update_project.md` for the agent context (Phase 81 P-3).
---

# Hestia Agent Update Guidelines (Phase 81)

This file defines the rules that hestia agents reference during the **update_ai cycle** (when receiving a "reflect current state" request from a superior, or when detecting a content diff between the superior's instructions and the current `<workspace>/requirements.md`, Phase 89 terminology unification). It is an independent entity in the hestia context, separate from `.aiprj/rules/update_project.md` which is intended for project management AI.

---

## Article 1: Update Trigger Determination

Agents transition to the update_ai cycle when any of the following is detected:

1. Receiving a prompt equivalent to "update / refresh / reload" from a superior conductor via `agent-cli send`
2. Detecting a content diff between the superior's instructions and the current `<workspace>/requirements.md`
3. Detecting inconsistency with existing artifacts within the scope of responsibilities (see Article 3)

(The previous mtime-monitoring-based trigger has been abolished by Phase 89 terminology unification; triggers are now organized into the above 3 categories.)

Judgment is also performed periodically within the exec_job cycle (on each prompt receipt).

---

## Article 2: Update Targets (Phase 91 - 3-Document Compliance)

Agents update **3 documents + workspace artifacts** within the scope of their persona responsibilities. Phase 81 makes 3-document compliance mandatory at all levels:

| Persona level | Primary update targets (3 documents + artifacts) |
|------------|---------------------------|
| ai-conductor | (1) `<workspace>/{requirements,design,tasks}.md` 3 documents / (2) `<root>/.hestia/run_log/<run-id>.json` aggregate / `<workspace>/agent.log` |
| domain conductor | (1) 3 documents / (2) Artifacts of the responsible domain (`<root>/<domain>/...`) / Artifacts generated via handler. Task management is handled directly by this conductor following the Phase 91 domain planner abolition |
| sub-agent | (1) 3 documents / (2) Design / implementation / test artifacts for the assigned module |

Project management AI documents such as `.aiprj/AI_PRJ_*.md` are **not updated** (outside scope). Skipping the 3 documents is prohibited (Phase 91 mandatory compliance).

---

**Phase 92 Clarification**: The 3 documents updated by each persona level are **per-agent exclusive** (not shared) and exist independently under `.hestia/workspaces/<peer>/`. Agents must not directly update other agents' 3 documents -- each agent updates only the 3 documents within its own workspace.

---

## Article 3: Consistency Maintenance and Halt Reporting

When inconsistency with existing artifacts is detected during an update (refer to Article 2 of `exec_job.md`), agents respond in the following priority order:

1. **Within scope of responsibilities and clearly inconsistent**: Auto-fix and explicitly document the fix in agent.log
2. **Within scope of responsibilities but wide impact**: Report to superior with halt reason and request a decision on the fix approach
3. **Outside scope of responsibilities**: Do not attempt to fix; report to superior with halt reason

The criterion for "within scope and clearly inconsistent" is limited to the scope explicitly stated in the persona's `name` / `description` fields and the responsibility definition in `.hestia/personas/<self>.md`.

---

## Article 4: Progress Recording (Phase 49 mirror inheritance)

Update progress is automatically recorded via the agent-cli structured JSONL -> mirror -> workspace agent.log path. Agents do not perform explicit progress fs_write (same as `exec_job.md` Article 3).

---

## Article 5: Transparent Failure Reporting (Phase 50 inheritance)

If an update fails, agents report to the superior using the same three-point set (reason / next action candidates / relevant log excerpt) as `exec_job.md` Article 5. Reporting only "could not update" is prohibited.

---

## Article 6: Version Control (Autonomous)

Agents **do not** perform version control (git commit / tag etc.) on workspace artifacts. Git operations are the responsibility domain of the project management AI and the user. Agents are responsible only for maintaining the latest state under the workspace, and delegate history management to the superior.

---

## Article 7: Prohibition of Direct `.aiprj/` Reference (Phase 81 new)

Hestia agents do not directly reference the `.aiprj/` directory. Update rules reference `.hestia/rules/update_project.md` (this file) instead.

---

## Article 8: Consistency with Self-Execution Loop (Phase 57b/68/71 inheritance)

The update_ai cycle is one of the 4 cycles: setup_ai -> exec_job -> update_ai -> close_ai. After update_ai completes, the agent typically transitions to exec_job or idle. The transition logic between each cycle is described in the "Self-execution rules for `.hestia/rules/` at startup" section within each persona.