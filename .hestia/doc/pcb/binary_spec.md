# pcb-conductor CLI バイナリ仕様

**対象 Conductor**: pcb-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §7（1982-2174行目付近）

## バイナリ名

`hestia-pcb-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `init` | pcb.toml テンプレート生成 |
| `build` | PCB ビルドフロー全体実行（要件パース→BOM→回路図合成→検証→配置→配線→出力） |
| `ai-synthesize` | AI 駆動回路図合成実行（LLM コア） |
| `output kicad` | KiCad 形式出力 |
| `output gerber` | ガーバーファイル出力 |
| `output bom` | BOM（部品表）出力 |
| `drc` | DRC（デザインルールチェック）実行 |
| `erc` | ERC（電気ルールチェック）実行 |
| `status` | ビルド状態・ジョブ状況表示 |

## 共通オプション（CommonOpts）

| オプション | 値 | 説明 |
|-----------|---|------|
| `--output` | `human` \| `json` | 出力フォーマット（既定: human） |
| `--timeout` | `<秒>` | RPC タイムアウト |
| `--registry` | `<path>` | agent-cli レジストリパス |
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

## CLI 使用例

```bash
# 初期化
hestia pcb init

# AI 駆動回路図合成
hestia pcb ai-synthesize --spec "STM32F103 + BME280 温湿度センサボード"

# DRC / ERC 実行
hestia pcb drc
hestia pcb erc

# ガーバー出力
hestia pcb output gerber
```

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。pcb-conductor の agent-cli peer（peer 名 `pcb`）に対して agent-cli ネイティブ IPC で接続する。

## 関連ドキュメント

- [pcb/config_schema.md](config_schema.md) — pcb.toml 設定スキーマ
- [pcb/message_methods.md](message_methods.md) — pcb.* メソッド一覧
- [pcb/state_machines.md](state_machines.md) — PCB ビルドステップ
- [pcb/tool_adapter.md](tool_adapter.md) — AI 駆動回路図設計 / KiCad アダプター