# hal-conductor 設定スキーマ

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §8.4（2218-...行目付近）

## hal.toml — 統一プロジェクトフォーマット

HAL プロジェクトの設定・レジスタ定義ソース・バスプロトコル・出力先を宣言的に定義するファイル。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[project]` | 必須 | プロジェクト基本設定（名前、入力フォーマット） |
| `[sources]` | 必須 | レジスタ定義ソース・メモリマップ |
| `[bus]` | 必須 | バスプロトコル・データ幅・アドレス幅 |
| `[outputs]` | 任意 | 各出力言語のファイルパス |

### `[project]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | プロジェクト名 |
| `input_format` | string | 入力フォーマット（`systemrdl` / `ipxact` / `toml`） |

### `[sources]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `register_definitions` | string[] | レジスタ定義ファイル（glob 対応、例: `regs/**/*.rdl`） |
| `memory_map` | string | メモリマップ設定ファイルパス |

### `[bus]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `protocol` | string | バスプロトコル（`axi4-lite` / `axi4` / `wishbone-b4` / `ahb-lite`） |
| `data_width` | integer | データ幅（ビット、例: 32） |
| `addr_width` | integer | アドレス幅（ビット、例: 32） |

### `[outputs]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `c_header` | string | C ヘッダ出力パス |
| `rust_crate` | string | Rust crate 出力パス |
| `python_module` | string | Python モジュール出力パス |
| `documentation` | string | Markdown ドキュメント出力パス |
| `svd` | string | SVD ファイル出力パス |

### 設定例

```toml
[project]
name = "soc_hal"
input_format = "systemrdl"

[sources]
register_definitions = ["regs/**/*.rdl"]
memory_map = "config/memory_map.toml"

[bus]
protocol = "axi4-lite"
data_width = 32
addr_width = 32

[outputs]
c_header = "build/hal/inc/soc_hal.h"
rust_crate = "build/hal/rust/soc-hal"
python_module = "build/hal/python/soc_hal.py"
documentation = "build/hal/docs/registers.md"
svd = "build/hal/svd/soc_hal.svd"
```

## 関連ドキュメント

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli バイナリ仕様
- [hal/register_map.md](register_map.md) — レジスタマップ定義
- [hal/codegen.md](codegen.md) — 多言語コード生成
- [../rtl/config_schema.md](../rtl/config_schema.md) — rtl.toml スキーマ