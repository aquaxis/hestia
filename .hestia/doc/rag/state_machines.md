# rag-conductor インデックス状態遷移

**対象 Conductor**: rag-conductor
**ソース**: 設計仕様書 §13.7（3252-3491行目付近）

## インデックス状態遷移

取り込み（Ingest）ジョブのライフサイクルを管理するステートマシン。

```
Queued → Processing → Completed
                  ├── Failed
                  └── PartiallyCompleted
```

### IngestJobStatus

| 状態 | 説明 |
|------|------|
| Queued | キュー待ち（取り込み要求を受領、リソース待ち） |
| Processing | 処理中（PDF/Web パイプライン実行中） |
| Completed | 全ソースの取り込み完了 |
| Failed | 取り込み失敗（致命的エラー） |
| PartiallyCompleted | 一部完了（一部ソースが失敗、他は成功） |

## ソース別パイプライン状態

### PDF 取り込み状態遷移

```
テキスト抽出 → OCR フォールバック → 表抽出 → 画像抽出
    → セクション認識 → メタデータ付与 → 共通パイプライン
```

### Web 取り込み状態遷移

```
URL 列挙 → robots.txt 確認 → HTTP 取得 → 本文抽出
    → ノイズ除去 → 言語検出 → メタデータ付与 → 共通パイプライン
```

### 共通パイプライン状態遷移

```
正規化 → 品質ゲート → チャンク分割 → 埋め込み → upsert → ログ
```

## 品質ゲート判定

品質ゲートで不合格となったデータは `quarantine/` に保留される。

| ルール | 条件 | アクション |
|-------|------|----------|
| 最小文字数 | チャンクが短すぎる | quarantine |
| 最大文字数 | チャンクが長すぎる | 分割再試行 |
| 言語検出 | 対応言語以外 | quarantine |
| HTML ノイズ除去 | ノイズ残存 | 再処理 |
| 重複（cosine >= 0.95） | 既存チャンクと高類似 | スキップ |
| UTF-8 妥当性 | 不正エンコーディング | quarantine |
| OCR 信頼度 | < 60% | quarantine |

## 増分更新フロー

```
変更検出（ETag / SHA-256）
    │
    ├── 変更なし → スキップ（incremental_skipped メトリクス更新）
    │
    └── 変更あり → 該当ソースのみ再取り込み
```

## ライセンス判定フロー

```
ソース取り込み要求
    │
    ├── OSS / free → 取り込み許可
    ├── CC-BY-* → クレジット表示付きで取り込み
    ├── vendor-proprietary → terms_accepted=true 必須
    └── unknown → 拒否（license_violations メトリクス更新）
```

## PII マスキングフロー

```
原文（PII 含む可能性）
    │
    ├── PII 検出 → マスキング適用
    │
    ├── 原本 → GPG 暗号化保管
    │
    └── インデックス → マスク済みテキストのみ
```

## キャッシュ保持期間

| ソース種別 | 保持期間 |
|-----------|---------|
| PDF | 無期限 |
| Web | 90 日 |
| quarantine | 30 日 |
| conductor-work-logs (design_case/bugfix_case) | 無期限 |
| conductor-work-logs (build_log 等) | 365 日 |

## 関連ドキュメント

- [rag/ingest_pipeline.md](ingest_pipeline.md) — 取り込みパイプライン詳細
- [rag/search_engine.md](search_engine.md) — 検索エンジン仕様
- [rag/config_schema.md](config_schema.md) — config.toml [rag] スキーマ
- [rag/error_types.md](error_types.md) — rag-conductor エラーコード