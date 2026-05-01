# 知識基盤オーケストレーター

**対象領域**: rag-conductor
**ソース**: 設計仕様書 §13.7（3252-3491行目）

---

## 概要

rag-conductor は知識基盤の構築（Ingest）・管理・検索を独立プロセスで提供する Conductor である。旧 `ai-conductor::rag-engine`（TypeScript + LangChain）と `rag-ingest`（Rust）は完全に rag-conductor に移行済み。ai-conductor からは agent-cli IPC の `rag` peer に対して `rag.*` 構造化メッセージで呼び出す。

---

## 構成と技術スタック

| 区分 | 技術 |
|------|------|
| バイナリ | `hestia-rag-conductor`（Rust + tokio） |
| ベクトル DB | Chroma（既定） / Qdrant |
| 埋め込み | Ollama `nomic-embed-text`（768 次元） |
| Rust 部分 | `rag-ingest` クレート（PDF 7 段 / Web 8 段パイプライン） |
| TS 部分 | `rag-engine`（Vector Search / Embedding / Citation Generation） |
| PDF 解析 | PyPDF / pdfplumber / Tesseract OCR（300 DPI、信頼度 >= 60%）/ Camelot（表抽出） |
| Web 取得 | trafilatura / BeautifulSoup / CLD3 / fasttext |

---

## ナレッジベース構成

```
.hestia/rag/
├── sources/                    # 取得元の生データ（PDF・HTML スナップショット）
│   ├── conductor-work-logs/    # 自己学習用蓄積領域
│   │   ├── ai/        YYYY-MM-DD_<task_id>.md
│   │   ├── rtl/       YYYY-MM-DD_<task_id>.md
│   │   ├── fpga/      YYYY-MM-DD_<task_id>.md
│   │   ├── asic/      YYYY-MM-DD_<task_id>.md
│   │   ├── pcb/       YYYY-MM-DD_<task_id>.md
│   │   ├── hal/       YYYY-MM-DD_<task_id>.md
│   │   ├── apps/      YYYY-MM-DD_<task_id>.md
│   │   └── debug/     YYYY-MM-DD_<task_id>.md
│   ├── datasheets/             # 外部資料
│   └── vendor-guides/          # ベンダーガイド
├── chunks/                     # チャンク分割済みテキスト
├── embeddings/                  # ベクトル化済み（Chroma/Qdrant にインデックス）
├── index-metadata.toml
├── queries/                     # クエリログ・ヒット率
├── quarantine/                  # 品質ゲート不合格データ（保留）
└── queue/                       # rag offline 時のローカルバッファ
```

---

## 取り込みパイプライン

### PDF 7段

テキスト抽出 → OCR フォールバック → 表抽出 → 画像抽出 → セクション認識 → メタデータ付与 → 共通パイプラインへ

### Web 8段

URL 列挙 → robots.txt 確認 → HTTP 取得 → 本文抽出 → ノイズ除去 → 言語検出 → メタデータ付与 → 共通パイプラインへ

### 共通6段

正規化 → 品質ゲート → チャンク分割（既定 1000 トークン / オーバーラップ 200）→ 埋め込み（Ollama）→ upsert（Chroma/Qdrant）→ ログ

---

## 品質ゲート6ルール

1. 最小／最大文字数
2. 言語検出
3. HTML ノイズ除去
4. 重複（cosine >= 0.95）
5. UTF-8 妥当性
6. OCR 信頼度

---

## 増分更新と運用

- ETag / SHA-256 で変更検出 → 増分更新（180 分の全再構築 → 3 分相当に短縮）
- ライセンス管理: OSS / free 許可、`vendor-proprietary`（`terms_accepted=true` 必須）、`CC-BY-*`（クレジット表示）、`unknown` 拒否
- PII マスキング: 原本は GPG 暗号化保管、インデックスはマスク済みテキストのみ
- キャッシュ保持: PDF 無期限 / Web 90 日 / quarantine 30 日

---

## RPC / CLI / メトリクス

### 主要 RPC

| メソッド | 役割 |
|---------|------|
| `rag.ingest` | 取り込み（source_type/file_path/url/source_id/all、force・incremental）|
| `rag.search` | 検索（query・top_k・filter・trace_id）|
| `rag.cleanup` | クリーンアップ |
| `rag.status` | ステータス確認 |

### 自己学習 RPC

| メソッド | 役割 |
|---------|------|
| `rag.ingest_work.v1` | conductor 作業内容の永続化（category 指定）|
| `rag.search_similar.v1` | 類似タスク検索（fpga.build, asic.synth 等）|
| `rag.search_bugfix.v1` | エラーシグネチャから過去修正事例検索 |
| `rag.search_design.v1` | 過去採用された設計パラメータ検索 |

- `IngestJobStatus`: `Queued` / `Processing` / `Completed` / `Failed` / `PartiallyCompleted`
- TypeScript I/F: `RagQuery { text, top_k, filter, trace_id }` / `RagResult { chunks, citations, embedding_time_ms, retrieval_time_ms }`
- MCP ツール: `hestia_rag_search`
- CLI: `hestia rag ingest|search|cleanup`

