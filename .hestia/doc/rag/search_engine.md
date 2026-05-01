# rag-conductor 検索エンジン仕様

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §13.7.5（3296-3305行目付近）

## RPC / CLI / メトリクス

### 主要 RPC

| RPC | パラメータ | 説明 |
|-----|----------|------|
| `rag.ingest` | source_type / file_path / url / source_id / all / force / incremental | ドキュメント取り込み |
| `rag.search` | query / top_k / filter / trace_id | ベクトル類似検索 |
| `rag.cleanup` | — | 古いインデックスのクリーンアップ |
| `rag.status` | — | インデックス状況・メトリクス |

### 自己学習 RPC（§13.7.8）

| RPC | パラメータ | 説明 |
|-----|----------|------|
| `rag.ingest_work.v1` | category / conductor / content / metadata | conductor 作業内容蓄積 |
| `rag.search_similar.v1` | query / top_k | 類似タスク検索 |
| `rag.search_bugfix.v1` | query / top_k | エラー修正事例検索 |
| `rag.search_design.v1` | query / top_k | 過去設計パラメータ検索 |

### CLI

```bash
hestia rag ingest --source-type <type> --file-path <path>
hestia rag search --query <text> --top-k <n>
hestia rag cleanup
hestia rag status
```

### MCP ツール

`hestia_rag_search` — Model Context Protocol 経由で外部ツールが RAG 検索を利用可能。

## ベクトル検索仕様

### 埋め込みモデル

| 項目 | 値 |
|------|---|
| モデル | `nomic-embed-text` |
| 次元数 | 768 |
| 実行環境 | Ollama（ローカル、プライバシー保護） |

### 検索フロー

```
1. クエリテキスト → 埋め込み生成（Ollama nomic-embed-text）
2. ベクトル DB（Chroma / Qdrant）で類似度検索（cosine similarity）
3. top-k 件の関連チャンクを取得
4. citation 生成（ソース・ページ番号・信頼度）
5. 結果返却（chunks + citations + メトリクス）
```

### TypeScript I/F

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

### フィルタリング

検索時のフィルタ条件により、特定ソース種別・conductor 種別等で絞り込みが可能。

```json
{
  "filter": {
    "source_type": "datasheet",
    "conductor": "fpga"
  }
}
```

## Prometheus メトリクス

| メトリクス名 | 型 | 説明 |
|-------------|---|------|
| `ingest_duration` | Histogram | 取り込み所要時間 |
| `docs_total` | Counter | 総ドキュメント数 |
| `chunks_total` | Counter | 総チャンク数 |
| `quarantine_total` | Counter | quarantine 保留数 |
| `incremental_skipped` | Counter | 増分更新でのスキップ数 |
| `license_violations` | Counter | ライセンス違反数 |
| `cache_size` | Gauge | キャッシュサイズ |
| `retrieval_seconds` | Histogram | 検索所要時間 |
| `hit_ratio` | Gauge | 検索ヒット率 |

## サブエージェント

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| planner | `rag-planner` | 取り込みプランニング | 1 |
| designer | `rag-designer` | 知識ベース仕様 | 1 |
| ingest | `rag-ingest-{source}` | ドキュメント取り込み | N（ソース並列） |
| search | `rag-search` | ベクトル検索 + reranking | 1（高負荷時 N） |
| quality_gate | `rag-quality` | 品質チェック | 1 |
| archivist | `rag-archivist` | 自己学習用蓄積パイプライン管理 | 1（高負荷時 N） |

## 関連ドキュメント

- [rag/binary_spec.md](binary_spec.md) — hestia-rag-cli バイナリ仕様
- [rag/ingest_pipeline.md](ingest_pipeline.md) — 取り込みパイプライン
- [rag/state_machines.md](state_machines.md) — インデックス状態遷移
- [rag/config_schema.md](config_schema.md) — config.toml [rag] スキーマ