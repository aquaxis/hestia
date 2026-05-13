# Test Specification

**Domain**: common — Test Strategy
**Source**: Design Specification §15.4, §19.2, §20.7

## Overview

HESTIA's test strategy consists of three layers (unit / integration / E2E). This document defines the purpose, scope, and execution method of each layer.

## Test Layer Structure

### Unit Tests

| Item | Description |
|------|------|
| Purpose | Verify the correctness of individual functions and methods |
| Scope | Logic, parsers, conversions, and data models within a crate |
| Execution method | `cargo test -p <crate>` |
| CI trigger | All PRs / pushes |

Key unit test examples:
- `project-model::config` — 8 `[agent_cli]` configuration tests
- `constraint-bridge` — Parse/generate tests for each format
- `ip-manager` — semver resolution and DAG construction tests
- `error_registry` — Error code range validation

### Integration Tests

| Item | Description |
|------|------|
| Purpose | Verify cross-crate interactions and IPC communication |
| Scope | Inter-conductor communication, DB read/write, container builds |
| Execution method | `cargo test -p integration-tests` |
| CI trigger | Merge to main |

Key integration test examples:
- `integration-tests::agent_cli_config` — 3 tests (parse / Default match / build_env)
- Inter-conductor `agent-cli send` communication tests
- SQLite / sled read/write consistency tests
- container-manager build pipeline tests

### E2E Tests

| Item | Description |
|------|------|
| Purpose | Verify end-to-end user scenarios |
| Scope | CLI -> conductor -> tool execution -> result verification |
| Execution method | `cargo test -p e2e-tests` / manual |
| CI trigger | Pre-release / scheduled |

Key E2E test examples:
- `hestia init` -> `hestia start fpga` -> `hestia fpga build` -> result verification
- Actual Ollama startup -> agent-cli spawn -> ping
- `hestia rag ingest` -> `hestia rag search` -> result verification

## TDD Practices (§19.2)

For each module, a two-phase "testbench phase -> implementation phase" is mandatory:

1. Generate testbench first (by AI or manually)
2. Run tests -> FAIL (DUT not implemented)
3. Implement design
4. Run tests -> PASS/FAIL
5. Do not proceed to the next step with coverage < 95%

## Test Coverage Targets

| Layer | Coverage Target |
|----|-------------|
| Unit tests | 80% or higher |
| Integration tests | 100% of critical paths |
| E2E tests | 100% of user scenarios |

## CI/CD Integration

- All PRs automatically run unit tests + integration tests
- Release branches automatically run E2E tests
- Test failures are recorded in `action-log`

## Related Documents

- [installation.md](installation.md) — Build procedure
- [cargo_workspace.md](cargo_workspace.md) — Workspace configuration
- [cicd_api.md](cicd_api.md) — CI/CD API