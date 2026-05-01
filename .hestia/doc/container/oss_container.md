# OSS FPGA コンテナ詳細

**対象領域**: container — FPGA ツールコンテナ
**ソース**: 設計仕様書 §12.2

## 概要

Yosys + nextpnr + icestorm + Verilator による完全 OSS FPGA フローを実行するコンテナイメージ `fpga/oss:latest`。商用ライセンス不要で、教育・プロトタイピング・小規模 FPGA 開発に対応する。

## イメージ構成

| 項目 | 値 |
|------|-----|
| イメージ名 | `fpga/oss:latest` |
| ベースイメージ | `docker.io/library/ubuntu:24.04` |
| 主要ツール | Yosys + nextpnr-ice40 + nextpnr-ecp5 + icestorm + Verilator |
| ライセンス | 不要（OSS）|
| ユーザー | `hestia` (UID 1000) |

## Containerfile（自動生成）

```dockerfile
ARG BASE_IMAGE=docker.io/library/ubuntu:24.04

# Stage 1: ビルドツール取得
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

# Stage 2: ランタイム（軽量）
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

## 実行例

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -v $(pwd):/workspace:Z \
  fpga/oss:latest \
  yosys -p "synth_ice40 -top top -json out.json" src/top.v
```

## 対応デバイス

| デバイスファミリ | nextpnr ターゲット |
|----------------|-------------------|
| iCE40 (LP/HX) | nextpnr-ice40 |
| ECP5 | nextpnr-ecp5 |
| Gowin | nextpnr-gowin（別途インストール）|

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [vivado_container.md](vivado_container.md) — Vivado コンテナ
- [efinity_container.md](efinity_container.md) — Efinity コンテナ