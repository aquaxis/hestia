# Registry Management

**Domain**: container — Registry
**Source**: Design Specification §12.6

## Overview

Container image registry management. Distinguishes between local development registries and remote registries (GHCR / Quay / Private Harbor), and improves operational efficiency through tag conventions and retention policies.

## Supported Registries

| Registry | Use Case | Authentication |
|-----------|-----|------|
| `localhost:5000` | Local development | None |
| `ghcr.io/hestia/*` | Official publishing (OSS images only) | GitHub OIDC |
| `quay.io/hestia/*` | Mirror (OSS images only) | Robot account |
| Private Harbor | Images containing commercial tools | mTLS + Robot account |

## Tag Convention

```
<registry>/<org>/<conductor>/<tool>:<version>
<registry>/<org>/<conductor>/<tool>:<YYYYMMDD>   # Date tag
<registry>/<org>/<conductor>/<tool>:cache        # Build cache only
```

Examples:
- `ghcr.io/hestia/fpga/vivado:2025.2`
- `ghcr.io/hestia/fpga/oss:20260423`
- `ghcr.io/hestia/fpga/oss:cache`

## push / pull Flow

```bash
# Authentication
podman login ghcr.io -u ${GITHUB_USER} --password-stdin < token

# Push
podman push ghcr.io/hestia/fpga/oss:latest
podman push ghcr.io/hestia/fpga/oss:20260423

# Pull-through cache (via Harbor)
podman pull harbor.internal/docker-proxy/docker.io/library/ubuntu:24.04
```

## Retention Policy

| Tag Type | Retention Period | Deletion Method |
|---------|---------|---------|
| `latest` / `<version>` | Permanent | Manual |
| Date tags (`YYYYMMDD`) | Most recent 30 days only | `skopeo delete` + cron |
| `cache` | Latest only | Automatic |

```bash
# Retention script example
skopeo list-tags docker://ghcr.io/hestia/fpga/oss | \
  jq -r '.Tags[] | select(test("^[0-9]{8}$"))' | \
  while read tag; do
    tag_date=$(date -d "${tag:0:4}-${tag:4:2}-${tag:6:2}" +%s 2>/dev/null) || continue
    cutoff=$(date -d "30 days ago" +%s)
    if [ "$tag_date" -lt "$cutoff" ]; then
      skopeo delete docker://ghcr.io/hestia/fpga/oss:$tag
    fi
  done
```

## Rate Limit Mitigation

- Use a private mirror (Harbor) to avoid Docker Hub pull limits (100 pulls/6h anonymous)
- Use GHCR as the pull source on GitHub Actions to avoid limits
- Save bandwidth with `podman pull --quiet`, recover from transient failures with `--retry 5`

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [security_hardening.md](security_hardening.md) — Signing & SBOM