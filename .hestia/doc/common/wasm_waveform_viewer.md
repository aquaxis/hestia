# WASM 波形ビューア

**対象領域**: common — 波形表示
**ソース**: 設定仕様書 §13.2

## 概要

VCD / FST / GHW / EVCD をストリーミングパース可能な波形ビューア。`waveform-core` クレートを `cdylib` + `rlib` でビルドし、ブラウザは WebWorker + SharedArrayBuffer 経由でロード、Tauri / VSCode WebView は同クレートを直接利用する。100 万サンプル表示時に 60fps を目標とする。

## 対応フォーマット

| フォーマット | 識別子 | 説明 |
|------------|-------|------|
| VCD | `Vcd` | Value Change Dump（標準）|
| FST | `Fst` | Fast Signal Trace（圧縮）|
| GHW | `Ghw` | GHDL Waveform |
| EVCD | `Evcd` | Extended VCD |

## 主要型

### WaveformFormat

```rust
pub enum WaveformFormat {
    Vcd,
    Fst,
    Ghw,
    Evcd,
}
```

### Signal

```rust
pub struct Signal {
    pub id: String,
    pub full_name: String,
    pub display_name: String,
    pub bit_width: u32,
    pub signal_type: SignalType,
    pub scope: String,
}

pub enum SignalType {
    Wire,
    Reg,
    Integer,
    Real,
}
```

### SignalValue

```rust
pub enum SignalValue {
    Logic(char),              // '0' / '1' / 'X' / 'Z'
    Vector { bits: String, hex: String },
    Real(f64),
    String(String),
}
```

## ビルド構成

`waveform-core` クレートは2つの crate-type でビルド:

| crate-type | 用途 |
|-----------|------|
| `cdylib` | WASM コンパイル（ブラウザ向け）|
| `rlib` | Rust ライブラリ（Tauri / VSCode WebView 向け）|

## レンダリング経路

### ブラウザ経路

```
waveform-core (cdylib → WASM)
  → WebWorker でロード
  → SharedArrayBuffer でメインスレッドと共有
  → Canvas / WebGL で 60fps レンダリング
```

### Tauri / VSCode WebView 経路

```
waveform-core (rlib)
  → 直接リンク
  → ネイティブレンダリング
```

## パフォーマンス目標

| 指標 | 目標値 |
|------|-------|
| 表示サンプル数 | 100 万サンプル |
| フレームレート | 60fps |
| ストリーミングパース | ファイル全体をメモリに載せず段階的読込 |

## 統合先

- VSCode 拡張: WebView 内 WASM レンダリング（§16.1）
- Tauri IDE: ネイティブレンダリング（§16.2）
- debug-conductor: 波形データ提供元

## クレート構成

```
waveform-core/
├── Cargo.toml         # crate-type = ["cdylib", "rlib"]
└── src/
    ├── lib.rs          # 公開 API
    ├── vcd.rs          # VCD パーサ
    ├── fst.rs          # FST パーサ
    ├── ghw.rs          # GHW パーサ
    ├── evcd.rs         # EVCD パーサ
    └── render.rs       # レンダリング抽象
```

## 関連ドキュメント

- [hdl_lsp_broker.md](hdl_lsp_broker.md) — HDL LSP Broker
- [observability.md](observability.md) — 監視