# Quartus Container Details

**Domain**: container — FPGA Tool Container
**Source**: Design Specification §12.2

## Overview

Container image `fpga/quartus:25.1` for running Intel/Altera Quartus Prime Pro 25.1. UBI 9 based, performing synthesis and place-and-route via QSF/QIP project format.

## Image Configuration

| Item | Value |
|------|-----|
| Image name | `fpga/quartus:25.1` |
| Base image | `registry.access.redhat.com/ubi9/ubi:9.5` |
| Primary tool | Intel Quartus Prime Pro 25.1 |
| License | FlexLM (QPF/QSF) |
| User | `hestia` (UID 1000) |

## Notable Points

- Follows the Vivado container pattern (commercial license handling)
- QSF / QIP project files are mounted for batch execution
- FlexLM license server is injected at runtime via `--env`

## Execution Example

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -e LM_LICENSE_FILE=27000@license-server \
  -v $(pwd):/workspace:Z \
  fpga/quartus:25.1 \
  quartus_sh --flow compile top.qpf
```

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [vivado_container.md](vivado_container.md) — Vivado container
- [security_hardening.md](security_hardening.md) — Security hardening