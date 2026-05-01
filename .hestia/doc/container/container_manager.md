# container-manager 全体仕様

**対象領域**: container — コンテナ管理
**ソース**: 設計仕様書 §12

## 概要

container-manager はコンテナ実行を選択した場合に使用されるコンテナライフサイクル管理システム。Containerfile 自動生成、ビルドフロー、プロビジョニング、署名、レジストリ、CI/CD 統合、運用ルールを提供する。ローカル実行を選択した場合は適用外。

## モジュール構成

| モジュール | 役割 |
|-----------|------|
| builder | Containerfile 自動生成・ビルド（ツール定義ファイルに基づく）|
| registry | コンテナイメージのレジストリ管理（ローカル/リモート）|
| updater | イメージの差分更新・バージョン管理・自動リビルド |
| provisioner | ツール定義ファイルに基づくプロビジョニング（パッケージインストール）|
| tool_updater | ツールバージョン検出・互換性チェック・段階的更新 |

## Containerfile 自動生成（FR-CTN-11）

`container.toml` を入力として、minijinja テンプレートエンジンで Containerfile を自動生成する。

```
container.toml → builder::parse → Container Spec (Rust 構造体)
    → Containerfile テンプレート（minijinja）
    → Containerfile（テキスト）
    → podman build（§12.3）
```

### 8 イメージ構成

| イメージ | ベース | 主要ツール | ライセンス取扱 |
|---------|-------|----------|--------------|
| `fpga/vivado:2025.2` | `ubi9/ubi:9.5` | AMD Vivado 2025.2 | FlexLM |
| `fpga/quartus:25.1` | `ubi9/ubi:9.5` | Intel Quartus Prime Pro 25.1 | FlexLM |
| `fpga/efinity:2025.2` | `ubuntu:24.04` | Efinix Efinity 2025.2 | Efinity Python |
| `fpga/radiant:2024.2` | `ubi9/ubi:9.5` | Lattice Radiant 2024.2 | FlexLM |
| `fpga/oss:latest` | `ubuntu:24.04` | Yosys + nextpnr + icestorm + Verilator | 不要（OSS）|
| `asic/openlane:latest` | `ubuntu:24.04` | OpenLane 2 + Yosys + OpenROAD + Magic | 不要（OSS）|
| `pcb/kicad:latest` | `ubuntu:24.04` | KiCad + SKiDL + Freerouting | 不要（OSS）|
| `debug/tools:latest` | `ubuntu:24.04` | OpenOCD + sigrok + PulseView + pyOCD | 不要（OSS）|

## ビルドフロー（FR-CTN-12）

### マルチステージ戦略

- **Stage 1（install / build）**: インストーラ・ビルド依存を含む大きいレイヤ（最終イメージには残さない）
- **Stage 2（runtime）**: 必要最小限のランタイム（サイズは Stage 1 の 30〜50%）

### レイヤキャッシュ戦略

| レイヤ種別 | 頻度 | 配置 |
|----------|-----|------|
| ベース OS | 低頻度 | 最上位 |
| 依存ライブラリ | 中頻度 | 2 番目 |
| EDA ツール本体 | 低頻度 | 3 番目（個別 stage）|
| 設定 / スクリプト | 高頻度 | 最下位 |

### ビルド所要時間監視

- OSS イメージ < 10 分、商用イメージ < 30 分
- `hestia_container_build_duration_seconds{image,stage}` メトリクスで監視

## プロビジョニング（FR-CTN-13）

| install_method | コマンド |
|---------------|---------|
| `apt` | `apt-get install -y ${package}` |
| `dnf` | `dnf install -y ${package}` |
| `tarball` | `wget -O - ${url} \| tar -xz -C ${prefix}` |
| `install_script` | `bash -c "${install_script}"` |
| `pip` | `pip install --no-cache-dir ${package}` |
| `cargo` | `cargo install ${package}` |

## 成果物署名・SBOM（FR-CTN-14）

```
podman build → イメージ生成 → syft で SBOM 生成 → grype で脆弱性スキャン
    → 評価ゲート → cosign sign（keyless） → podman push
```

### 評価ゲート閾値

| 重大度 | 閾値 | 挙動 |
|-------|-----|------|
| Critical | 0 | ビルド失敗（push 不可）|
| High（修正可能）| 0 | ビルド失敗 |
| High（修正不可）| 記録 | 警告 + 例外承認 |
| Medium 以下 | 記録 | 通常 push 可 |

## CI/CD 統合

週次ビルド（毎週月曜 02:00 UTC）、パッチバージョンは自動、マイナーは Review Agent 承認必須、メジャーは手動トリガ（Canary 戦略）。

## 運用ルール

1. 週次ビルドでベースイメージ + 依存更新を取り込み再ビルド
2. パッチバージョン: 自動ビルド + 自動 push
3. マイナーバージョン: 自動ビルド後、Review Agent 経由で 1 人承認必須
4. メジャーバージョン: 手動トリガ、Canary 戦略で段階的展開
5. 失敗時: `action-log` に記録、PatcherAgent に通知
6. イメージサイズ上限: OSS 5GB、商用 20GB
7. BuildKit secret 管理: `.hestia/secure/` は 0700 権限、Git 管理外
8. CVE 通知: Critical 発生時は即座セキュリティチームへエスカレーション

## 実装クレート構成

```
container-manager/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── builder/         # Containerfile 生成 + podman build
    │   ├── mod.rs
    │   ├── templates/*.j2
    │   └── parser.rs    # container.toml → ContainerSpec
    ├── registry/        # push/pull/retention（skopeo ラッパー）
    ├── updater/          # 差分更新 / 自動リビルド
    ├── provisioner/      # [tools.*] → インストールコマンド翻訳
    ├── tool_updater/     # バージョン検出・semver マッチ
    ├── signer/          # cosign ラッパー
    ├── sbom/            # syft / grype ラッパー
    └── observability.rs # メトリクス送信
```

## 関連ドキュメント

- [vivado_container.md](vivado_container.md) — Vivado コンテナ詳細
- [quartus_container.md](quartus_container.md) — Quartus コンテナ詳細
- [security_hardening.md](security_hardening.md) — セキュリティ強化
- [registry_config.md](registry_config.md) — レジストリ管理