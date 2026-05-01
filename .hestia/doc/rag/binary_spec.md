# rag-conductor CLI バイナリ仕様

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §13.7（3252-3491行目付近）

## バイナリ名

`hestia-rag-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `ingest` | ドキュメント取り込み（PDF / Web / ソースコード / conductor-work-logs） |
| `search` | ベクトル類似検索（query / top_k / filter / trace_id） |
| `cleanup` | 古いインデックスのクリーンアップ（retention 期限切れデータ削除） |
| `status` | インデックス状況・メトリクス表示 |

## 共通オプション（CommonOpts）

| オプション | 値 | 説明 |
|-----------|---|------|
| `--output` | `human` \| `json` | 出力フォーマット（既定: human） |
| `--timeout` | `<秒>` | RPC タイムアウト |
| `--registry` | `<path>` | agent-cli レジストリパス |
| `--config` | `<path>` | 設定ファイルパス |
| `--verbose` | — | 詳細ログ出力 |

## Exit Code

| Exit Code | 意味 |
|-----------|------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## ingest サブコマンド オプション

| オプション | 値 | 説明 |
|-----------|---|------|
| `--source-type` | `pdf` / `web` / `source` / `all` | 取り込みソース種別 |
| `--file-path` | `<path>` | 単一ファイル取り込み |
| `--url` | `<url>` | URL 取り込み |
| `--source-id` | `<id>` | ソース ID 指定 |
| `--force` | — | 強制再取り込み |
| `--incremental` | — | 増分更新モード |

## search サブコマンド オプション

| オプション | 値 | 説明 |
|-----------|---|------|
| `--query` | `<text>` | 検索クエリ |
| `--top-k` | `<n>` | 取得件数（既定: 5） |
| `--filter` | `<json>` | フィルタ条件 |
| `--trace-id` | `<id>` | トレース ID |

## CLI 使用例

```bash
# PDF 取り込み
hestia rag ingest --source-type pdf --file-path datasheets/STM32F103.pdf

# Web 取り込み
hestia rag ingest --source-type web --url https://example.com/guide

# 全ソース増分取り込み
hestia rag ingest --source-type all --incremental

# 検索
hestia rag search --query "STM32F103 SPI ピン配置" --top-k 5

# クリーンアップ
hestia rag cleanup

# 状況表示
hestia rag status
```

## 関連ドキュメント

- [rag/config_schema.md](config_schema.md) — config.toml [rag] スキーマ
- [rag/message_methods.md](message_methods.md) — rag.* メソッド一覧
- [rag/ingest_pipeline.md](ingest_pipeline.md) — 取り込みパイプライン
- [rag/search_engine.md](search_engine.md) — 検索エンジン仕様