# Container Security Hardening

**Domain**: container — Security
**Source**: Design Specification §12.5, §11.1

## Overview

Multi-layered security hardening for the container execution environment. In addition to Podman rootless baseline security, artifact signing (cosign), SBOM generation (syft), vulnerability scanning (grype), and protective measures based on the security design principles in §11.1 are applied.

## Podman Security Principles (§11.1)

| Principle | Implementation |
|------|------|
| Daemonless | No system service required (does not depend on Docker's dockerd) |
| UID matching | Avoid file permission issues with `--userns=keep-id` |
| SELinux support | `--security-opt=label=type:container_runtime_t` |
| Privilege escalation prevention | `--security-opt=no-new-privileges` |
| License protection | Read-only mount of vendor license files |
| Network isolation | Build containers use `--network=none` |
| Intellectual property protection | Prevent HDL source and bitstream leakage outside containers |

## Security by Container Operation Pattern

| Pattern | Security Settings |
|---------|----------------|
| Batch build | `--rm`, `--network=none`, `--security-opt=no-new-privileges` |
| GUI launch | X11/Wayland forwarding, `--security-opt=label=type:container_runtime_t` |
| systemd service | Podman Quad definition (.container file) |
| Device access | `--device /dev/bus/usb` (JTAG programming only) |

## Artifact Signing (cosign, keyless OIDC)

```bash
# Keyless signing (on GitHub Actions)
COSIGN_EXPERIMENTAL=1 cosign sign ghcr.io/hestia/fpga/oss:latest \
  --attachment sbom.spdx.json

# SBOM attestation
cosign attach sbom --sbom .hestia/containers/fpga/oss/sbom.spdx.json \
  ghcr.io/hestia/fpga/oss:latest
```

### Unsigned Image Rejection Policy

- `cosign verify` is executed before `podman-runtime::run_build()`
- Container launch is refused on signature verification failure (`SignatureVerificationError`)
- Development environments can bypass with `HESTIA_ALLOW_UNSIGNED=1` (production always verifies)

## SBOM Generation (syft)

```bash
# SPDX format
syft ghcr.io/hestia/fpga/oss:latest -o spdx-json > sbom.spdx.json

# CycloneDX format
syft ghcr.io/hestia/fpga/oss:latest -o cyclonedx-json > sbom.cdx.json
```

## Vulnerability Scanning (grype)

```bash
grype ghcr.io/hestia/fpga/oss:latest \
  --fail-on high \
  --only-fixed \
  -o json > vuln.json
```

### Evaluation Gate Thresholds

| Severity | Threshold | Behavior |
|-------|-----|------|
| Critical | 0 | Build fails (push blocked) |
| High (fixable) | 0 | Build fails |
| High (unfixable) | Log | Warning + exception approval |
| Medium or below | Log | Normal push allowed |

## Vendor License Protection

- Installer and license files are not baked into the image (mounted only via `--secret`)
- Runtime licenses (FlexLM, etc.) are injected via `podman run -e LM_LICENSE_FILE=...`
- `.hestia/secure/` has mode 0700, excluded from Git via `.gitignore`

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [registry_config.md](registry_config.md) — Registry management
- [vivado_container.md](vivado_container.md) — Vivado container