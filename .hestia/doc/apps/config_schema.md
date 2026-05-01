# apps-conductor 設定スキーマ

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9.4（2325-2354行目付近）

## apps.toml — 統一プロジェクトフォーマット

アプリケーションファームウェアプロジェクトの設定・ツールチェーン・RTOS・メモリレイアウト・HAL 連携・テスト設定を宣言的に定義するファイル。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[project]` | 必須 | プロジェクト基本設定 |
| `[toolchain]` | 必須 | コンパイラ・バージョン指定 |
| `[rtos]` | 任意 | RTOS カーネル・バージョン |
| `[memory]` | 必須 | メモリレイアウト（Flash / RAM） |
| `[hal]` | 任意 | HAL モジュールの import |
| `[test]` | 任意 | テストモード・プローブ設定 |

### `[project]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | プロジェクト名 |
| `language` | string | 言語（`c` / `cpp` / `rust`） |
| `target` | string | ターゲットトリプル（例: `thumbv7em-none-eabihf`） |

### `[toolchain]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `compiler` | string | コンパイラ（`arm-none-eabi-gcc` / `riscv32-unknown-elf-gcc` / `cargo`） |
| `version` | string | バージョン |

### `[rtos]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `kernel` | string | RTOS カーネル（`freertos` / `zephyr` / `embassy-rs` / `bare-metal`） |
| `version` | string | RTOS バージョン |

### `[memory]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `flash_origin` | integer | Flash 開始アドレス |
| `flash_length` | string | Flash サイズ（例: `256K`） |
| `ram_origin` | integer | RAM 開始アドレス |
| `ram_length` | string | RAM サイズ（例: `64K`） |
| `linker_script` | string | リンカスクリプトパス |

### `[hal]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `import` | string | hal-conductor（§8）の生成物 import パス |

### `[test]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `mode` | string | テストモード（`sil` / `hil` / `qemu`） |
| `probe` | string | デバッグプローブ（`stlink-v3` 等、debug-conductor §10 経由） |

### 設定例

```toml
[project]
name = "sensor_node_fw"
language = "rust"
target = "thumbv7em-none-eabihf"

[toolchain]
compiler = "arm-none-eabi-gcc"
version = "14.2.1"

[rtos]
kernel = "embassy-rs"
version = "0.4"

[memory]
flash_origin  = 0x08000000
flash_length  = "256K"
ram_origin    = 0x20000000
ram_length    = "64K"
linker_script = "memory.x"

[hal]
import = "build/hal/rust/soc-hal"

[test]
mode  = "hil"
probe = "stlink-v3"
```

## 関連ドキュメント

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli バイナリ仕様
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [apps/rtos.md](rtos.md) — RTOS サポート
- [../hal/config_schema.md](../hal/config_schema.md) — hal.toml スキーマ