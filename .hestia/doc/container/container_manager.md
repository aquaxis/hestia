# container-manager Full Specification

**Domain**: container — Container Management
**Source**: Design Specification §12

## Overview

container-manager is the container lifecycle management system used when container execution is selected. It provides Containerfile auto-generation, build flow, provisioning, signing, registry, CI/CD integration, and operational rules. Not applicable when local execution is selected.

## Module Structure

| Module | Role |
|-----------|------|
| builder | Containerfile auto-generation and build (based on tool definition files) |
| registry | Container image registry management (local/remote) |
| updater | Image differential update, version management, and automatic rebuild |
| provisioner | Provisioning based on tool definition files (package installation) |
| tool_updater | Tool version detection, compatibility checking, and staged updates |

## Containerfile Auto-Generation (FR-CTN-11)

Takes `container.toml` as input and auto-generates a Containerfile using the minijinja template engine.

```
container.toml → builder::parse → Container Spec (Rust struct)
    → Containerfile template (minijinja)
    → Containerfile (text)
    → podman build (§12.3)
```

### 8 Image Configuration

| Image | Base | Primary Tools | License Handling |
|---------|-------|----------|--------------|
| `fpga/vivado:2025.2` | `ubi9/ubi:9.5` | AMD Vivado 2025.2 | FlexLM |
| `fpga/quartus:25.1` | `ubi9/ubi:9.5` | Intel Quartus Prime Pro 25.1 | FlexLM |
| `fpga/efinity:2025.2` | `ubuntu:24.04` | Efinix Efinity 2025.2 | Efinity Python |
| `fpga/radiant:2024.2` | `ubi9/ubi:9.5` | Lattice Radiant 2024.2 | FlexLM |
| `fpga/oss:latest` | `ubuntu:24.04` | Yosys + nextpnr + icestorm + Verilator | Not required (OSS) |
| `asic/openlane:latest` | `ubuntu:24.04` | OpenLane 2 + Yosys + OpenROAD + Magic | Not required (OSS) |
| `pcb/kicad:latest` | `ubuntu:24.04` | KiCad + SKiDL + Freerouting | Not required (OSS) |
| `debug/tools:latest` | `ubuntu:24.04` | OpenOCD + sigrok + PulseView + pyOCD | Not required (OSS) |

## Build Flow (FR-CTN-12)

### Multi-Stage Strategy

- **Stage 1 (install / build)**: Large layer containing installers and build dependencies (not retained in the final image)
- **Stage 2 (runtime)**: Minimal runtime only (size is 30-50% of Stage 1)

### Layer Cache Strategy

| Layer Type | Frequency | Placement |
|----------|-----|------|
| Base OS | Low | Topmost |
| Dependency libraries | Medium | Second |
| EDA tool binaries | Low | Third (separate stage) |
| Configuration / scripts | High | Bottom |

### Build Duration Monitoring

- OSS images < 10 minutes, commercial images < 30 minutes
- Monitored via `hestia_container_build_duration_seconds{image,stage}` metric

## Provisioning (FR-CTN-13)

| install_method | Command |
|---------------|---------|
| `apt` | `apt-get install -y ${package}` |
| `dnf` | `dnf install -y ${package}` |
| `tarball` | `wget -O - ${url} \| tar -xz -C ${prefix}` |
| `install_script` | `bash -c "${install_script}"` |
| `pip` | `pip install --no-cache-dir ${package}` |
| `cargo` | `cargo install ${package}` |

## Artifact Signing & SBOM (FR-CTN-14)

```
podman build → Image generation → SBOM generation via syft → Vulnerability scan via grype
    → Evaluation gate → cosign sign (keyless) → podman push
```

### Evaluation Gate Thresholds

| Severity | Threshold | Behavior |
|-------|-----|------|
| Critical | 0 | Build fails (push blocked) |
| High (fixable) | 0 | Build fails |
| High (unfixable) | Log | Warning + exception approval |
| Medium or below | Log | Normal push allowed |

## CI/CD Integration

Weekly build (every Monday 02:00 UTC), patch versions are automatic, minor versions require Review Agent approval, major versions require manual trigger (Canary strategy).

## Operational Rules

1. Weekly build incorporates base image + dependency updates and rebuilds
2. Patch version: automatic build + automatic push
3. Minor version: automatic build, then 1 approval required via Review Agent
4. Major version: manual trigger, staged deployment via Canary strategy
5. On failure: log to `action-log`, notify PatcherAgent
6. Image size limits: OSS 5GB, commercial 20GB
7. BuildKit secret management: `.hestia/secure/` has 0700 permissions, excluded from Git
8. CVE notification: escalate to security team immediately on Critical occurrence

## Implementation Crate Structure

```
container-manager/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── builder/         # Containerfile generation + podman build
    │   ├── mod.rs
    │   ├── templates/*.j2
    │   └── parser.rs    # container.toml → ContainerSpec
    ├── registry/        # push/pull/retention (skopeo wrapper)
    ├── updater/          # Differential update / automatic rebuild
    ├── provisioner/      # [tools.*] → install command translation
    ├── tool_updater/     # Version detection / semver matching
    ├── signer/          # cosign wrapper
    ├── sbom/            # syft / grype wrapper
    └── observability.rs # Metrics dispatch
```

## Related Documentation

- [vivado_container.md](vivado_container.md) — Vivado container details
- [quartus_container.md](quartus_container.md) — Quartus container details
- [security_hardening.md](security_hardening.md) — Security hardening
- [registry_config.md](registry_config.md) — Registry management