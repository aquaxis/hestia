# apps-conductor CLI バイナリ仕様

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §15（3631-3730行目付近）, §9（2281-2400行目付近）

## バイナリ名

`hestia-apps-cli`

## サブコマンド一覧

| サブコマンド | 説明 |
|-------------|------|
| `init` | apps.toml テンプレート生成 |
| `build` | クロスコンパイル・ビルド実行 |
| `flash` | ターゲットデバイスへのファームウェア書込 |
| `test sil` | SIL（Software-in-the-Loop）テスト実行（QEMU） |
| `test hil` | HIL（Hardware-in-the-Loop）テスト実行（実機 + debug-conductor） |
| `test qemu` | QEMU テスト実行 |
| `size` | バイナリサイズレポート表示 |
| `debug` | デバッグセッション開始（debug-conductor §10 との bridging） |
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
hestia apps init

# ビルド
hestia apps build

# QEMU テスト
hestia apps test qemu

# HIL テスト
hestia apps test hil

# フラッシュ書込
hestia apps flash

# サイズレポート
hestia apps size
```

## CLI アーキテクチャ

Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）。apps-conductor の agent-cli peer（peer 名 `apps`）に対して agent-cli ネイティブ IPC で接続する。

## 関連ドキュメント

- [apps/config_schema.md](config_schema.md) — apps.toml 設定スキーマ
- [apps/message_methods.md](message_methods.md) — apps.* メソッド一覧
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [apps/rtos.md](rtos.md) — RTOS サポート
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL テスト