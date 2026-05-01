# Radiant コンテナ詳細

**対象領域**: container — FPGA ツールコンテナ
**ソース**: 設計仕様書 §12.2

## 概要

Lattice Radiant 2024.2 を実行するためのコンテナイメージ `fpga/radiant:2024.2`。Lattice LIFCL / LFD2NX / iCE40 系 FPGA の合成・配置配線に使用する。

## イメージ構成

| 項目 | 値 |
|------|-----|
| イメージ名 | `fpga/radiant:2024.2` |
| ベースイメージ | `registry.access.redhat.com/ubi9/ubi:9.5` |
| 主要ツール | Lattice Radiant 2024.2 |
| ライセンス | FlexLM |
| ユーザー | `hestia` (UID 1000) |

## 特記事項

- Vivado コンテナパターンを踏襲（商用ライセンス扱い）
- FlexLM ライセンスサーバは実行時に注入
- UBI 9 ベース

## 実行例

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

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [vivado_container.md](vivado_container.md) — Vivado コンテナ
- [oss_container.md](oss_container.md) — OSS FPGA コンテナ