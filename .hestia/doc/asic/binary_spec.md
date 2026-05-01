# asic-conductor CLI バイナリ仕様

**対象 Conductor**: asic-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §6（1761-1981行目付近）

## バイナリ名

`hestia-asic-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `init` | asic.toml テンプレート生成 |
| `build` | RTL-to-GDSII フルビルド実行 |
| `pdk install <pdk>` | PDK インストール（Sky130 / GF180MCU / IHP SG13G2） |
| `pdk list` | インストール済み PDK 一覧表示 |
| `advance` | ビルドを次ステップに進める（OpenLane 2 Step-based Execution と連携） |
| `drc` | DRC（デザインルールチェック）実行 |
| `lvs` | LVS（レイアウト対回路図検証）実行 |
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
hestia asic init

# PDK インストール
hestia asic pdk install sky130_fd_sc_hd

# フルビルド
hestia asic build

# DRC のみ実行
hestia asic drc

# 特定ステップから再開
hestia asic advance --from placement
```

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。asic-conductor の agent-cli peer（peer 名 `asic`）に対して agent-cli ネイティブ IPC で接続する。OpenLane 2 は Podman コンテナ内で実行される。

## 関連ドキュメント

- [asic/config_schema.md](config_schema.md) — asic.toml 設定スキーマ
- [asic/message_methods.md](message_methods.md) — asic.* メソッド一覧
- [asic/state_machines.md](state_machines.md) — ASIC ビルドステートマシン
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter トレイト