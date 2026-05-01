# デバッグ環境オーケストレーター

**対象領域**: debug-conductor
**ソース**: 設計仕様書 §10（2401-2550行目）

---

## 概要

debug-conductor はハードウェアデバッグ環境を統合管理するオーケストレーターである。JTAG / SWD プロトコルによるターゲット接続、オンチップデバッグ（ILA / SignalTap / Reveal）、ロジックアナライザ（sigrok）、波形キャプチャ（VCD / FST）を統一的に管理し、セッションベースのデバッグワークフローを提供する。debug-conductor は **ローカル専用**（USB プローブアクセスが必要）であり、全サブエージェントもローカル実行となる。

---

## クレート構成

```
debug-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona・main.rs
│   ├── project-model/              # debug.toml パーサー
│   ├── plugin-registry/            # ツール登録・解決
│   ├── adapter-jtag/               # JTAG デバッグ (OpenOCD 統合)
│   ├── adapter-swd/                # SWD デバッグ
│   ├── adapter-ila/                # オンチップデバッグ統合
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── xilinx_ila.rs       # Xilinx ILA
│   │       ├── intel_signaltap.rs  # Intel SignalTap
│   │       └── lattice_reveal.rs   # Lattice Reveal
│   ├── waveform-capture/           # 波形キャプチャ
│   ├── protocol-analyzer/          # プロトコル解析 (sigrok 統合)
│   └── podman-runtime/             # コンテナ管理
├── debug-cli/                      # Rust 製 CLI
└── conductor-sdk/
```

---

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

---

## セッション管理ステートマシン

```
Idle → Connecting → Connected → Running → Paused → Capturing → Disconnected
                                                                     │
                                                               Error ←─ 任意の状態
```

| 状態 | 説明 |
|------|------|
| Idle | セッション未開始 |
| Connecting | デバッグプローブに接続中 |
| Connected | 接続完了、コマンド受付可能 |
| Running | ターゲット実行中 |
| Paused | ターゲット一時停止中 |
| Capturing | 波形キャプチャ中 |
| Disconnected | 切断済み |
| Error | エラー発生 |

```rust
pub enum ResetType {
    Hardware,  // ハードウェアリセット（SRST/TRST ピン使用）
    Software,  // ソフトウェアリセット（レジスタ書き込み）
    System,    // システムリセット（プロセッサ全体）
}
```

---

## 公開 method

| メソッド | 方向 | 説明 |
|---------|------|------|
| `connect` | Request | ターゲットデバイスへ接続 |
| `disconnect` | Request | ターゲットデバイスから切断 |
| `reset` | Request | ターゲットデバイスをリセット |
| `setBreakpoint` | Request | ブレークポイント設定（Source/Address/Symbol 3方式） |
| `removeBreakpoint` | Request | ブレークポイント削除 |
| `run` | Request | ターゲットプログラム実行 |
| `pause` | Request | 一時停止 |
| `stepOver` / `stepInto` | Request | ステップ実行 |
| `readMemory` / `writeMemory` | Request | メモリ読み書き |
| `startCapture` / `stopCapture` | Request | 波形キャプチャ開始/停止 |
| `sessionStateChanged` | Notification | セッション状態変化通知 |
| `breakpointHit` | Notification | ブレークポイント到達通知 |
| `captureComplete` | Notification | キャプチャ完了通知 |

---

## JTAG TAP ステートマシン

adapter-jtag は IEEE 1149.1 準拠の TAP ステートマシンを実装する。TMS 信号で状態遷移を制御する。

```rust
pub enum TapState {
    TestLogicReset, RunTestIdle,
    SelectDR, CaptureDR, ShiftDR, Exit1DR, PauseDR, Exit2DR, UpdateDR,
    SelectIR, CaptureIR, ShiftIR, Exit1IR, PauseIR, Exit2IR, UpdateIR,
}
```

---

## SWD プロトコル

adapter-swd は ARM Serial Wire Debug（2線式: SWCLK / SWDIO）を実装する。

| リクエスト種別 | 説明 | 対象 |
|---------------|------|------|
| `ReadDP` | Debug Port レジスタ読み出し | DPIDR, CTRL/STAT, SELECT 等 |
| `WriteDP` | Debug Port レジスタ書き込み | SELECT, ABORT 等 |
| `ReadAP` | Access Port レジスタ読み出し | CSW, TAR, DRW 等 |
| `WriteAP` | Access Port レジスタ書き込み | CSW, TAR, DRW 等 |

---

## プロトコルデコーダ

debug-conductor は以下のプロトコルデコーダを内蔵する。

| プロトコル | デコード対象 |
|-----------|------------|
| UART | ボーレート自動検出、8N1/7E1 等のフレーム設定 |
| SPI | Mode 0〜3、CPOL/CPHA 設定 |
| I2C | 7bit/10bit アドレス、ACK/NACK 解析 |
| CAN | Standard/Extended ID、DLC、データフィールド |
| LIN | Break/Sync/PID/Data/Checksum 解析 |

---

## サブエージェント構成

debug-conductor は **planner / designer / session_manager / analyzer / programmer** の5種類のサブエージェントを持ち、デバッグセッション管理・波形解析・ファームウェア書込を分担する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で debug-conductor 本体（peer 名 `debug`）と協調する。debug-conductor は **ローカル専用**（USB プローブアクセス）のため全サブエージェントもローカル実行。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `debug-planner` | デバッグプランニング（テストポイント選定、トリガ条件、キャプチャ深さ）| 1 |
| **designer** | `debug-designer` | 検証シナリオ仕様（信号定義、ステート遷移確認項目、期待波形）| 1 |
| **session_manager** | `debug-session` | デバッグセッション管理（JTAG / SWD 接続、OpenOCD/pyOCD 制御、break point/watchpoint 設定）| 1（target ごとに並列可）|
| **analyzer** | `debug-analyzer` | 波形解析 / プロトコルデコード / ロジックアナライザ集計（sigrok/PulseView、ILA/SignalTap/Reveal）| 1 |
| **programmer** | `debug-programmer` | ファームウェア書込（probe-rs、OpenOCD、SVF / JAM）| 1 |

**フロー**: planner → designer → session_manager（接続）→ programmer（書込）→ analyzer（実行 + 解析）。

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター
- [apps_conductor.md](apps_conductor.md) — アプリケーションソフトウェア開発オーケストレーター