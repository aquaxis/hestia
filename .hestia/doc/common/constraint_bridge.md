# Constraint Bridge

**対象領域**: common — 制約変換
**ソース**: 設計仕様書 §13.3

## 概要

制約ファイルの相互変換エンジン。`ConstraintModel` を中間表現とし、N 種類のフォーマット間で変換可能。従来の N x N 変換から N + M 個のパーサ/ジェネレータに削減する。

## 対応フォーマット

| フォーマット | 対象 | 拡張子 |
|------------|------|--------|
| XDC | Xilinx (Vivado) | `.xdc` |
| PCF | iCE40 (nextpnr) | `.pcf` |
| SDC | Synopsys (OpenSTA / Vivado) | `.sdc` |
| Efinity XML | Efinix (Efinity) | `.xml` |
| QSF | Intel (Quartus) | `.qsf` |
| UCF | 旧 Xilinx (ISE) | `.ucf` |

## 中間表現: ConstraintModel

### ConstraintFormat

```rust
pub enum ConstraintFormat {
    Xdc,
    Pcf,
    Sdc,
    // その他は拡張型
}
```

### 主要構造体

```rust
pub struct ClockConstraint {
    pub name: String,
    pub period_ns: f64,
    pub waveform: Option<String>,
    pub target_pins: Vec<String>,
}

pub struct PinConstraint {
    pub port_name: String,
    pub pin_id: String,
    pub io_standard: Option<String>,
    pub drive_strength: Option<String>,
    pub slew_rate: Option<String>,
    pub differential_pair: Option<String>,
}

pub struct TimingConstraint {
    pub kind: TimingKind,
    pub from_clock: String,
    pub to_clock: String,
    pub delay_ns: f64,
}

pub struct PlacementConstraint {
    pub instance: String,
    pub site: String,
}

pub struct RawConstraint {
    pub format: ConstraintFormat,
    pub text: String,
}
```

## 変換フロー

```
入力フォーマット (XDC / PCF / SDC / ...) → パーサ → ConstraintModel → ジェネレータ → 出力フォーマット
```

- N 種フォーマット → N 個のパーサ + N 個のジェネレータ = 2N 個のモジュール
- 従来の N x N = N^2 個の変換関数を削減

## 対応制約の網羅範囲

- ピンアサイン（PORT → PIN マッピング）
- I/O 標準（LVCMOS33 / LVDS 等）
- ドライブ強度（mA 指定）
- スルーレート（FAST / SLOW）
- 差動ペア（p/n ペア制約）
- クロック制約（周期 / 周波数 / 波形）
- マルチサイクルパス
- フォルスパス
- タイミング例外

## クレート構成

```
constraint-bridge/
├── Cargo.toml
└── src/
    ├── lib.rs              # ConstraintModel, 変換ディスパッチ
    ├── parsers/
    │   ├── xdc.rs          # XDC パーサ
    │   ├── pcf.rs          # PCF パーサ
    │   ├── sdc.rs          # SDC パーサ
    │   ├── efinity.rs      # Efinity XML パーサ
    │   ├── qsf.rs          # QSF パーサ
    │   └── ucf.rs          # UCF パーサ
    └── generators/
        ├── xdc.rs          # XDC ジェネレータ
        ├── pcf.rs          # PCF ジェネレータ
        ├── sdc.rs          # SDC ジェネレータ
        ├── efinity.rs      # Efinity XML ジェネレータ
        ├── qsf.rs          # QSF ジェネレータ
        └── ucf.rs          # UCF ジェネレータ
```

## 関連ドキュメント

- [hdl_lsp_broker.md](hdl_lsp_broker.md) — HDL LSP Broker
- [ip_manager.md](ip_manager.md) — IP Manager
- [observability.md](observability.md) — 監視