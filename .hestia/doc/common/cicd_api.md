# CI/CD API

**対象領域**: common — CI/CD
**ソース**: 設計仕様書 §13.5

## 概要

CI/CD パイプラインを宣言的に定義し、複数バックエンド（GitHub Actions / GitLab CI / Local）で実行する共有サービス。agent-cli peer `cicd` として提供される。

## 対応バックエンド

| Backend | 識別子 | 用途 |
|---------|-------|------|
| `GithubActions` | `github_actions` | GitHub ホスティング時の CI/CD |
| `GitlabCi` | `gitlab_ci` | GitLab ホスティング時の CI/CD |
| `LocalPipeline` | `local` | ローカル実行（オフライン開発・検証）|

## 主要型

### PipelineDefinition

パイプラインの宣言的定義。ステージ・ジョブ・条件を構造化する。

### PipelineStage

ステージ定義。複数ジョブを含み、並列または逐次実行を制御。

### PipelineJob

ジョブ定義。実行コマンド・環境・成果物・リトライ設定を含む。

### StageCondition

| 値 | 意味 |
|----|------|
| `Always` | 常に実行 |
| `OnSuccess` | 前段成功時のみ |
| `OnFailure` | 前段失敗時のみ |
| `Custom` | カスタム条件式 |

## 制御パラメータ

パイプラインは JSON 経由で以下を制御:

| パラメータ | 説明 |
|----------|------|
| Artifact リテンション | 成果物の保持期間 |
| Retry policy | ジョブ失敗時のリトライ回数・間隔 |
| Timeout secs | ジョブごとのタイムアウト |
| Cache key | ビルドキャッシュのキー |

## CI/CD 統合例（GitHub Actions）

```yaml
name: Container Build & Publish
on:
  push:
    paths:
      - '.hestia/containers/**'
  schedule:
    - cron: '0 2 * * 1'   # 毎週月曜 02:00 UTC

jobs:
  build:
    strategy:
      matrix:
        image: [fpga/oss, asic/openlane, pcb/kicad, debug/tools]
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: podman build --file .hestia/containers/${{ matrix.image }}/Containerfile ...
      - name: SBOM
        run: syft ... -o spdx-json > sbom.spdx.json
      - name: Vulnerability scan
        run: grype ... --fail-on high --only-fixed
      - name: Sign (cosign keyless)
        run: cosign sign ...
```

## Observability 連携

| メトリクス | 型 | 意味 |
|----------|----|------|
| `hestia_container_build_total{image,status}` | Counter | ビルド回数 |
| `hestia_container_build_duration_seconds{image,stage}` | Histogram | ステージ別所要時間 |

## 関連ドキュメント

- [observability.md](observability.md) — 監視・メトリクス
- [container_manager](../container/container_manager.md) — コンテナ管理
- [agent_cli_messaging.md](agent_cli_messaging.md) — メッセージング仕様