### Prometheus メトリクス

`ingest_duration` / `docs_total` / `chunks_total` / `quarantine_total` / `incremental_skipped` / `license_violations` / `cache_size` / `retrieval_seconds` / `hit_ratio` / `work_log_ingested_total` / `similar_task_hits` / `bugfix_search_latency_seconds`

---

## config.toml [rag] 設定

```toml
[rag]
backend = "chroma"                 # "chroma" | "qdrant"
embedding_model = "nomic-embed-text"
top_k = 5
chunk_size = 1000
chunk_overlap = 200
vector_db_url = "http://localhost:8000"
batch_size = 32
retention_days = 90                # 既存ソース（datasheets / web 等）
retention_days_work_log = 365      # 自己学習 conductor-work-logs/ の保持期間（design_case / bugfix_case は無期限）
self_learning_enabled = true       # 自己学習機能の ON/OFF
queue_dir = ".hestia/rag/queue"    # rag offline 時のローカルバッファ
```

---

## サブエージェント構成

rag-conductor は **planner / designer / ingest（複数）/ search / quality_gate / archivist** の6種類のサブエージェントを持ち、知識ベース構築・検索フローを分担する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で rag-conductor 本体（peer 名 `rag`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `rag-planner` | 取り込みプランニング（クロール戦略、ソース優先度、増分更新スケジュール）| 1 |
| **designer** | `rag-designer` | 知識ベース仕様（チャンク戦略、メタデータスキーマ、埋め込みモデル選定、retention ポリシー）| 1 |
| **ingest** | `rag-ingest-{source}` | ドキュメント取り込み（PDF 7 段パイプライン / Web 8 段パイプライン）| **N**（ソース数だけ並列起動）|
| **search** | `rag-search` | ベクトル検索 + reranking（Chroma/Qdrant、`top_k` 取得、citation 生成）| 1（高負荷時 N）|
| **quality_gate** | `rag-quality` | 品質チェック（PII 検出 / ライセンス判定 / 重複排除 / quarantine 管理）| 1 |
| **archivist** | `rag-archivist` | 自己学習用 conductor-work-logs/ への蓄積パイプライン管理。メタデータ正規化、PII 再マスキング検証、古いログの集約・要約 | 1（高負荷時 N）|

**フロー**: planner → designer → ingest（ソース並列）→ quality_gate → search（検索リクエスト時）。自己学習は archivist が独立フローで他 conductor からの `rag.ingest_work.v1` を処理。

---

## 自己学習機能

rag-conductor が稼働中の場合、他の全 conductor および各サブエージェントは完了した作業内容を自動的に rag-conductor へ送信し、知識ベースに永続化する。蓄積された事例は次回以降の同種タスクで検索され、AI エージェントの判断材料として注入される（自己学習ループ）。

### 自動蓄積カテゴリ

| カテゴリ | 内容 | 送信元 | 送信タイミング |
|---------|------|--------|------------|
| **design_case** | 成功した設計パラメータ + ビルド結果サマリ | 全 conductor | ビルド成功時 |
| **bugfix_case** | エラー → 原因分析 → 修正パッチ → 検証結果の対 | 全 conductor + ai-conductor | 修正完了時 |
| **build_log** | ツール出力の要約 | fpga / asic / rtl / apps / hal | ビルド完了時 |
| **verification_result** | シミュレーション / 形式検証 / DRC / LVS / signoff の通過/失敗履歴 | テスト系サブエージェント | 検証完了時 |
| **decision_cot** | 重要な設計判断の chain-of-thought | 各 planner / designer サブエージェント | プランニング完了時 |
| **agent_action_log** | 各 agent-cli ワークスペースの AI_LOG | 全 agent-cli プロセス | exec_job 完了時 |
| **probe_result** | WatcherAgent / ProbeAgent / ValidatorAgent の検証ログ | ai-conductor | 検証完了時 |

### 知識検索の発火タイミング

| シナリオ | 発火元 | クエリ | 注入先 |
|---------|-------|-------|-------|
| 新規ビルド開始時 | ai-conductor task-router | `rag.search_similar.v1` | planner サブエージェントの context |
| エラー発生時 | 任意 conductor | `rag.search_bugfix.v1` | exec_job の reasoning context |
| 設計レビュー時 | designer サブエージェント | `rag.search_design.v1` | designer の判断材料 |
| パッチ生成時 | ai-conductor UpgradeManager | `rag.search_bugfix.v1` | パッチ生成プロンプト |

### rag offline 時の挙動

各 conductor は `.hestia/rag/queue/<peer>/` に作業ログをバッファ。rag 復旧（health-checker で `online` 検知）後、ai-conductor が一括 flush。

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [ai_conductor.md](ai_conductor.md) — ai-conductor 全体概要
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター
- [apps_conductor.md](apps_conductor.md) — アプリケーションソフトウェア開発オーケストレーター