# rag-conductor 取り込みパイプライン

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §13.7.3（3282-3287行目付近）, §13.7.4（3289-3294行目付近）

## パイプライン構成

### PDF 7 段パイプライン

| 段階 | 処理 | 使用ツール |
|------|------|----------|
| 1 | テキスト抽出 | PyPDF / pdfplumber |
| 2 | OCR フォールバック | Tesseract OCR（300 DPI、信頼度 >= 60%） |
| 3 | 表抽出 | Camelot |
| 4 | 画像抽出 | PyPDF / pdfplumber |
| 5 | セクション認識 | ヘッダー・見出し検出 |
| 6 | メタデータ付与 | ソース・ページ番号・作成日等 |
| 7 | 共通パイプラインへ | — |

### Web 8 段パイプライン

| 段階 | 処理 | 使用ツール |
|------|------|----------|
| 1 | URL 列挙 | サイトマップ / クロール |
| 2 | robots.txt 確認 | robots.txt パーサー |
| 3 | HTTP 取得 | HTTP クライアント |
| 4 | 本文抽出 | trafilatura |
| 5 | ノイズ除去 | BeautifulSoup |
| 6 | 言語検出 | CLD3 / fasttext |
| 7 | メタデータ付与 | URL・タイトル・日付等 |
| 8 | 共通パイプラインへ | — |

### 共通 6 段パイプライン

| 段階 | 処理 | 説明 |
|------|------|------|
| 1 | 正規化 | Unicode 正規化・空白統一・HTML エンティティ展開 |
| 2 | 品質ゲート | 6 ルールによる品質チェック（不合格 → quarantine） |
| 3 | チャンク分割 | 既定 1000 トークン / オーバーラップ 200 |
| 4 | 埋め込み | Ollama `nomic-embed-text`（768 次元） |
| 5 | upsert | Chroma / Qdrant へのベクトル登録 |
| 6 | ログ | 取り込み結果ログ記録 |

## 品質ゲート 6 ルール

| ルール | 条件 | 合格時 | 不合格時 |
|-------|------|--------|---------|
| 最小文字数 | チャンク >= 最小閾値 | 次段階へ | quarantine |
| 最大文字数 | チャンク <= 最大閾値 | 次段階へ | 分割再試行 |
| 言語検出 | 対応言語である | 次段階へ | quarantine |
| HTML ノイズ除去 | ノイズなし | 次段階へ | 再処理 |
| 重複（cosine >= 0.95） | 既存チャンクと非類似 | 次段階へ | スキップ |
| UTF-8 妁当性 | 正しいエンコーディング | 次段階へ | quarantine |
| OCR 信頼度 | >= 60% | 次段階へ | quarantine |

## 増分更新

ETag / SHA-256 で変更を検出し、変更のあったソースのみ再取り込みする。全再構築が 180 分かかる処理を、増分更新により約 3 分に短縮する。

## ライセンス管理

| ライセンス種別 | 取り込み可否 | 条件 |
|-------------|------------|------|
| OSS / free | 許可 | 無条件 |
| CC-BY-* | 許可 | クレジット表示必須 |
| vendor-proprietary | 条件付き許可 | `terms_accepted=true` 必須 |
| unknown | 拒否 | — |

## PII マスキング

- 原本: GPG 暗号化で保管
- インデックス: マスク済みテキストのみ（PII は `[REDACTED]` 等で置換）
- マスキング対象: 氏名・メールアドレス・電話番号・IP アドレス等

## self_learning 取り込み（§13.7.8）

他 conductor からの作業内容自動蓄積パイプライン。

| カテゴリ | 内容 | 送信タイミング |
|---------|------|------------|
| design_case | 成功した設計パラメータ + ビルド結果 | ビルド成功時 |
| bugfix_case | エラー → 修正パッチ → 検証結果 | 修正完了時 |
| build_log | ツール出力要約 | ビルド完了時 |
| verification_result | 検証通過/失敗履歴 | 検証完了時 |
| decision_cot | 設計判断の CoT | プランニング/設計完了時 |
| agent_action_log | 作業ログ | exec_job 完了時 |
| probe_result | 互換性プローブ結果 | 検証完了時 |

## 関連ドキュメント

- [rag/config_schema.md](config_schema.md) — config.toml [rag] スキーマ
- [rag/search_engine.md](search_engine.md) — 検索エンジン仕様
- [rag/state_machines.md](state_machines.md) — インデックス状態遷移
- [rag/error_types.md](error_types.md) — rag-conductor エラーコード