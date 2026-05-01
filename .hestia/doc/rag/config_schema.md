# rag-conductor 設定スキーマ

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §13.7.6（3307-3323行目付近）

## config.toml [rag] セクション

rag-conductor の動作設定を宣言的に定義する。

### 設定項目一覧

| フィールド | 型 | 既定値 | 説明 |
|-----------|---|-------|------|
| `backend` | string | `"chroma"` | ベクトル DB バックエンド（`chroma` / `qdrant`） |
| `embedding_model` | string | `"nomic-embed-text"` | 埋め込みモデル名（768 次元） |
| `top_k` | integer | 5 | 検索時の取得件数 |
| `chunk_size` | integer | 1000 | チャンク分割サイズ（トークン） |
| `chunk_overlap` | integer | 200 | チャンクオーバーラップ（トークン） |
| `vector_db_url` | string | `"http://localhost:8000"` | ベクトル DB 接続 URL |
| `batch_size` | integer | 32 | 埋め込みバッチサイズ |
| `retention_days` | integer | 90 | 既存ソース保持期間（日） |
| `retention_days_work_log` | integer | 365 | 自己学習 conductor-work-logs/ の保持期間（日） |
| `self_learning_enabled` | boolean | true | 自己学習機能（§13.7.8）の ON/OFF |
| `queue_dir` | string | `".hestia/rag/queue"` | rag offline 時のローカルバッファ |

### 設定例

```toml
[rag]
backend = "chroma"
embedding_model = "nomic-embed-text"
top_k = 5
chunk_size = 1000
chunk_overlap = 200
vector_db_url = "http://localhost:8000"
batch_size = 32
retention_days = 90
retention_days_work_log = 365
self_learning_enabled = true
queue_dir = ".hestia/rag/queue"
```

## ナレッジベース構成

```
.hestia/rag/
├── sources/                    # 取得元の生データ
│   ├── conductor-work-logs/    # 自己学習用蓄積領域（§13.7.8）
│   │   ├── ai/      YYYY-MM-DD_<task_id>.md
│   │   ├── rtl/     YYYY-MM-DD_<task_id>.md
│   │   ├── fpga/    YYYY-MM-DD_<task_id>.md
│   │   ├── asic/    YYYY-MM-DD_<task_id>.md
│   │   ├── pcb/     YYYY-MM-DD_<task_id>.md
│   │   ├── hal/     YYYY-MM-DD_<task_id>.md
│   │   ├── apps/    YYYY-MM-DD_<task_id>.md
│   │   └── debug/   YYYY-MM-DD_<task_id>.md
│   ├── datasheets/             # 外部データシート PDF
│   └── vendor-guides/          # ベンダーガイド
├── chunks/                     # チャンク分割済みテキスト
├── embeddings/                 # ベクトル化済み（Chroma/Qdrant にインデックス）
├── index-metadata.toml
├── queries/                    # クエリログ・ヒット率
├── quarantine/                 # 品質ゲート不合格データ（保留）
└── queue/                      # offline 時のローカルバッファ
```

## 技術スタック

| 区分 | 技術 |
|------|------|
| バイナリ | `hestia-rag-conductor`（Rust + tokio） |
| ベクトル DB | Chroma（既定） / Qdrant |
| 埋め込み | Ollama `nomic-embed-text`（768 次元） |
| Rust 部分 | `rag-ingest` クレート（PDF 7 段 / Web 8 段パイプライン） |
| TS 部分 | `rag-engine`（Vector Search / Embedding / Citation Generation） |
| PDF 解析 | PyPDF / pdfplumber / Tesseract OCR / Camelot |
| Web 取得 | trafilatura / BeautifulSoup / CLD3 / fasttext |

## 関連ドキュメント

- [rag/binary_spec.md](binary_spec.md) — hestia-rag-cli バイナリ仕様
- [rag/ingest_pipeline.md](ingest_pipeline.md) — 取り込みパイプライン
- [rag/search_engine.md](search_engine.md) — 検索エンジン仕様
- [rag/state_machines.md](state_machines.md) — インデックス状態遷移