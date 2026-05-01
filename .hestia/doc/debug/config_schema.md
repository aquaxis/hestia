# debug-conductor 設定スキーマ

**対象 Conductor**: debug-conductor
**ソース**: 設計仕様書 §10（2401-2550行目付近）

## debug-conductor 設定

debug-conductor はローカル専用 conductor であり、USB デバッグプローブへのアクセスを必要とする。

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

## プローブ設定

| 設定項目 | 型 | 説明 |
|---------|---|------|
| probe_type | string | プローブ種別（`stlink` / `jlink` / `cmsis-dap` / `ftdi`） |
| interface | string | インターフェース（`jtag` / `swd`） |
| clock_speed_khz | integer | クロック速度（kHz） |
| target_device | string | ターゲットデバイス名 |

## オンチップデバッグ設定

| 設定項目 | 型 | 説明 |
|---------|---|------|
| ila_type | string | ILA 種別（`xilinx_ila` / `intel_signaltap` / `lattice_reveal`） |
| trigger_position | integer | トリガ位置 |
| sample_depth | integer | サンプル深度 |
| signals | string[] | キャプチャ信号リスト |

## 波形キャプチャ設定

| 設定項目 | 型 | 説明 |
|---------|---|------|
| format | string | 波形フォーマット（`vcd` / `fst`） |
| max_samples | integer | 最大サンプル数 |
| viewer | string | ビューア（`pulseview` / `wasm`） |

## リセット種別

| 種別 | 説明 |
|------|------|
| Hardware | ハードウェアリセット（SRST/TRST ピン使用） |
| Software | ソフトウェアリセット（レジスタ書き込み） |
| System | システムリセット（プロセッサ全体） |

## 関連ドキュメント

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli バイナリ仕様
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD プロトコル
- [debug/state_machines.md](state_machines.md) — セッション管理ステートマシン
- [../apps/hil_sil.md](../apps/hil_sil.md) — HIL/SIL テスト