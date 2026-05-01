# fpga-conductor CLI バイナリ仕様

**対象 Conductor**: fpga-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §5（1398-1760行目付近）

## バイナリ名

`hestia-fpga-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `init` | fpga.toml テンプレート生成 |
| `build <target>` | 指定ターゲットでフルビルド（合成→配置配線→bitstream） |
| `synthesize` | 合成のみ実行 |
| `implement` | 配置配線のみ実行 |
| `bitstream` | bitstream 生成のみ実行 |
| `simulate` | シミュレーション実行 |
| `program` | FPGA へ bitstream 書込 |
| `report timing` | タイミングレポート表示 |
| `report resource` | リソース使用率レポート表示 |
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
hestia fpga init

# フルビルド
hestia fpga build artix7

# 合成のみ
hestia fpga synthesize

# タイミングレポート
hestia fpga report timing --job-id 1

# bitstream 書込
hestia fpga program --target artix7_dev
```

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。fpga-conductor の agent-cli peer（peer 名 `fpga`）に対して agent-cli ネイティブ IPC で接続する。

## 関連ドキュメント

- [fpga/config_schema.md](config_schema.md) — fpga.toml 設定スキーマ
- [fpga/message_methods.md](message_methods.md) — fpga.* メソッド一覧
- [fpga/state_machines.md](state_machines.md) — ビルドステートマシン
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter トレイト