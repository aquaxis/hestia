# CI/CD API

**Domain**: common — CI/CD
**Source**: Design Specification §13.5

## Overview

A shared service that declaratively defines CI/CD pipelines and runs them across multiple backends (GitHub Actions / GitLab CI / Local). Provided as agent-cli peer `cicd`.

## Supported Backends

| Backend | Identifier | Purpose |
|---------|-------|------|
| `GithubActions` | `github_actions` | CI/CD when hosted on GitHub |
| `GitlabCi` | `gitlab_ci` | CI/CD when hosted on GitLab |
| `LocalPipeline` | `local` | Local execution (offline development/verification)|

## Key Types

### PipelineDefinition

Declarative definition of a pipeline. Structures stages, jobs, and conditions.

### PipelineStage

Stage definition. Contains multiple jobs and controls parallel or sequential execution.

### PipelineJob

Job definition. Includes execution commands, environment, artifacts, and retry settings.

### StageCondition

| Value | Meaning |
|----|------|
| `Always` | Always execute |
| `OnSuccess` | Only when previous stage succeeds |
| `OnFailure` | Only when previous stage fails |
| `Custom` | Custom condition expression |

## Control Parameters

Pipelines are controlled via JSON with the following:

| Parameter | Description |
|----------|------|
| Artifact retention | Artifact retention period |
| Retry policy | Retry count and interval on job failure |
| Timeout secs | Per-job timeout |
| Cache key | Build cache key |

## CI/CD Integration Example (GitHub Actions)

```yaml
name: Container Build & Publish
on:
  push:
    paths:
      - '.hestia/containers/**'
  schedule:
    - cron: '0 2 * * 1'   # Every Monday 02:00 UTC

jobs:
  build:
    strategy:
      matrix:
        image: [fpga/oss, asic/openlane, pcb/kicad, debug/tools]
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: podman build --file .hestia/containers/${{ matrix.image }}/Containerfile ...
      - name: SBOM
        run: syft ... -o spdx-json > sbom.spdx.json
      - name: Vulnerability scan
        run: grype ... --fail-on high --only-fixed
      - name: Sign (cosign keyless)
        run: cosign sign ...
```

## Observability Integration

| Metric | Type | Meaning |
|----------|----|------|
| `hestia_container_build_total{image,status}` | Counter | Build count |
| `hestia_container_build_duration_seconds{image,stage}` | Histogram | Duration per stage |

## Related Documents

- [observability.md](observability.md) — Monitoring and metrics
- [container_manager](../container/container_manager.md) — Container management
- [agent_cli_messaging.md](agent_cli_messaging.md) — Messaging specification