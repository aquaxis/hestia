# Daemon Startup Order

**Domain**: common — Startup and Operations
**Source**: Design Specification §15.5

## Overview

HESTIA's 9 conductors start with ai-conductor as the highest priority. After readiness is confirmed, the remaining 8 conductors start in parallel. This two-phase startup order guarantees that the meta-orchestrator is running before the remaining conductors initialize.

## Startup Groups

### Group 0 (Serial, Highest Priority)

Start ai-conductor alone and wait until `system.health.v1` returns `status="online"`.

```bash
# Group 0
agent-cli run --persona-file ./.hestia/personas/ai.md --name ai &
# Wait until system.health.v1 returns status="online"
agent-cli send ai '{"method":"system.health.v1"}'
```

### Group 1 (8 Parallel)

After ai-conductor readiness is confirmed, start the following 8 conductors in parallel.

```bash
# Group 1 (8 parallel)
agent-cli run --persona-file ./.hestia/personas/rtl.md --name rtl &
agent-cli run --persona-file ./.hestia/personas/fpga.md --name fpga &
agent-cli run --persona-file ./.hestia/personas/asic.md --name asic &
agent-cli run --persona-file ./.hestia/personas/pcb.md --name pcb &
agent-cli run --persona-file ./.hestia/personas/hal.md --name hal &
agent-cli run --persona-file ./.hestia/personas/apps.md --name apps &
agent-cli run --persona-file ./.hestia/personas/debug.md --name debug &
agent-cli run --persona-file ./.hestia/personas/rag.md --name rag &
```

## systemd User Units (Recommended)

```bash
# /etc/systemd/user/hestia-{ai,rtl,fpga,asic,pcb,hal,apps,debug,rag}.service
systemctl --user start hestia-ai hestia-rtl hestia-fpga hestia-asic \
  hestia-pcb hestia-hal hestia-apps hestia-debug hestia-rag
```

Using systemd units automates the following:

- Declaration of startup dependencies (ai-conductor -> remaining 8 conductors)
- Automatic restart on failure
- Log management (journald integration)

Each systemd unit's `ExecStart` uses `agent-cli run --persona-file ./.hestia/personas/<conductor>.md --name <conductor>`.

## Readiness Check Details

Whether ai-conductor has finished starting is determined by:

```bash
agent-cli send ai '{"method":"system.health.v1"}'
```

Response example:

```json
{
  "result": {
    "status": "online",
    "uptime_secs": 5,
    "tools_ready": ["task-router", "health-checker", "skill-system"],
    "active_jobs": 0
  }
}
```

If an `"online"` response is not received within 3 seconds, continue waiting.

## Related Documents

- [health_check_orchestration.md](health_check_orchestration.md) — Health check details
- [installation.md](installation.md) — Build procedure
- [sub_agent_lifecycle.md](sub_agent_lifecycle.md) — Sub-agent startup and shutdown management