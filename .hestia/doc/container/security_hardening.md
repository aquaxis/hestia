# コンテナセキュリティ強化

**対象領域**: container — セキュリティ
**ソース**: 設計仕様書 §12.5, §11.1

## 概要

コンテナ実行環境のセキュリティを多層的に強化する。Podman rootless の基本セキュリティに加え、成果物署名（cosign）、SBOM 生成（syft）、脆弱性スキャン（grype）、および§11.1 のセキュリティ設計方針に基づく保護措置を講じる。

## Podman セキュリティ方針（§11.1）

| 方針 | 実装 |
|------|------|
| デーモンレス | システムサービス不要（Docker の dockerd に依存しない）|
| UID 一致 | `--userns=keep-id` でファイル権限問題を回避 |
| SELinux 対応 | `--security-opt=label=type:container_runtime_t` |
| 権限昇格防止 | `--security-opt=no-new-privileges` |
| ライセンス保護 | ベンダーライセンスファイルの読み取り専用マウント |
| ネットワーク隔離 | ビルドコンテナは `--network=none` |
| 知的財産保護 | HDL ソース・ビットストリームのコンテナ外流出防止 |

## コンテナ運用パターン別セキュリティ

| パターン | セキュリティ設定 |
|---------|----------------|
| バッチビルド | `--rm`, `--network=none`, `--security-opt=no-new-privileges` |
| GUI 起動 | X11/Wayland フォワード、`--security-opt=label=type:container_runtime_t` |
| systemd サービス | Podman Quad 定義（.container ファイル）|
| デバイスアクセス | `--device /dev/bus/usb`（JTAG プログラミングのみ）|

## 成果物署名（cosign、keyless OIDC）

```bash
# キーレス署名（GitHub Actions 上）
COSIGN_EXPERIMENTAL=1 cosign sign ghcr.io/hestia/fpga/oss:latest \
  --attachment sbom.spdx.json

# SBOM attestation
cosign attach sbom --sbom .hestia/containers/fpga/oss/sbom.spdx.json \
  ghcr.io/hestia/fpga/oss:latest
```

### 署名なしイメージの拒否ポリシー

- `podman-runtime::run_build()` の前に `cosign verify` を実行
- 署名検証失敗時はコンテナ起動を拒否（`SignatureVerificationError`）
- 開発環境のみ `HESTIA_ALLOW_UNSIGNED=1` で回避可（本番は常に検証）

## SBOM 生成（syft）

```bash
# SPDX 形式
syft ghcr.io/hestia/fpga/oss:latest -o spdx-json > sbom.spdx.json

# CycloneDX 形式
syft ghcr.io/hestia/fpga/oss:latest -o cyclonedx-json > sbom.cdx.json
```

## 脆弱性スキャン（grype）

```bash
grype ghcr.io/hestia/fpga/oss:latest \
  --fail-on high \
  --only-fixed \
  -o json > vuln.json
```

### 評価ゲート閾値

| 重大度 | 閾値 | 挙動 |
|-------|-----|------|
| Critical | 0 | ビルド失敗（push 不可）|
| High（修正可能）| 0 | ビルド失敗 |
| High（修正不可）| 記録 | 警告 + 例外承認 |
| Medium 以下 | 記録 | 通常 push 可 |

## ベンダーライセンス保護

- インストーラ・ライセンスファイルはイメージに焼き込まない（`--secret` マウントのみ）
- 実行時ライセンス（FlexLM 等）は `podman run -e LM_LICENSE_FILE=...` で注入
- `.hestia/secure/` はモード 0700、`.gitignore` で Git 管理外

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [registry_config.md](registry_config.md) — レジストリ管理
- [vivado_container.md](vivado_container.md) — Vivado コンテナ