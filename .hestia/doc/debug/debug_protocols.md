# debug-conductor デバッグプロトコル

**対象 Conductor**: debug-conductor
**ソース**: 設計仕様書 §10.5（2495-2528行目付近）

## JTAG TAP ステートマシン（§10.5）

adapter-jtag は IEEE 1149.1 準拠の TAP（Test Access Port）ステートマシンを実装する。TMS 信号で状態遷移を制御する。

### TapState 一覧

```rust
pub enum TapState {
    TestLogicReset, RunTestIdle,
    SelectDR, CaptureDR, ShiftDR, Exit1DR, PauseDR, Exit2DR, UpdateDR,
    SelectIR, CaptureIR, ShiftIR, Exit1IR, PauseIR, Exit2IR, UpdateIR,
}
```

### TAP ステートマシン遷移図

TMS=0 / TMS=1 で制御される16状態の有限オートマトン。

- `TestLogicReset` は TMS=1 を5クロック連続入力で常に到達可能（リセット状態）
- `RunTestIdle` はアイドル状態
- DR パス: `SelectDR → CaptureDR → ShiftDR → Exit1DR → PauseDR → Exit2DR → UpdateDR`
- IR パス: `SelectIR → CaptureIR → ShiftIR → Exit1IR → PauseIR → Exit2IR → UpdateIR`

## SWD プロトコル（§10.6）

adapter-swd は ARM Serial Wire Debug（2線式: SWCLK / SWDIO）を実装する。

### リクエスト種別

| リクエスト種別 | 説明 | 対象レジスタ |
|---------------|------|------------|
| `ReadDP` | Debug Port レジスタ読み出し | DPIDR, CTRL/STAT, SELECT 等 |
| `WriteDP` | Debug Port レジスタ書き込み | SELECT, ABORT 等 |
| `ReadAP` | Access Port レジスタ読み出し | CSW, TAR, DRW 等 |
| `WriteAP` | Access Port レジスタ書き込み | CSW, TAR, DRW 等 |

### SWD パケット構成

```
[Start] [APnDP] [RnW] [Addr(2bit)] [Parity] [Stop] [Park] → [Trn] → [Data(32bit)] [Parity]
```

### SWD 特記事項

- JTAG（4線: TCK/TMS/TDI/TDO）に対して2線（SWCLK/SWDIO）で実装
- ARM Cortex-M プロセッサの標準デバッグインターフェース
- OpenOCD / pyOCD でサポート

## プロトコルデコーダ（§10.7）

debug-conductor は以下のプロトコルデコーダを内蔵する（sigrok / PulseView 統合）。

| プロトコル | デコード対象 | 設定パラメータ |
|-----------|------------|--------------|
| UART | ボーレート自動検出、8N1/7E1 等のフレーム設定 | ボーレート、データビット、パリティ、ストップビット |
| SPI | Mode 0〜3、CPOL/CPHA 設定 | クロック極性・位相、CS 極性 |
| I2C | 7bit/10bit アドレス、ACK/NACK 解析 | アドレスモード |
| CAN | Standard/Extended ID、DLC、データフィールド | ビットレート |
| LIN | Break/Sync/PID/Data/Checksum 解析 | ボーレート |

## オンチップデバッグ統合

| ILA 種別 | ベンダー | 接続方式 |
|---------|---------|---------|
| Xilinx ILA | AMD/Xilinx | Vivado hw_server 経由 |
| Intel SignalTap | Intel/Altera | Quartus Signal Tap 経由 |
| Lattice Reveal | Lattice | Radiant Reveal 経由 |

## 波形フォーマット

| フォーマット | 説明 |
|------------|------|
| VCD | Value Change Dump（標準波形フォーマット） |
| FST | Fast Signal Trace（圧縮波形フォーマット） |
| WASM | WASM ベース波形ビューア（100万サンプル、60fps） |

## 関連ドキュメント

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli バイナリ仕様
- [debug/state_machines.md](state_machines.md) — セッション管理ステートマシン
- [debug/message_methods.md](message_methods.md) — debug.* メソッド一覧
- [debug/error_types.md](error_types.md) — debug-conductor エラーコード