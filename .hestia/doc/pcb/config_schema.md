# pcb-conductor 設定スキーマ

**対象 Conductor**: pcb-conductor
**ソース**: 設計仕様書 §7.6（2157-...行目付近）

## pcb.toml — 統一プロジェクトフォーマット

PCB プロジェクトの設定・ボード定義・層構成・AI 設計設定・出力設定を宣言的に定義するファイル。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[project]` | 必須 | プロジェクト基本設定 |
| `[board]` | 必須 | ボード寸法・層数 |
| `[[layers]]` | 必須 | 各層の定義（信号/電源/GND） |
| `[design]` | 任意 | AI 設計設定 |
| `[output]` | 任意 | 出力設定 |

### `[project]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | プロジェクト名 |
| `version` | string | バージョン |
| `board_name` | string | ボード名 |

### `[board]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `layer_count` | integer | 層数 |
| `width_mm` | float | ボード幅（mm） |
| `height_mm` | float | ボード高さ（mm） |

### `[[layers]]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | 層名（例: `F.Cu`, `In1.Cu`, `B.Cu`） |
| `type` | string | 層種別（`signal` / `power` / `ground`） |

### `[design]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `input_format` | string | 入力フォーマット（`natural_language` / `skidl` / `kicad_sch`） |
| `ai_enabled` | boolean | AI 駆動回路図設計の有効化 |

### `[output]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `format` | string | 出力フォーマット（`kicad` / `altium` / `gerber`） |
| `output_dir` | string | 出力ディレクトリ |

### 設定例

```toml
[project]
name = "my-pcb-project"
version = "0.1.0"
board_name = "motor_controller"

[board]
layer_count = 4
width_mm = 100
height_mm = 80

[[layers]]
name = "F.Cu"
type = "signal"

[[layers]]
name = "In1.Cu"
type = "power"

[[layers]]
name = "In2.Cu"
type = "ground"

[[layers]]
name = "B.Cu"
type = "signal"

[design]
input_format = "natural_language"
ai_enabled = true

[output]
format = "kicad"
output_dir = "output/"
```

## 関連ドキュメント

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli バイナリ仕様
- [pcb/state_machines.md](state_machines.md) — PCB ビルドステップ
- [pcb/tool_adapter.md](tool_adapter.md) — AI 駆動回路図設計 / KiCad アダプター
- [../fpga/config_schema.md](../fpga/config_schema.md) — fpga.toml スキーマ