# 設定ファイル管理

**対象領域**: common — 設定管理
**ソース**: 設計仕様書 §18.9, §13.7.6, §19.3.5

## 概要

HESTIA では各種設定ファイル（`config.toml` / `fpga.toml` / `container.toml` / `sources.toml` 等）を `inotify` で監視し、変更検知時にホットリロードを行う。これにより、conductor の再起動なしで設定変更を反映できる。

## inotify 変更検知

### 監視対象

| パス | 監視内容 |
|------|---------|
| `.hestia/config.toml` | グローバル設定（agent_cli / health 等）|
| `.hestia/<conductor>/*.toml` | conductor 固有設定 |
| `.hestia/rag/sources.toml` | RAG ソース宣言 |
| `.hestia/rag/sources/` | RAG ソースファイル（PDF / Web）|

### 検知フロー

```
[inotify watch] → ファイル変更検知 → SHA-256 ハッシュ比較 → 差分あり →
  ├── 設定再読込（HestiaConfig::from_toml_file）
  ├── 変更差分を構造化ログに記録
  └── 影響を受ける conductor への設定反映通知
```

## ホットリロード

### リロード可能な設定項目

| 項目 | リロード可否 | 備考 |
|------|------------|------|
| `[health] interval_secs` | 可能 | 次回ヘルスチェック周期から反映 |
| `[rag] top_k / chunk_size` | 可能 | 次回インジェストから反映 |
| `[agent_cli] model / max_tokens` | 可能 | 次回 agent-cli 子プロセス起動から反映 |
| `[agent_cli] backend` | 要再起動 | バックエンド切替はプロセス再起動が必要 |
| `[build]` ターゲット変更 | 可能 | 実行中ジョブには影響なし |

### リロード不可の設定項目

- `[agent_cli] backend` — プロセス再起動が必要
- `[container]` — コンテナイメージの再ビルドが必要
- ポート番号・ソケットパス — プロセス再起動が必要

## cron / inotify スケジューリング

RAG ソースの自動更新は以下のトリガで発火:

1. **cron**: `0 3 * * *`（毎日 03:00 UTC、既定）
2. **ファイル変更**: `.hestia/rag/sources/` の `inotify` / `fswatch` 監視
3. **手動**: `hestia rag ingest --source-id <id>`

## 実装クレート

- `configuration_management` — inotify ラッパー（`inotify` クレート、LGPL）
- `project-model` — TOML パーサー・設定モデル（`serde` + `toml`）

## 関連ドキュメント

- [config_common.md](config_common.md) — 共通設定セクション
- [backend_switching.md](backend_switching.md) — LLM バックエンド切替
- [observability.md](observability.md) — 監視・ログ