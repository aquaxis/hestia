# Efinity コンテナ詳細

**対象領域**: container — FPGA ツールコンテナ
**ソース**: 設計仕様書 §12.2

## 概要

Efinix Efinity 2025.2 を実行するためのコンテナイメージ `fpga/efinity:2025.2`。Ubuntu 24.04 ベースで、Efinity 同梱の Python API を使用した合成・配置配線を実行する。

## イメージ構成

| 項目 | 値 |
|------|-----|
| イメージ名 | `fpga/efinity:2025.2` |
| ベースイメージ | `docker.io/library/ubuntu:24.04` |
| 主要ツール | Efinix Efinity 2025.2 |
| ライセンス | Efinity 同梱 Python（独自ライセンス）|
| ユーザー | `hestia` (UID 1000) |

## 特記事項

- 他の商用ツールと異なり Python API ベースで操作
- Vivado / Quartus と異なり FlexLM は使用しない
- Ubuntu ベース（UBI 9 ではない）

## 実行例

```bash
podman run --rm \
  --userns=keep-id \
  --security-opt=no-new-privileges \
  --network=none \
  -v $(pwd):/workspace:Z \
  fpga/efinity:2025.2 \
  python3 -m efinity.flow run --project project.xml
```

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [vivado_container.md](vivado_container.md) — Vivado コンテナ
- [oss_container.md](oss_container.md) — OSS FPGA コンテナ