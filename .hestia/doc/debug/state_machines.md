# debug-conductor セッション管理ステートマシン

**対象 Conductor**: debug-conductor
**ソース**: 設計仕様書 §10.3（2450-2475行目付近）

## セッション管理ステートマシン

```
Idle → Connecting → Connected → Running → Paused → Capturing → Disconnected
                                                                     │
                                                               Error ←─ 任意の状態
```

## 状態定義

| 状態 | 説明 |
|------|------|
| Idle | セッション未開始 |
| Connecting | デバッグプローブに接続中（JTAG / SWD） |
| Connected | 接続完了、コマンド受付可能 |
| Running | ターゲット実行中 |
| Paused | ターゲット一時停止中（ブレークポイント到達 / 手動一時停止） |
| Capturing | 波形キャプチャ中 |
| Disconnected | 切断済み |
| Error | エラー発生（任意の状態から遷移可能） |

## 状態遷移ルール

| 遷移 | トリガー | 説明 |
|------|---------|------|
| Idle → Connecting | `debug.connect` | プローブ接続開始 |
| Connecting → Connected | 接続成功 | JTAG / SWD 接続確立 |
| Connecting → Error | 接続失敗 | プローブ未検出・ドライバエラー等 |
| Connected → Running | `debug.run` | ターゲット実行開始 |
| Connected → Disconnected | `debug.disconnect` | 切断 |
| Running → Paused | `debug.pause` / `breakpointHit` | 一時停止 |
| Running → Capturing | `debug.startCapture` | キャプチャ開始 |
| Paused → Running | `debug.run` / `debug.stepOver` / `debug.stepInto` | 再開 |
| Capturing → Running | `debug.stopCapture` | キャプチャ完了 |
| 任意 → Error | エラー発生 | プローブ切断・通信エラー等 |
| Error → Idle | リカバリ | セッション再作成 |
| Disconnected → Idle | リソース解放 | — |

## デバッグフロー

```
デバッグセッション開始
    │
    ▼ デバッグプローブ検出
    │  ├── JTAG: OpenOCD 経由 (USB デバイスアクセス)
    │  └── SWD: OpenOCD / pyOCD 経由
    │
    ├─── オンチップデバッグ
    │    ├── ILA (Xilinx) — Vivado hw_server 経由
    │    ├── SignalTap (Intel) — Quartus Signal Tap 経由
    │    └── Reveal (Lattice) — Radiant Reveal 経由
    │
    ├─── ロジックアナライザ
    │    ├── sigrok — 汎用ロジアナフレームワーク
    │    └── PulseView — GUI 波形ビューア
    │
    └─── 波形キャプチャ・表示・解析
         ├── VCD / FST 形式での波形保存
         └── WASM ベース波形ビューア (100万サンプル、60fps)
```

## ResetType

```rust
pub enum ResetType {
    Hardware,  // ハードウェアリセット（SRST/TRST ピン使用）
    Software,  // ソフトウェアリセット（レジスタ書き込み）
    System,    // システムリセット（プロセッサ全体）
}
```

## 関連ドキュメント

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli バイナリ仕様
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD プロトコル
- [debug/message_methods.md](message_methods.md) — debug.* メソッド一覧
- [debug/error_types.md](error_types.md) — debug-conductor エラーコード