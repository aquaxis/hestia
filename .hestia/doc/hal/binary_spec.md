# hal-conductor CLI バイナリ仕様

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §8（2175-2280行目付近）

## バイナリ名

`hestia-hal-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `init` | hal.toml テンプレート生成 |
| `parse` | レジスタ定義ファイルのパース（SystemRDL / IP-XACT / TOML） |
| `validate` | レジスタマップのバリデーション（アドレス重複・型整合性・バス境界チェック） |
| `generate c` | C ヘッダファイル生成 |
| `generate rust` | Rust crate 生成（embedded-hal 互換） |
| `generate python` | Python モジュール生成 |
| `generate svd` | CMSIS SVD ファイル生成 |
| `export-rtl` | SystemVerilog テンプレート出力（rtl/asic/fpga conductor 向け） |
| `diff` | レジスタマップ差分表示 |
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
hestia hal init

# レジスタ定義パース
hestia hal parse

# バリデーション
hestia hal validate

# C ヘッダ生成
hestia hal generate c

# Rust crate 生成
hestia hal generate rust

# 差分表示
hestia hal diff --baseline v1.0 --current v1.1
```

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。hal-conductor の agent-cli peer（peer 名 `hal`）に対して agent-cli ネイティブ IPC で接続する。

## 関連ドキュメント

- [hal/config_schema.md](config_schema.md) — hal.toml 設定スキーマ
- [hal/message_methods.md](message_methods.md) — hal.* メソッド一覧
- [hal/register_map.md](register_map.md) — レジスタマップ定義
- [hal/codegen.md](codegen.md) — 多言語コード生成