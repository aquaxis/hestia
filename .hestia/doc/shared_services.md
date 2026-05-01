# 共有サービス層

**対象領域**: 共有サービス（Layer 5）/ rag-conductor
**ソース**: 設計仕様書 §13（3186-3491行目付近）

---

## 1. 概要

共有サービス層（Layer 5）は **6 種の横断サービス** を提供する。各サービスは agent-cli peer として起動され、全 conductor から利用可能である。

**表 HD-040: 共有サービス 6 種**

| サービス | クレート / バイナリ | ソケット | 一次仕様 |
|---------|---------------------|---------|---------|
| HDL LSP Broker | `hdl-lsp-broker` | agent-cli peer `lsp` | `common/hdl_lsp_broker.md` |
| WASM 波形ビューア | `waveform-core` (cdylib + rlib) | agent-cli peer `waveform`（ホスト経路）／WebWorker（WASM 経路）| `common/wasm_waveform_viewer.md` |
| Constraint Bridge | `constraint-bridge` | agent-cli peer `constraint-bridge` | `common/constraint_bridge.md` |
| IP Manager | `ip-manager` | agent-cli peer `ip-manager` | `common/ip_manager.md` |
| CI/CD API | `cicd-api` | agent-cli peer `cicd` | `common/cicd_api.md` |
| Observability | `observability` (Prometheus+tracing+OTLP) | agent-cli peer `observability` + HTTP `:9090/metrics` + OTLP `:4317` + Health `:8080` | `common/observability.md` |

---

## 2. HDL LSP Broker（§13.1）

Verilog / SystemVerilog / VHDL / Verilog-AMS の LSP サーバ群を統一インターフェースで提供する LSP プロキシ。フロントエンド（VSCode 拡張 / Tauri IDE）からは単一接続で複数言語の補完・診断・ジャンプ・参照・リネームを利用できる。

### 2.1 主要型

- `HdlLanguage`: `Verilog` / `SystemVerilog` / `Vhdl` / `VerilogAms`
- `LspServerConfig`: LSP サーバ設定
- `RoutingTable`: 言語→LSPサーバのルーティング

### 2.2 対応 LSP サーバ

| LSP サーバ | バージョン | 対応言語 |
|-----------|----------|---------|
| svls | v0.2.x | SystemVerilog |
| vhdl_ls | v0.3.x | VHDL |
| verilog-ams-ls | v0.1.x | Verilog-AMS |

### 2.3 拡張子マップ

| 拡張子 | 言語 |
|-------|------|
| `.v` | Verilog |
| `.sv` / `.svh` | SystemVerilog |
| `.vhd` / `.vhdl` | VHDL |
| `.va` / `.vams` | Verilog-AMS |

### 2.4 パラメータ既定値

- `max_instances=4`
- `idle_timeout_secs=300`

---

## 3. WASM 波形ビューア（§13.2）

VCD / FST / GHW / EVCD をストリーミングパース可能な波形ビューア。`waveform-core` クレートを `cdylib` + `rlib` でビルドし、ブラウザは WebWorker + SharedArrayBuffer 経由でロード、Tauri / VSCode WebView は同クレートを直接利用する。100 万サンプル表示時に 60fps を目標とする。

### 3.1 対応フォーマット

`WaveformFormat`: `Vcd` / `Fst` / `Ghw` / `Evcd`

### 3.2 信号モデル

- `Signal`: `id`, `full_name`, `display_name`, `bit_width`, `signal_type`（`Wire` / `Reg` / `Integer` / `Real`）, `scope`
- `SignalValue`: `Logic(char)` / `Vector{bits, hex}` / `Real(f64)` / `String`

### 3.3 パフォーマンス目標

100 万サンプル表示時に 60fps を目標とする。WebWorker + SharedArrayBuffer によりメインスレッドのブロックを回避する。

---

## 4. Constraint Bridge（§13.3）

制約ファイルの相互変換エンジン。`ConstraintModel` を中間表現とし、N 種類のフォーマット間で 2N 個のパーサ／ジェネレータで変換可能（旧 N×N → N+M 削減）。

### 4.1 対応フォーマット

| フォーマット | 対象 | 拡張子 |
|------------|------|-------|
| XDC | Xilinx | `.xdc` |
| PCF | iCE40 | `.pcf` |
| SDC | Synopsys | `.sdc` |
| Efinity XML | Efinix | XML |
| QSF | Intel Quartus | `.qsf` |
| UCF | 旧 ISE | `.ucf` |

