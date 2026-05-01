# Conductor 共通設定セクション

**対象領域**: common — 設定
**ソース**: 設計仕様書 §3.8, §13.7.6, §20.2

## 概要

各 conductor の TOML 設定ファイル（`fpga.toml` / `asic.toml` / `pcb.toml` 等）には、共通するセクションが定義されている。本ドキュメントは共通セクションのスキーマと意味を規定する。

## 共通セクション一覧

### `[project]`

プロジェクト基本情報。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `name` | string | 必須 | プロジェクト名 |
| `version` | string | 必須 | プロジェクトバージョン |
| `description` | string | 任意 | プロジェクト概要 |

### `[adapters]`

使用するツールアダプターの宣言。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `active` | string[] | 必須 | 使用するアダプター名のリスト |
| `search_paths` | string[] | 任意 | adapter.toml 検索パス |

### `[build]`

ビルド設定。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `targets` | table[] | 必須 | ビルドターゲット定義 |
| `steps` | string[] | 任意 | ビルドステップ順序 |
| `timeout_secs` | int | 任意 | ビルドタイムアウト（既定 3600）|
| `max_parallel` | int | 任意 | 最大並列数（既定 4）|

### `[container]`

コンテナ実行設定（コンテナ実行を選択した場合のみ）。

| キー | 型 | 必須 | 説明 |
|------|----|------|------|
| `name` | string | 必須 | コンテナ名 |
| `base_image` | string | 必須 | ベースイメージ |
| `conductor` | string | 必須 | 対象 conductor |

### `[health]`

ヘルスチェック設定。

| キー | 型 | 既定 | 説明 |
|------|----|------|------|
| `cmd` | string | — | ローカル実行モードでの簡易確認コマンド |
| `interval_secs` | int | 30 | ポーリング間隔 |
| `timeout_secs` | int | 3 | 1 回の応答タイムアウト |
| `max_retries` | int | 3 | 失敗時の連続リトライ回数 |
| `escalate_on_fail` | bool | true | 連続失敗時にフロントエンドへ通知 |
| `restart_on_fail` | bool | true | 自動再起動試行 |

### `[agent_cli]`

agent-cli バックエンド設定（§20 参照）。

| キー | 型 | 既定 | 説明 |
|------|----|------|------|
| `backend` | string | `"claude"` | LLM バックエンド種別 |
| `binary_path` | string | `""` | agent-cli バイナリパス |
| `anthropic_base_url` | string | `""` | OpenAI 互換 API エンドポイント |
| `anthropic_api_key_env` | string | `"ANTHROPIC_API_KEY"` | API キー環境変数名 |
| `model` | string | `"claude-opus-4-7"` | LLM モデル名 |
| `max_tokens` | int | 4096 | 応答上限トークン数 |
| `registry_dir` | string | `""` | IPC レジストリディレクトリ |

### `[rag]`

RAG 設定（rag-conductor 用）。

| キー | 型 | 既定 | 説明 |
|------|----|------|------|
| `backend` | string | `"chroma"` | ベクトル DB バックエンド |
| `embedding_model` | string | `"nomic-embed-text"` | 埋め込みモデル |
| `top_k` | int | 5 | 検索上位件数 |
| `chunk_size` | int | 1000 | チャンクサイズ |
| `chunk_overlap` | int | 200 | チャンクオーバーラップ |
| `self_learning_enabled` | bool | true | 自己学習機能 ON/OFF |

## TOML パーサー

共通パーサーは `project-model` クレートに実装され、各 conductor が利用する。`serde` によるデシリアライズで `#[serde(default)]` を活用し、各キー個別に省略可能。

## 関連ドキュメント

- [configuration_management.md](configuration_management.md) — 設定ファイル管理（ホットリロード）
- [backend_switching.md](backend_switching.md) — LLM バックエンド切替
- [health_check_orchestration.md](health_check_orchestration.md) — ヘルスチェック