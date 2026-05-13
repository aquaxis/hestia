# OSS FPGA Container Details

**Domain**: container — FPGA Tool Container
**Source**: Design Specification §12.2

## Overview

Container image `fpga/oss:latest` for running a complete OSS FPGA flow with Yosys + nextpnr + icestorm + Verilator. No commercial license required; suitable for education, prototyping, and small-scale FPGA development.

## Image Configuration

| Item | Value |
|------|-----|
| Image name | `fpga/oss:latest` |
| Base image | `docker.io/library/ubuntu:24.04` |
| Primary tools | Yosys + nextpnr-ice40 + nextpnr-ecp5 + icestorm + Verilator |
| License | Not required (OSS) |
| User | `hestia` (UID 1000) |

## Containerfile (Auto-Generated)

```dockerfile
ARG BASE_IMAGE=docker.io/library/ubuntu:24.04

# Stage 1: Build tool acquisition
FROM ${BASE_IMAGE} AS build
ENV DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      build-essential git cmake ninja-build \
      yosys nextpnr-ice40 nextpnr-ecp5 \
      fpga-icestorm icestorm verilator \
      python3 python3-pip && \
    rm -rf /var/lib/apt/lists/*

# Stage 2: Runtime (lightweight)
FROM ${BASE_IMAGE} AS runtime
LABEL org.opencontainers.image.source="https://github.com/hestia/hestia" \
      org.opencontainers.image.title="Hestia FPGA OSS Toolchain" \
      org.opencontainers.image.licenses="ISC,GPL-3.0,LGPL-2.1"
ENV DEBIAN_FRONTEND=noninteractive LC_ALL=C.UTF-8 \
    HESTIA_CONTAINER_ROLE=fpga-oss
COPY --from=build /usr/bin/yosys /usr/bin/nextpnr-ice40 /usr/bin/nextpnr-ecp5 /usr/bin/
COPY --from=build /usr/bin/icepack /usr/bin/icetime /usr/bin/verilator /usr/bin/
COPY --from=build /usr/share/yosys /usr/share/yosys
RUN groupadd -g 1000 hestia && useradd -u 1000 -g hestia -m hestia
USER hestia
WORKDIR /workspace
HEALTHCHECK --interval=60s --timeout=5s CMD yosys -V || exit 1
CMD ["bash"]
```

## Execution Example

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -v $(pwd):/workspace:Z \
  fpga/oss:latest \
  yosys -p "synth_ice40 -top top -json out.json" src/top.v
```

## Supported Devices

| Device Family | nextpnr Target |
|----------------|-------------------|
| iCE40 (LP/HX) | nextpnr-ice40 |
| ECP5 | nextpnr-ecp5 |
| Gowin | nextpnr-gowin (separate installation) |

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [vivado_container.md](vivado_container.md) — Vivado container
- [efinity_container.md](efinity_container.md) — Efinity container