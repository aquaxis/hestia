# rtl-conductor CLI バイナリ仕様

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §4（1241-1397行目付近）

## バイナリ名

`hestia-rtl-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `init` | rtl.toml テンプレート生成 |
| `lint` | HDL ソースの Lint / フォーマット / 静的解析（Verilator/Verible 等） |
| `simulate` | シミュレーション実行（`--tb <testbench>` / `--simulator verilator`） |
| `formal` | 形式検証実行（SymbiYosys、`--properties <file>`） |
| `transpile` | HDL 言語間トランスパイル（Chisel/SpinalHDL/Amaranth → Verilog） |
| `handoff` | 下流 conductor へのハンドオフ（`--target fpga` / `--target asic`） |
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
hestia rtl init

# Lint 実行
hestia rtl lint

# シミュレーション
hestia rtl simulate --tb tb_alu --simulator verilator

# 形式検証
hestia rtl formal --properties properties.sv

# 下流ハンドオフ
hestia rtl handoff --target fpga
```

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。rtl-conductor の agent-cli peer（peer 名 `rtl`）に対して agent-cli ネイティブ IPC で接続する。

## 関連ドキュメント

- [rtl/config_schema.md](config_schema.md) — rtl.toml 設定スキーマ
- [rtl/message_methods.md](message_methods.md) — rtl.* メソッド一覧
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter トレイト
- [rtl/handoff.md](handoff.md) — 下流連携ハンドオフ
- [../ai/binary_spec.md](../ai/binary_spec.md) — 統合 CLI 仕様