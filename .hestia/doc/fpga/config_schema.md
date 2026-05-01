# fpga-conductor 設定スキーマ

**対象 Conductor**: fpga-conductor
**ソース**: 設計仕様書 §5.4（1542-1683行目付近）

## fpga.toml — 統一プロジェクトフォーマット

FPGA プロジェクトの設定・ターゲット定義・ツールチェーン制約・IP 管理・ビルド設定を宣言的に定義するファイル。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[project]` | 必須 | プロジェクト基本設定 |
| `[targets.*]` | 必須 | ターゲットデバイス定義（複数定義可能） |
| `[toolchain]` | 任意 | ツールチェーンバージョン制約（semver） |
| `[toolchain.lock]` | 任意 | ツールチェーンロック（再現性保証） |
| `[ip.*]` | 任意 | IP コア管理 |
| `[build]` | 任意 | ビルド設定 |
| `[sim]` | 任意 | シミュレーション設定 |

### `[project]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | プロジェクト名 |
| `version` | string | バージョン |
| `hdl_files` | string[] | HDL ソースファイル |
| `include_dirs` | string[] | インクルードディレクトリ |
| `testbenches` | string[] | テストベンチファイル |

### `[targets.*]` セクション

ターゲットごとに個別セクションを定義。

| フィールド | 型 | 説明 |
|-----------|---|------|
| `vendor` | string | ベンダー名（`xilinx` / `intel` / `efinix` / `yosyshq`） |
| `device` | string | デバイス名（例: `xc7a35tcsg324-1`） |
| `top` | string | トップモジュール名 |
| `constraints` | string[] | 制約ファイル（XDC / SDC / PCF / peri.xml） |
| `interface_script` | string | Efinity 用インターフェーススクリプト（任意） |

### `[toolchain]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `vivado` | string | Vivado バージョン制約（semver 例: `>=2023.1, <2026`） |
| `quartus` | string | Quartus バージョン制約 |
| `efinity` | string | Efinity バージョン制約 |

### `[toolchain.lock]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `vivado` | string | 固定バージョン（例: `2025.2.0`） |
| `quartus` | string | 固定バージョン |
| `efinity` | string | 固定バージョン |

### `[ip.*]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `vendor` | string | IP ベンダー |
| `name` | string | IP コア名 |
| `version` | string | IP バージョン |
| `config` | string | 設定ファイルパス（.xci 等） |

### `[build]` セクション

| フィールド | 型 | 既定値 | 説明 |
|-----------|---|-------|------|
| `parallel_jobs` | integer | — | 並列ジョブ数 |
| `incremental_compile` | boolean | — | インクリメンタルコンパイル有効化 |
| `cache_dir` | string | `.fpga-cache` | キャッシュディレクトリ |

### `[sim]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `tool` | string | シミュレーションツール（`iverilog` / `modelsim` / `questa` / `xsim`） |
| `top_tb` | string | トップテストベンチ名 |
| `plusargs` | string[] | プラス引数 |

### 設定例

```toml
[project]
name    = "my_dsp_core"
version = "0.2.0"
hdl_files   = ["hdl/top.sv", "hdl/fir_filter.sv", "hdl/bram_ctrl.sv"]
include_dirs = ["hdl/include"]
testbenches = ["sim/tb_top.sv", "sim/tb_fir.sv"]

[targets.artix7_dev]
vendor      = "xilinx"
device      = "xc7a35tcsg324-1"
top         = "top"
constraints = ["constraints/artix7.xdc"]

[targets.cyclone10]
vendor      = "intel"
device      = "10CL025YU256C8G"
top         = "top"
constraints = ["constraints/cyclone10.sdc"]

[targets.trion_t20]
vendor            = "efinix"
device            = "T20F256"
top               = "top"
interface_script  = "constraints/trion_t20.peri.xml"

[targets.ice40]
vendor      = "yosyshq"
device      = "iCE40HX8K"
top         = "top"
constraints = ["constraints/ice40.pcf"]

[toolchain]
vivado   = ">=2023.1, <2026"
quartus  = "~23.1"
efinity  = "*"

[toolchain.lock]
vivado   = "2025.2.0"
quartus  = "23.1.1"
efinity  = "2025.2.0"

[ip.fifo_gen]
vendor  = "xilinx"
name    = "fifo_generator"
version = "13.2"
config  = "ip/fifo_gen.xci"

[build]
parallel_jobs       = 8
incremental_compile = true
cache_dir           = ".fpga-cache"

[sim]
tool    = "iverilog"
top_tb  = "tb_top"
plusargs = ["+DUMP_WAVES=1"]
```

## 関連ドキュメント

- [fpga/binary_spec.md](binary_spec.md) — hestia-fpga-cli バイナリ仕様
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter トレイト
- [fpga/state_machines.md](state_machines.md) — ビルドステートマシン
- [../rtl/config_schema.md](../rtl/config_schema.md) — rtl.toml スキーマ