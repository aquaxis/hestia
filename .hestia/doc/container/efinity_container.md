# Efinity Container Details

**Domain**: container — FPGA Tool Container
**Source**: Design Specification §12.2

## Overview

Container image `fpga/efinity:2025.2` for running Efinix Efinity 2025.2. Ubuntu 24.04 based, using the bundled Efinity Python API for synthesis and place-and-route.

## Image Configuration

| Item | Value |
|------|-----|
| Image name | `fpga/efinity:2025.2` |
| Base image | `docker.io/library/ubuntu:24.04` |
| Primary tool | Efinix Efinity 2025.2 |
| License | Efinity bundled Python (proprietary license) |
| User | `hestia` (UID 1000) |

## Notable Points

- Operates via Python API, unlike other commercial tools
- Does not use FlexLM, unlike Vivado / Quartus
- Ubuntu based (not UBI 9)

## Execution Example

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -v $(pwd):/workspace:Z \
  fpga/efinity:2025.2 \
  python3 -m efinity.flow run --project project.xml
```

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [vivado_container.md](vivado_container.md) — Vivado container
- [oss_container.md](oss_container.md) — OSS FPGA container