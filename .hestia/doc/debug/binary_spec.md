# debug-conductor CLI バイナリ仕様

**対象 Conductor**: debug-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §10（2401-2550行目付近）

## バイナリ名

`hestia-debug-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `create` | デバッグセッション作成 |
| `connect` | ターゲットデバイスへ接続（JTAG / SWD） |
| `disconnect` | ターゲットデバイスから切断 |
| `program` | ファームウェア書込（SVF / JAM / probe-rs / OpenOCD） |
| `capture start` | 波形キャプチャ開始 |
| `capture stop` | 波形キャプチャ停止 |
| `signals read` | 信号読み取り |
| `trigger set` | トリガ条件設定 |
| `reset` | ターゲットリセット（Hardware / Software / System） |
| `status` | セッション状態・接続状況表示 |

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

## ローカル専用

debug-conductor は **ローカル専用**（USB プローブアクセス、§2.2）である。コンテナ内での実行は USB デバイスアクセスの制約により対応しない。

## CLI 使用例

```bash
# セッション作成・接続
hestia debug create
hestia debug connect --probe stlink-v3

# ファームウェア書込
hestia debug program --firmware build/sensor_node_fw.bin

# 波形キャプチャ
hestia debug capture start --signals "clk,data"
hestia debug capture stop

# リセット
hestia debug reset --type hardware
```

## 関連ドキュメント

- [debug/config_schema.md](config_schema.md) — debug-conductor 設定
- [debug/message_methods.md](message_methods.md) — debug.* メソッド一覧
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD プロトコル
- [debug/state_machines.md](state_machines.md) — セッション管理ステートマシン