`ConstraintFormat`: `Xdc` / `Pcf` / `Sdc`（その他は拡張型）

### 4.2 主要構造体

- `ClockConstraint`
- `PinConstraint`
- `TimingConstraint`
- `PlacementConstraint`
- `RawConstraint`

### 4.3 対応プロパティ

ピンアサイン・I/O 標準・ドライブ強度・スルーレート・差動ペアまで網羅する。

---

## 5. IP Manager（§13.4）

IP コアの登録・検索・バージョン解決・ライセンス管理・依存関係解決を提供する。`petgraph` の DAG ベース解決アルゴリズム（トポロジカルソート）で多段依存を解く。

### 5.1 IP コアデータモデル

- `IpCore`: `id`（`com.vendor.name`）/ `version`（semver）/ `vendor` / `library` / `device_families[]` / `supported_languages[]` / `dependencies[]` / `files[]` / `parameters[]`
- `IpDependency`: `ip_id` + `VersionReq` + `optional`
- `IpFile.type`: `rtl` / `testbench` / `doc` / `constraint`、`language`: `verilog` / `vhdl` / 他

### 5.2 依存関係解決

`petgraph` を用いた DAG ベースのトポロジカルソートにより、多段依存関係を自動的に解決する。

### 5.3 バージョン管理

semver（セマンティックバージョニング）に基づくバージョン要求（VersionReq）による解決を行う。

### 5.4 ライセンス分類

| 分類 | 内容 |
|------|------|
| `Oss` | MIT / Apache-2.0 / BSD / GPL / ISC / CC0 |
| `VendorProprietary` | FlexLM・seat 制限 |
| `Unknown` | 拒否 |

---

## 6. CI/CD API（§13.5）

CI/CD パイプラインを宣言的に定義し、複数バックエンド（GitHub Actions / GitLab CI / Local）で実行する。

### 6.1 バックエンド

`Backend`: `GithubActions` / `GitlabCi` / `LocalPipeline`

### 6.2 主要構造体

- `PipelineDefinition` / `PipelineStage` / `PipelineJob`
- `StageCondition`: `Always` / `OnSuccess` / `OnFailure` / `Custom`

### 6.3 制御機能

成果物（Artifact）リテンション・retry policy・timeout secs・cache key を JSON 経由で制御する。

---

## 7. Observability（§13.6）

### 7.1 メトリクス

- `prometheus` クレート、ポート `:9090/metrics`
- conductor／service 別 counter / gauge / histogram

### 7.2 ロギング

- `tracing` クレート、JSON 出力 `.hestia/logs/observability.log`

### 7.3 トレーシング

- OpenTelemetry SDK、OTLP/gRPC `:4317`

### 7.4 ヘルスチェック

- HTTP `:8080/{health, ready, live}`
- `HealthStatus`: `Healthy` / `Degraded` / `Unhealthy`

### 7.5 構成

`ConductorName`（`Ai` / `Fpga` / `Asic` / `Pcb` / `Debug` / `Rag`）毎にメトリクス／ヘルスを集約する。

---

## 8. rag-conductor — 知識基盤オーケストレーター（§13.7）

rag-conductor は **第 6 の Conductor** として、知識基盤の構築（Ingest）・管理・検索を独立プロセスで提供する。一次仕様は `.hestia/doc/rag_conductor.md` および `.hestia/doc/rag/*.md`。

> ai-conductor からの分離: 旧 `ai-conductor::rag-engine`（TypeScript + LangChain）と `rag-ingest`（Rust）は完全に rag-conductor に移行済み。ai-conductor からは agent-cli IPC の `rag` peer に対して `rag.*` 構造化メッセージで呼び出す。

### 8.1 構成と技術スタック

| 区分 | 技術 |
|------|------|
| バイナリ | `hestia-rag-conductor`（Rust + tokio） |
| ベクトル DB | Chroma（既定） / Qdrant |
| 埋め込み | Ollama `nomic-embed-text`（768 次元） |
| Rust 部分 | `rag-ingest` クレート（PDF 7 段 / Web 8 段パイプライン） |
| TS 部分 | `rag-engine`（Vector Search / Embedding / Citation Generation） |
| PDF 解析 | PyPDF / pdfplumber / Tesseract OCR（300 DPI、信頼度 >= 60%）/ Camelot（表抽出） |
| Web 取得 | trafilatura / BeautifulSoup / CLD3 / fasttext |

### 8.2 ナレッジベース構成

