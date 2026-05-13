# Radiant Container Details

**Domain**: container — FPGA Tool Container
**Source**: Design Specification §12.2

## Overview

Container image `fpga/radiant:2024.2` for running Lattice Radiant 2024.2. Used for synthesis and place-and-route of Lattice LIFCL / LFD2NX / iCE40 family FPGAs.

## Image Configuration

| Item | Value |
|------|-----|
| Image name | `fpga/radiant:2024.2` |
| Base image | `registry.access.redhat.com/ubi9/ubi:9.5` |
| Primary tool | Lattice Radiant 2024.2 |
| License | FlexLM |
| User | `hestia` (UID 1000) |

## Notable Points

- Follows the Vivado container pattern (commercial license handling)
- FlexLM license server is injected at runtime
- UBI 9 based

## Execution Example

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -e LM_LICENSE_FILE=27000@license-server \
  -v $(pwd):/workspace:Z \
  fpga/radiant:2024.2 \
  radiantc --job project.rdf
```

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [vivado_container.md](vivado_container.md) — Vivado container
- [oss_container.md](oss_container.md) — OSS FPGA container