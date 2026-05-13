# Vivado Container Details

**Domain**: container — FPGA Tool Container
**Source**: Design Specification §12.2

## Overview

Container image `fpga/vivado:2025.2` for running AMD/Xilinx Vivado 2025.2. UBI 9 based, primarily intended for headless execution in TCL batch mode.

## Image Configuration

| Item | Value |
|------|-----|
| Image name | `fpga/vivado:2025.2` |
| Base image | `registry.access.redhat.com/ubi9/ubi:9.5` |
| Primary tool | AMD Vivado 2025.2 |
| License | FlexLM (`LM_LICENSE_FILE`) |
| User | `hestia` (UID 1000) |

## Containerfile (Auto-Generated)

```dockerfile
ARG BASE_IMAGE=registry.access.redhat.com/ubi9/ubi:9.5
ARG VIVADO_VERSION=2024.2
ARG VIVADO_INSTALLER=/opt/installer/Xilinx_Unified_${VIVADO_VERSION}_Lin64.bin

FROM ${BASE_IMAGE} AS install
ENV LC_ALL=C.UTF-8
RUN dnf install -y glibc-locale-source glibc-langpack-en \
      libX11 libXext libXrender libXtst libXi \
      libglvnd-glx libglvnd-opengl libstdc++ \
      tar gzip which && dnf clean all

# Inject FlexLM license server info via BuildKit secret
ARG VIVADO_INSTALL_OPTS="-agreeToEULA -ignore_warning"
RUN --mount=type=secret,id=vivado_installer,target=/opt/installer/vivado.bin,required=true \
    --mount=type=secret,id=vivado_config,target=/opt/installer/install_config.txt \
    chmod +x /opt/installer/vivado.bin && \
    /opt/installer/vivado.bin --keep --noexec --target /opt/installer/extracted && \
    /opt/installer/extracted/xsetup \
      --config /opt/installer/install_config.txt ${VIVADO_INSTALL_OPTS} && \
    rm -rf /opt/installer/extracted

FROM ${BASE_IMAGE} AS runtime
LABEL org.opencontainers.image.title="Hestia FPGA Vivado ${VIVADO_VERSION}" \
      org.opencontainers.image.licenses="proprietary"
COPY --from=install /opt/Xilinx /opt/Xilinx

ENV XILINX_ROOT=/opt/Xilinx \
    PATH=/opt/Xilinx/Vivado/${VIVADO_VERSION}/bin:$PATH \
    HESTIA_CONTAINER_ROLE=fpga-vivado

RUN groupadd -g 1000 hestia && useradd -u 1000 -g hestia -m hestia
USER hestia
WORKDIR /workspace
HEALTHCHECK --interval=120s --timeout=10s CMD vivado -version || exit 1
CMD ["bash"]
```

## TCL Batch Mode Execution

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -e LM_LICENSE_FILE=27000@license-server \
  -v $(pwd):/workspace:Z \
  fpga/vivado:2025.2 \
  vivado -mode batch -source synth.tcl
```

## License Management

- FlexLM license server address is injected at runtime via `podman run -e LM_LICENSE_FILE=...`
- Installer and license files are not baked into the image (mounted only via `--secret`)
- `.hestia/secure/` directory has mode 0700, excluded from Git via `.gitignore`

## Related Documentation

- [container_manager.md](container_manager.md) — container-manager overview
- [security_hardening.md](security_hardening.md) — Security hardening
- [registry_config.md](registry_config.md) — Registry management