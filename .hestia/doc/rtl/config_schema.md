# rtl-conductor 設定スキーマ

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §4.4（1288-1310行目付近）

## rtl.toml — 統一プロジェクトフォーマット

RTL プロジェクトの設定・ソース定義・アダプター選択・ハンドオフ先を宣言的に定義するファイル。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[project]` | 必須 | プロジェクト基本設定（名前、トップモジュール、言語） |
| `[sources]` | 必須 | ソースファイル定義（RTL / テストベンチ / 共有制約） |
| `[adapters]` | 任意 | 各機能のアダプター選択 |
| `[handoff]` | 任意 | 下流 conductor へのハンドオフ成果物 |

### `[project]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | プロジェクト名 |
| `top` | string | トップモジュール名 |
| `language` | string | HDL 言語（`systemverilog` / `vhdl` / `chisel` / `spinalhdl` / `amaranth`） |

### `[sources]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `rtl` | string[] | RTL ソースファイル（glob 対応） |
| `testbench` | string[] | テストベンチファイル |
| `constraints_shared` | string[] | 共通制約ファイル（SDC 等） |

### `[adapters]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `lint` | string | Lint アダプター名 |
| `simulation` | string | シミュレーションアダプター名 |
| `formal` | string | 形式検証アダプター名 |

### `[handoff]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `fpga` | string[] | fpga-conductor（§5）に渡す成果物 |
| `asic` | string[] | asic-conductor（§6）に渡す成果物 |
| `hal_bus_decl` | string | hal-conductor（§8）のバス定義入力 |

### 設定例

```toml
[project]
name = "core_v"
top = "Cv32e40p"
language = "systemverilog"

[sources]
rtl = ["src/**/*.sv"]
testbench = ["tb/**/*.sv"]
constraints_shared = ["constraints/timing_shared.sdc"]

[adapters]
lint = "verilator-lint"
simulation = "verilator"
formal = "symbiyosys"

[handoff]
fpga = ["build/synth_ready.sv"]
asic = ["build/asic_ready.sv"]
hal_bus_decl = "build/bus_iface.rdl"
```

## 関連ドキュメント

- [rtl/binary_spec.md](binary_spec.md) — hestia-rtl-cli バイナリ仕様
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter トレイト
- [rtl/handoff.md](handoff.md) — 下流連携ハンドオフ
- [../fpga/config_schema.md](../fpga/config_schema.md) — fpga.toml スキーマ