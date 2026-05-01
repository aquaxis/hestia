# ai-conductor CLI バイナリ仕様

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §3（745-1240行目付近）

## バイナリ名

`hestia-ai-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `exec` | agent-cli 上で自然言語指示を直接実行 |
| `run --file <path>` | ワークフロー YAML ファイルを実行 |
| `agent ls` | 登録済みサブエージェント一覧表示 |
| `container ls` | コンテナ一覧表示 |
| `container start <id>` | コンテナ起動 |
| `container stop <id>` | コンテナ停止 |
| `container create` | container.toml から Containerfile 生成・ビルド |
| `workflow run <yaml>` | DAG ベースワークフローを実行（§3.5 WorkflowEngine 経由） |
| `review start` | 仕様書レビューセッションを開始（§3.6 SpecDriven） |

## 共通オプション（CommonOpts）

| オプション | 値 | 説明 |
|-----------|---|------|
| `--output` | `human` \| `json` | 出力フォーマット（既定: human） |
| `--timeout` | `<秒>` | RPC タイムアウト |
| `--registry` | `<path>` | agent-cli レジストリパス（既定: `$XDG_RUNTIME_DIR/agent-cli/`） |
| `--config` | `<path>` | 設定ファイルパス |
| `--verbose` | — | 詳細ログ出力 |

## Exit Code

| Exit Code | 意味 |
|-----------|------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。対応する conductor の agent-cli peer（peer 名 `ai`）に対して agent-cli ネイティブ IPC（`agent-cli send <peer> <payload>`）で接続する。フロントエンドなしでもフルフロー実行可能。

## 使用例

```bash
# エージェント一覧
hestia-ai-cli agent ls

# コンテナ作成・起動
hestia-ai-cli container create
hestia-ai-cli container start vivado-build

# ワークフロー実行
hestia-ai-cli run --file workflow/fpga_to_asic.yaml

# 仕様書レビュー開始
hestia-ai-cli review start --spec spec/dsp_core.md
```

## 関連ドキュメント

- [ai/config_schema.md](config_schema.md) — container.toml / upgrade.toml 設定スキーマ
- [ai/message_methods.md](message_methods.md) — ai.* メソッド一覧
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine 詳細
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様
- [../frontend/cli_clients.md](../frontend/cli_clients.md) — CLI クライアント共通仕様