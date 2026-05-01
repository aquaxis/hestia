# asic-conductor 設定スキーマ

**対象 Conductor**: asic-conductor
**ソース**: 設計仕様書 §6.9（1930-1958行目付近）

## asic.toml — 統一プロジェクトフォーマット

ASIC プロジェクトの設定・PDK 指定・合成設定・配置配線設定を宣言的に定義するファイル。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[project]` | 必須 | プロジェクト基本設定 |
| `[target]` | 必須 | PDK・クロック周期の指定 |
| `[synthesis]` | 任意 | 論理合成設定 |
| `[placement]` | 任意 | 配置設定 |
| `[cts]` | 任意 | クロックツリー合成設定 |
| `[routing]` | 任意 | 配線設定 |

### `[project]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | プロジェクト名 |
| `version` | string | バージョン |
| `rtl_files` | string[] | RTL ソースファイル（glob 対応） |
| `top` | string | トップモジュール名 |

### `[target]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `pdk` | string | PDK 名（例: `sky130_fd_sc_hd` / `gf180mcu_fd_sc_mcu7t5v0` / `ihp_sg13g2`） |
| `clock_period_ns` | float | クロック周期（ナノ秒） |

### `[synthesis]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `flatten` | boolean | フラット化有効化 |
| `abc_script` | string | ABC テクノロジマッピングスクリプト（例: `resyn2`） |
| `strategy` | string | 合成戦略（`area` / `speed` / `balanced`） |

### `[placement]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `target_density` | float | ターゲット配置密度（0.0〜1.0） |

### `[cts]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `max_skew_ns` | float | 最大クロックスキュー（ナノ秒） |

### `[routing]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `min_layer` | string | 最小配線層（例: `met1`） |
| `max_layer` | string | 最大配線層（例: `met5`） |

### 設定例

```toml
[project]
name = "my-asic-project"
version = "0.1.0"
rtl_files = ["src/*.v"]
top = "top_module"

[target]
pdk = "sky130_fd_sc_hd"
clock_period_ns = 10.0

[synthesis]
flatten = true
abc_script = "resyn2"
strategy = "area"

[placement]
target_density = 0.6

[cts]
max_skew_ns = 0.5

[routing]
min_layer = "met1"
max_layer = "met5"
```

## サポート PDK

| PDK | プロセス | 提供元 | 用途 |
|-----|---------|--------|------|
| Sky130 | 130nm CMOS | SkyWater Technology | デジタル・混合信号、最も安定 |
| GF180MCU | 180nm CMOS | GlobalFoundries | MCU 向け、高信頼性 |
| IHP SG13G2 | 130nm BiCMOS | IHP | 高速アナログ・高周波設計 |

## 関連ドキュメント

- [asic/binary_spec.md](binary_spec.md) — hestia-asic-cli バイナリ仕様
- [asic/state_machines.md](state_machines.md) — ASIC ビルドステートマシン
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter トレイト
- [../rtl/config_schema.md](../rtl/config_schema.md) — rtl.toml スキーマ