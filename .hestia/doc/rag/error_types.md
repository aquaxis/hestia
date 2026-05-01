# rag-conductor エラーコード

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）

## エラーコード範囲

rag-conductor のエラーコードは **-32600 〜 -32699** の範囲を使用する。

## エラーカテゴリ

### Ingest（取り込み）

| コード | 名称 | 説明 |
|-------|------|------|
| -32600 | INGEST_FAILED | 取り込み処理失敗 |
| -32601 | INGEST_SOURCE_NOT_FOUND | 取り込みソース未検出 |
| -32602 | INGEST_UNSUPPORTED_SOURCE_TYPE | 未対応のソース種別 |

### PDF

| コード | 名称 | 説明 |
|-------|------|------|
| -32610 | PDF_TEXT_EXTRACTION_FAILED | PDF テキスト抽出失敗 |
| -32611 | PDF_OCR_FAILED | OCR 処理失敗（Tesseract） |
| -32612 | PDF_TABLE_EXTRACTION_FAILED | 表抽出失敗（Camelot） |
| -32613 | PDF_IMAGE_EXTRACTION_FAILED | 画像抽出失敗 |

### Web

| コード | 名称 | 説明 |
|-------|------|------|
| -32620 | WEB_FETCH_FAILED | HTTP 取得失敗 |
| -32630 | WEB_ROBOTS_TXT_DENIED | robots.txt によりアクセス拒否 |
| -32621 | WEB_CONTENT_EXTRACTION_FAILED | 本文抽出失敗（trafilatura） |
| -32622 | WEB_LANGUAGE_DETECTION_FAILED | 言語検出失敗（CLD3 / fasttext） |

### Quality Gate（品質ゲート）

| コード | 名称 | 説明 |
|-------|------|------|
| -32640 | QUALITY_GATE_FAILED | 品質ゲート不合格 |
| -32641 | QUALITY_MIN_LENGTH | 最小文字数不足 |
| -32642 | QUALITY_MAX_LENGTH | 最大文字数超過 |
| -32643 | QUALITY_DUPLICATE | 重複検出（cosine >= 0.95） |
| -32644 | QUALITY_UTF8_INVALID | UTF-8 妥当性エラー |
| -32645 | QUALITY_OCR_LOW_CONFIDENCE | OCR 信頼度不足（< 60%） |

### Chunk / Embedding

| コード | 名称 | 説明 |
|-------|------|------|
| -32650 | CHUNK_SPLIT_FAILED | チャンク分割失敗 |
| -32651 | EMBEDDING_FAILED | 埋め込み生成失敗（Ollama） |
| -32652 | EMBEDDING_MODEL_NOT_FOUND | 埋め込みモデル未検出 |
| -32653 | UPSERT_FAILED | ベクトル DB への upsert 失敗 |

### Vector / Search

| コード | 名称 | 説明 |
|-------|------|------|
| -32660 | VECTOR_DB_CONNECTION_FAILED | ベクトル DB 接続失敗（Chroma / Qdrant） |
| -32661 | SEARCH_FAILED | 検索実行失敗 |
| -32662 | SEARCH_TIMEOUT | 検索タイムアウト |

### License / PII

| コード | 名称 | 説明 |
|-------|------|------|
| -32670 | LICENSE_VIOLATION | ライセンス違反（unknown / vendor-proprietary without terms_accepted） |
| -32671 | PII_DETECTION_FAILED | PII 検出処理失敗 |
| -32672 | PII_MASKING_FAILED | PII マスキング処理失敗 |

### Scheduler / Cache

| コード | 名称 | 説明 |
|-------|------|------|
| -32680 | SCHEDULER_QUEUE_FULL | 取り込みキュー満杯 |
| -32681 | CACHE_EXPIRED | キャッシュ期限切れ |
| -32682 | CACHE_READ_ERROR | キャッシュ読み出しエラー |

## IngestJobStatus

| ステータス | 説明 |
|-----------|------|
| `Queued` | キュー待ち |
| `Processing` | 処理中 |
| `Completed` | 完了 |
| `Failed` | 失敗 |
| `PartiallyCompleted` | 一部完了（一部ソースが失敗） |

## 関連ドキュメント

- [rag/message_methods.md](message_methods.md) — rag.* メソッド一覧
- [rag/ingest_pipeline.md](ingest_pipeline.md) — 取り込みパイプライン
- [rag/search_engine.md](search_engine.md) — 検索エンジン仕様
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