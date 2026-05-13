# UpgradeManager (Sustainable Upgrade)

**Domain**: common — Version Management
**Source**: Design Specification §3.4, §1.3.7

## Overview

A sustainable upgrade mechanism where AI agents automatically generate, validate, and apply patches in response to vendor tool version upgrades, minimizing human intervention. Provides phased rollout and automatic rollback based on semantic versioning.

## 4-Agent Configuration

Each agent operates as an independent agent-cli process:

| Agent | Peer Name | Role |
|------------|---------|------|
| WatcherAgent | `ai-watcher` | Monitors vendor sites every 6 hours and detects new versions |
| ProbeAgent | `ai-probe` | Runs test builds with new versions on standard test project sets, detects incompatibilities |
| PatcherAgent | `ai-patcher` | Leverages agent-cli's Tool Use functionality to automatically generate patches |
| ValidatorAgent | `ai-validator` | Validates patches in a sandbox environment and calculates confidence scores |

### PatcherAgent Tool Use (6 types)

| Tool | Function |
|-------|------|
| `read_adapter_manifest()` | Retrieve adapter.toml contents |
| `read_error_log()` | Retrieve build error details |
| `search_breaking_changes()` | Search for known breaking changes |
| `read_vendor_changelog()` | Retrieve release notes |
| `propose_patch()` | Submit a patch proposal |
| `trigger_validation()` | Execute validation |

## HumanReviewGate

Determines automatic application or manual review based on confidence score:

```
PatcherAgent -> ValidatorAgent -> Confidence score calculation
                                    |
                                    +-- High confidence -> Auto-apply
                                    +-- Low confidence -> HumanReviewGate (human review)
```

## Compatibility Determination

| Version Change | Compatibility | Required Strategy |
|--------------|--------|---------|
| `1.0.0` -> `1.1.0` | Compatible | Production OK |
| `1.0.0` -> `1.0.1` | Compatible | Production OK |
| `1.0.0` -> `2.0.0` | Incompatible | Canary or Staging required |

## Phased Rollout

| Strategy | Description | Use Case |
|------|------|---------|
| `Canary` | Deploy to a small number of environments first | Major version changes |
| `Staging` | Deploy to production after staging verification | Minor version updates |
| `Production` | Deploy directly to production | Patch releases |

## Automatic Rollback

```rust
pub struct RollbackConfig {
    pub auto_rollback: bool,     // Enable automatic rollback
    pub timeout_secs: u64,       // Timeout (default: 300 seconds)
    pub max_retries: u32,        // Maximum retry count (default: 3)
}
```

Rollback triggers on:
- Consecutive health check failures (§3.3.2)
- Test suite regressions
- Observability metrics threshold exceeded

## Overall Flow

```
Detection (WatcherAgent) -> Testing (ProbeAgent) -> Patch Generation (PatcherAgent)
  -> Validation (ValidatorAgent) -> Determination (HumanReviewGate)
  -> Phased Application (Canary -> Staging -> Production)
  -> Rollback on Anomaly
```

## Crate Structure

```
upgrade-manager/
└── src/
    ├── lib.rs
    ├── version_policy.rs   # Semantic versioning
    ├── rollout.rs          # Phased rollout
    └── rollback.rs         # Automatic rollback
```

## Related Documents

- [health_check_orchestration.md](health_check_orchestration.md) — Health checks
- [backend_switching.md](backend_switching.md) — LLM backend switching
- [sub_agent_lifecycle.md](sub_agent_lifecycle.md) — Sub-agent management