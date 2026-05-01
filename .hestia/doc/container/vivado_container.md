# Vivado コンテナ詳細

**対象領域**: container — FPGA ツールコンテナ
**ソース**: 設計仕様書 §12.2

## 概要

AMD/Xilinx Vivado 2025.2 を実行するためのコンテナイメージ `fpga/vivado:2025.2`。UBI 9 ベースで TCL バッチモードによるヘッドレス実行を主目的とする。

## イメージ構成

| 項目 | 値 |
|------|-----|
| イメージ名 | `fpga/vivado:2025.2` |
| ベースイメージ | `registry.access.redhat.com/ubi9/ubi:9.5` |
| 主要ツール | AMD Vivado 2025.2 |
| ライセンス | FlexLM（`LM_LICENSE_FILE`）|
| ユーザー | `hestia` (UID 1000) |

## Containerfile（自動生成）

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

# BuildKit secret で FlexLM ライセンスサーバ情報を注入
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

## TCL バッチモード実行

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

## ライセンス管理

- FlexLM ライセンスサーバのアドレスは `podman run -e LM_LICENSE_FILE=...` で実行時注入
- インストーラ・ライセンスファイルはイメージに焼き込まない（`--secret` マウントのみ）
- `.hestia/secure/` ディレクトリはモード 0700、`.gitignore` で Git 管理外

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [security_hardening.md](security_hardening.md) — セキュリティ強化
- [registry_config.md](registry_config.md) — レジストリ管理