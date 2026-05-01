# Observability

**対象領域**: common — 監視・観測
**ソース**: 設計仕様書 §13.6, §19.8

## 概要

全 Hestia コンポーネントの状態を構造化ログ / メトリクス / 分散トレーシングで可視化する。3 本柱（Logs / Metrics / Traces）を統合し、LLM 呼び出し統計 / エージェント状態 / 開発プロセス指標を集約する。agent-cli peer `observability` として提供される。

## 3 本柱

| 柱 | 技術 | エンドポイント | 保存先 |
|----|------|-------------|-------|
| Logs | 構造化 JSON（`tracing` クレート）| — | `.hestia/observability/logs/<YYYY-MM-DD>.jsonl` |
| Metrics | OpenMetrics 形式（`prometheus` クレート）| `localhost:9090/metrics` | Prometheus スクレイプ |
| Traces | OpenTelemetry（OTLP gRPC）| `:4317` | ローカル Tempo / Jaeger（オプション）|

## ヘルスチェックエンドポイント

| エンドポイント | 用途 |
|-------------|------|
| `:8080/health` | プロセス生存確認 |
| `:8080/ready` | サービス準備完了確認 |
| `:8080/live` | ライブネス確認 |

### HealthStatus

| 値 | 意味 |
|----|------|
| `Healthy` | 正常 |
| `Degraded` | 一部機能制限 |
| `Unhealthy` | 異常 |

## 構造化ログの共通フィールド

```json
{
  "timestamp": "2026-04-23T16:00:00.123Z",
  "level": "INFO",
  "trace_id": "01HE...",
  "span_id": "...",
  "component": "fpga-conductor",
  "event": "build.started",
  "target": "artix7_dev",
  "job_id": 42,
  "metadata": { ... }
}
```

## 主要メトリクス

### 共通メトリクス

| メトリクス名 | 型 | 説明 |
|-------------|-----|------|
| `hestia_build_total{conductor,status}` | Counter | ビルド回数（成功/失敗別）|
| `hestia_build_duration_seconds{conductor,step}` | Histogram | ステップ別所要時間 |
| `hestia_agent_active{skill}` | Gauge | スキル別アクティブ Agent 数 |
| `hestia_agent_pending_tasks{skill}` | Gauge | キュー長 |

### LLM メトリクス

| メトリクス名 | 型 | 説明 |
|-------------|-----|------|
| `hestia_llm_requests_total{model,status}` | Counter | LLM 呼び出し回数 |
| `hestia_llm_tokens_total{model,direction}` | Counter | 入出力トークン数 |
| `hestia_llm_latency_seconds{model}` | Histogram | レイテンシ分布 |

### RAG メトリクス

| メトリクス名 | 型 | 説明 |
|-------------|-----|------|
| `hestia_rag_retrieval_seconds` | Histogram | RAG 取得時間 |
| `hestia_rag_hit_ratio` | Gauge | 知識ベース有用ヒット率 |

### コンテナメトリクス

| メトリクス名 | 型 | 説明 |
|-------------|-----|------|
| `hestia_container_build_total{image,status}` | Counter | ビルド回数 |
| `hestia_container_build_duration_seconds{image,stage}` | Histogram | ステージ別所要時間 |
| `hestia_container_image_size_bytes{image,tag}` | Gauge | イメージサイズ |
| `hestia_container_vuln_total{image,severity}` | Gauge | 脆弱性件数 |
| `hestia_container_signature_verified{image}` | Gauge | 署名検証成功 |

### フィードバックループメトリクス

| メトリクス名 | 型 | 説明 |
|-------------|-----|------|
| `hestia_feedback_loops_total{outcome}` | Counter | フィードバックループ発生回数 |

## ConductorName 別集約

`ConductorName`（`Ai` / `Fpga` / `Asic` / `Pcb` / `Debug` / `Rag`）毎にメトリクス / ヘルスを集約。

## 開発プロセス KPI（派生メトリクス）

- 仕様 → 実装リードタイム
- テストベンチ先行度（TDD 遵守率）
- CoT 有無率 / 平均ステージ数
- ハルシネーション検出率（RAG `参考資料外` フラグ比）

## ダッシュボード

```bash
ai-cli observability dashboard --open
```

主要ビュー: Build Health / Agent Fleet / LLM Spend / Feedback Loop / Knowledge Coverage

## 運用ルール

- 全コンポーネントは `tracing` で構造化ログ出力
- `trace_id` は CoT / Action Log / Prompt Archive と共通
- メトリクスは 30 秒間隔でスクレイプ、90 日保持
- 異常検知: `hestia_build_duration_seconds` の p99 が通常比 2 倍で警告

## 関連ドキュメント

- [health_check_orchestration.md](health_check_orchestration.md) — ヘルスチェック
- [error_registry.md](error_registry.md) — エラーコード
- [agent_cli_messaging.md](agent_cli_messaging.md) — メッセージング