# Health Check Orchestration

**Domain**: common — Operational Monitoring
**Source**: Design Specification §3.3.2

## Overview

ai-conductor's `health-checker` periodically polls all conductors for liveness and health, centrally managing ConductorStatus. On anomaly detection, it attempts automatic restart and escalates to a human (frontend notification) if recovery is impossible.

## Polling Specification

| Parameter | Default | Configuration Location |
|----------|-------|---------|
| Polling interval | 30 seconds | `[health] interval_secs` |
| Response timeout | 3 seconds | `[health] timeout_secs` |
| Auto-restart limit | 3 times | `[health] max_retries` |

## ConductorStatus

| State | Meaning | Transition Condition |
|------|------|---------|
| `Online` | Healthy and running | `"online"` response within 3 seconds |
| `Offline` | Stopped | Timeout (3 seconds)|
| `Degraded` | Degraded (partial feature restrictions) | `"degraded"` response |
| `Upgrading` | Upgrading | `"upgrading"` response |

## Health Check Flow

```
ai-conductor::health-checker (tokio interval task)
    |
    | 30-second interval
    |
    v
For each peer in [rtl, fpga, asic, pcb, hal, apps, debug, rag]:
    |
    | agent-cli send <peer> '{"method":"system.health.v1","id":"hc_<ts>"}'
    |
    v
Response pattern -> ConductorStatus update:
    - "online" response within 3 sec -> Online
    - Timeout (3 sec)                -> Offline
    - "degraded" response            -> Degraded
    - "upgrading" response            -> Upgrading
    |
    v
Action on state change:
    - Online -> Offline / Degraded -> Observability log + auto restart (max 3)
    - 3 consecutive failures        -> Frontend notification
    - Upgrading -> Online           -> Success notification to upgrade-manager
    - Any                          -> Persist state history to sled
```

## Response Specification (system.health.v1)

```json
// Request
{
  "method": "system.health.v1",
  "id": "hc_2026-05-01T12:00:00Z_abc123",
  "trace_id": "health_loop_20260501T120000"
}

// Response
{
  "result": {
    "status": "online",
    "uptime_secs": 12345,
    "tools_ready": ["vivado", "yosys"],
    "load": { "cpu_pct": 12, "mem_mb": 512 },
    "active_jobs": 3,
    "last_error": null
  },
  "id": "hc_2026-05-01T12:00:00Z_abc123",
  "trace_id": "health_loop_20260501T120000"
}
```

## Configuration Example

```toml
[health]
cmd = "vivado -version || true"   # Quick verification command in local execution mode
interval_secs = 30                # Polling interval
timeout_secs = 3                  # Response timeout
max_retries = 3                   # Consecutive retry count
escalate_on_fail = true           # Notify frontend on consecutive failures
restart_on_fail = true            # Automatic restart attempt
```

## Related Documents

- [conductor_startup.md](conductor_startup.md) — Daemon startup order
- [update_mechanism.md](update_mechanism.md) — UpgradeManager
- [observability.md](observability.md) — Monitoring and metrics