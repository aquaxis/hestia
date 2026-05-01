# rag-conductor メッセージメソッド一覧

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §13.7.5（3296-3305行目付近）, §14（3492-3630行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `rag`。

## rag.* メソッド一覧

### 取り込み

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rag.ingest` | Request | ドキュメント取り込み（source_type / file_path / url / source_id / all / force / incremental 指定） |

### 検索

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rag.search` | Request | ベクトル類似検索（query / top_k / filter / trace_id 指定） |

### 管理

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rag.cleanup` | Request | 古いインデックスのクリーンアップ（retention 期限切れデータ削除） |
| `rag.status` | Request | インデックス状況・メトリクス取得 |

### 自己学習（§13.7.8）

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rag.ingest_work.v1` | Request | conductor 作業内容蓄積（design_case / bugfix_case / build_log 等） |
| `rag.search_similar.v1` | Request | 類似タスク検索（過去の同種タスク事例を取得） |
| `rag.search_bugfix.v1` | Request | エラー修正事例検索 |
| `rag.search_design.v1` | Request | 過去設計パラメータ検索 |

### conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |

## ペイロード例

### rag.ingest リクエスト

```json
{
  "method": "rag.ingest",
  "params": {
    "source_type": "pdf",
    "file_path": "datasheets/STM32F103.pdf",
    "force": false
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### rag.search リクエスト

```json
{
  "method": "rag.search",
  "params": {
    "query": "STM32F103 の SPI ピン配置",
    "top_k": 5,
    "filter": { "source_type": "datasheet" }
  },
  "id": "msg_2026-05-01T12:00:00Z_def456",
  "trace_id": "trace_xyz789"
}
```

### rag.ingest_work.v1 リクエスト

```json
{
  "method": "rag.ingest_work.v1",
  "params": {
    "category": "design_case",
    "conductor": "fpga",
    "content": "<markdown>",
    "metadata": { "target": "artix7", "outcome": "success" }
  },
  "id": "msg_2026-05-01T12:00:00Z_ghi789"
}
```

## TypeScript I/F

```typescript
interface RagQuery {
  text: string;
  top_k: number;
  filter?: Record<string, any>;
  trace_id?: string;
}

interface RagResult {
  chunks: RagChunk[];
  citations: Citation[];
  embedding_time_ms: number;
  retrieval_time_ms: number;
}
```

## MCP ツール

`hestia_rag_search` — Model Context Protocol 経由で外部ツールが RAG 検索を利用可能。

## 関連ドキュメント

- [rag/binary_spec.md](binary_spec.md) — hestia-rag-cli バイナリ仕様
- [rag/ingest_pipeline.md](ingest_pipeline.md) — 取り込みパイプライン
- [rag/search_engine.md](search_engine.md) — 検索エンジン仕様
- [rag/error_types.md](error_types.md) — rag-conductor エラーコード