```
.hestia/rag/
├── sources/        # 取得元の生データ（PDF・HTML スナップショット）
├── chunks/         # チャンク分割済みテキスト
├── embeddings/     # ベクトル化済み（Chroma/Qdrant にインデックス）
├── index-metadata.toml
├── queries/        # クエリログ・ヒット率
└── quarantine/     # 品質ゲート不合格データ（保留）
```

### 8.3 取り込みパイプライン

- **PDF 7 段**: テキスト抽出 → OCR フォールバック → 表抽出 → 画像抽出 → セクション認識 → メタデータ付与 → 共通パイプラインへ
- **Web 8 段**: URL 列挙 → robots.txt 確認 → HTTP 取得 → 本文抽出 → ノイズ除去 → 言語検出 → メタデータ付与 → 共通パイプラインへ
- **共通 6 段**: 正規化 → 品質ゲート → チャンク分割（既定 1000 トークン / オーバーラップ 200）→ 埋め込み（Ollama）→ upsert（Chroma/Qdrant）→ ログ
- **品質ゲート 6 ルール**: 最小／最大文字数、言語検出、HTML ノイズ除去、重複（cosine >= 0.95）、UTF-8 妥当性、OCR 信頼度

### 8.4 増分更新と運用

- ETag / SHA-256 で変更検出 → 増分更新（180 分の全再構築 → 3 分相当に短縮）
- ライセンス管理: OSS / free 許可、`vendor-proprietary`（`terms_accepted=true` 必須）、`CC-BY-*`（クレジット表示）、`unknown` 拒否
- PII マスキング: 原本は GPG 暗号化保管、インデックスはマスク済みテキストのみ
- キャッシュ保持: PDF 無期限 / Web 90 日 / quarantine 30 日

### 8.5 RPC / CLI / メトリクス

- 主要 RPC: `rag.ingest`（source_type/file_path/url/source_id/all・force・incremental）、`rag.search`（query・top_k・filter・trace_id）、`rag.cleanup`、`rag.status`
- 自己学習 RPC: `rag.ingest_work.v1`、`rag.search_similar.v1`、`rag.search_bugfix.v1`、`rag.search_design.v1`
- `IngestJobStatus`: `Queued` / `Processing` / `Completed` / `Failed` / `PartiallyCompleted`
- TypeScript I/F: `RagQuery { text, top_k, filter, trace_id }` / `RagResult { chunks, citations, embedding_time_ms, retrieval_time_ms }`
- MCP ツール: `hestia_rag_search`
- CLI: `hestia rag ingest|search|cleanup`
- Prometheus メトリクス: `ingest_duration`, `docs_total`, `chunks_total`, `quarantine_total`, `incremental_skipped`, `license_violations`, `cache_size`, `retrieval_seconds`, `hit_ratio`

### 8.6 サブエージェント構成

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `rag-planner` | 取り込みプランニング（クロール戦略、ソース優先度、増分更新スケジュール）| 1 |
| **designer** | `rag-designer` | 知識ベース仕様（チャンク戦略、メタデータスキーマ、埋め込みモデル選定、retention ポリシー）| 1 |
| **ingest** | `rag-ingest-{source}` | ドキュメント取り込み（PDF 7 段 / Web 8 段パイプライン）| **N**（ソース数だけ並列起動）|
| **search** | `rag-search` | ベクトル検索 + reranking | 1（高負荷時 N）|
| **quality_gate** | `rag-quality` | 品質チェック（PII 検出 / ライセンス判定 / 重複排除 / quarantine 管理）| 1 |
| **archivist** | `rag-archivist` | 自己学習用 conductor-work-logs/ への蓄積パイプライン管理 | 1（高負荷時 N）|

---

## 関連ドキュメント

- [アーキテクチャ概要](architecture_overview.md) — 全体アーキテクチャにおける共有サービス層の位置づけ
- [セキュリティ](security.md) — API キー保護・ライセンス管理
- [コンテナ実行](container_execution.md) — コンテナビルドの Observability 連携
- [Hestia Flow](hestia_flow.md) — RAG の概念（§1.3.9）
- `.hestia/doc/common/observability.md` — Observability 詳細仕様
- `.hestia/doc/common/hdl_lsp_broker.md` — HDL LSP Broker 詳細仕様
- `.hestia/doc/common/constraint_bridge.md` — Constraint Bridge 詳細仕様
- `.hestia/doc/common/ip_manager.md` — IP Manager 詳細仕様
- `.hestia/doc/common/cicd_api.md` — CI/CD API 詳細仕様
- `.hestia/doc/common/wasm_waveform_viewer.md` — WASM 波形ビューア詳細仕様