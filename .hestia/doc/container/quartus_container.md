# Quartus コンテナ詳細

**対象領域**: container — FPGA ツールコンテナ
**ソース**: 設計仕様書 §12.2

## 概要

Intel/Altera Quartus Prime Pro 25.1 を実行するためのコンテナイメージ `fpga/quartus:25.1`。UBI 9 ベースで QSF/QIP プロジェクト形式による合成・配置配線を実行する。

## イメージ構成

| 項目 | 値 |
|------|-----|
| イメージ名 | `fpga/quartus:25.1` |
| ベースイメージ | `registry.access.redhat.com/ubi9/ubi:9.5` |
| 主要ツール | Intel Quartus Prime Pro 25.1 |
| ライセンス | FlexLM（QPF/QSF）|
| ユーザー | `hestia` (UID 1000) |

## 特記事項

- Vivado コンテナパターンを踏襲（商用ライセンス扱い）
- QSF / QIP プロジェクトファイルをマウントしてバッチ実行
- FlexLM ライセンスサーバは実行時に `--env` で注入

## 実行例

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

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [vivado_container.md](vivado_container.md) — Vivado コンテナ
- [security_hardening.md](security_hardening.md) — セキュリティ強化