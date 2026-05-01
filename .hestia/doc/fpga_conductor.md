# FPGA 設計フローオーケストレーター

**対象領域**: fpga-conductor
**ソース**: 設計仕様書 §5（1398-1760行目）

---

## 概要

fpga-conductor は FPGA デザインの合成・配置配線・ビットストリーム生成・デバイスプログラミングを統一的に管理するオーケストレーターである。AMD Vivado / Intel Quartus Prime / Efinix Efinity / Lattice Radiant / OSS（Yosys + nextpnr）の全ベンダーを VendorAdapter トレイトで抽象化し、fpga.toml による宣言的プロジェクト定義と fpga.lock による再現性を提供する。

---

## クレート構成

```
fpga-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona・ステートマシン・main.rs
│   │   └── src/
│   │       ├── main.rs             # デーモン起動エントリポイント
│   │       ├── rpc.rs              # agent-cli メッセージハンドラー
│   │       ├── state_machine.rs    # ビルドステートマシン
│   │       ├── router.rs           # CapabilityRouter
│   │       └── self_healing.rs     # SelfHealingPipeline
│   ├── project-model/              # fpga.toml パーサー・モデル
│   │   └── src/
│   │       ├── lib.rs              # ProjectInfo, Target 定義
│   │       ├── parser.rs           # TOML パーサー
│   │       └── lock.rs            # fpga.lock 管理
│   ├── plugin-registry/            # アダプター登録・解決エンジン
│   │   └── src/
│   │       ├── lib.rs              # PluginRegistry
│   │       ├── adapter/
│   │       │   ├── mod.rs          # VendorAdapter トレイト
│   │       │   ├── script.rs       # ScriptAdapter (adapter.toml)
│   │       │   ├── dynamic.rs      # Dynamic Adapter (dlopen)
│   │       │   └── remote.rs       # Remote Adapter (gRPC)
│   │       └── capability.rs       # CapabilitySet, CapabilityRouter
│   ├── adapter-vivado/             # AMD Vivado アダプター
│   │   └── src/
│   │       ├── lib.rs              # VivadoAdapter 実装
│   │       └── templates/          # TCL テンプレート (minijinja)
│   ├── adapter-quartus/            # Intel Quartus Prime アダプター
│   │   └── src/
│   │       └── lib.rs              # QuartusAdapter (QSF/QIP 生成)
│   ├── adapter-efinity/            # Efinix Efinity アダプター
│   │   └── src/
│   │       └── lib.rs              # EfinityAdapter (Python API 呼び出し)
│   ├── constraint-bridge/          # XDC ⇔ SDC ⇔ Efinity XML ⇔ PCF 変換
│   ├── toolchain-registry/         # バージョン検出・解決
│   ├── compat-matrix/              # 互換性マトリクス DB (SQLite)
│   ├── podman-runtime/             # Podman コンテナ管理
│   ├── hdl-lsp-broker/             # HDL 言語サーバープロキシ
│   ├── waveform-core/              # VCD/FST パーサー (→WASM 対応)
│   └── agent-system/               # AI エージェント群 (Rust 部分)
│       └── src/
│           ├── watcher.rs          # WatcherAgent
│           ├── probe.rs            # ProbeAgent
│           └── validator.rs        # ValidatorAgent
├── packages/
│   ├── vscode-extension/           # VSCode 拡張 (TypeScript)
│   ├── agent-system/               # PatcherAgent (TypeScript + Anthropic SDK)
│   ├── fpga-ci/                    # CI/CD CLI (TypeScript)
│   └── tauri-app/                  # Tauri デスクトップアプリ
├── fpga-cli/                       # Rust 製 CLI クライアント
└── conductor-sdk/                  # サードパーティ向け SDK
```

---

## VendorAdapter トレイト

すべてのアダプターが実装すべき統一インターフェースである。

```rust
#[async_trait::async_trait]
pub trait VendorAdapter: Send + Sync + 'static {
    // --- 必須: 自己記述 ---
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> CapabilitySet;

    // --- 必須: コアフロー ---
    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;

    // --- オプション (デフォルト: CapabilityUnsupported を返す) ---
    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError>;
    async fn start_debug_session(&self, ctx: &BuildContext) -> Result<DebugSession, AdapterError>;
    async fn hls_compile(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn program_device(&self, ctx: &ProgramContext) -> Result<(), AdapterError>;
    async fn simulate(&self, ctx: &SimContext) -> Result<SimResult, AdapterError>;

    // --- ログ診断 (デフォルト: None) ---
    fn parse_log_line(&self, line: &str) -> Option<Diagnostic> { None }
}
```

**AdapterManifest**: `id` / `name` / `version` / `vendor` / `api_version` / `supported_devices` / `capabilities` / `release_notes_url`

**CapabilitySet**: `synthesis` / `implementation` / `bitstream` / `timing_analysis` / `on_chip_debug` / `device_program` / `hls` / `simulation` / `ip_catalog`

---

## ビルドステートマシン

```
Idle → Resolving (ツールチェーン確定)
     → ContainerStarting (Podman コンテナ起動)
     → Synthesizing (adapter.synthesize)
     → Implementing (adapter.implement)
     → Bitstreamming (adapter.generate_bitstream)
     → Success

各ステップで失敗 → SelfHealingPipeline.on_build_failure()
    → CompatibilityMatrix で診断
    → 既知パッチあり → 自動適用/通知
    → 未知エラー    → PatcherAgent 起動
```

---

## 統一プロジェクトフォーマット（fpga.toml）

```toml
[project]
name    = "my_dsp_core"
version = "0.2.0"
hdl_files   = ["hdl/top.sv", "hdl/fir_filter.sv", "hdl/bram_ctrl.sv"]
include_dirs = ["hdl/include"]
testbenches = ["sim/tb_top.sv", "sim/tb_fir.sv"]

# ターゲット定義
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
vendor      = "yosyshq"     # OSS チェーン (adapter.toml)
device      = "iCE40HX8K"
top         = "top"
constraints = ["constraints/ice40.pcf"]

# ツールチェーンバージョン制約 (semver)
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

---

## Vivado アダプター実装

- minijinja テンプレートエンジンで TCL スクリプト自動生成
- Vivado を `-mode batch` で起動
- リアルタイムログパース（`ERROR: [Synth 8-439]` 形式の正規表現マッチ）

## Quartus アダプター実装

- .qpf / .qsf プロジェクトファイル自動生成
- `quartus_sh --flow compile` で全フロー実行

## Efinity アダプター実装

- インターフェーススクリプト (XML) を Rust の serde で直接生成
- ビルドスクリプトを Rust テンプレートエンジンで生成
- Efinity 同梱の Python で実行（外部 Python に依存しない）

---

## サブエージェント構成

fpga-conductor は **planner / designer / synthesizer / implementer / tester / programmer** の6種類のサブエージェントを持つ。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で fpga-conductor 本体（peer 名 `fpga`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `fpga-planner` | FPGA 開発プランニング（target/family 選定、ビルド戦略、IP 利用判断）| 1 |
| **designer** | `fpga-designer` | FPGA 詳細仕様（XDC/SDC/PCF 制約、IO mapping、クロックドメイン、IP 構成）| 1 |
| **synthesizer** | `fpga-synthesizer` | RTL → netlist 合成（Vivado / Quartus / Efinity / Yosys+nextpnr）| 1（target 並列時 N）|
| **implementer** | `fpga-implementer` | 配置配線 + bitstream 生成 | 1（target 並列時 N）|
| **tester** | `fpga-tester` | シミュレーション + タイミング検証 + リソース解析 | 1 |
| **programmer** | `fpga-programmer` | FPGA への bitstream 書込（debug-conductor 連携）| 1 |

**フロー**: planner → designer → synthesizer → implementer → tester → programmer の順次実行。複数 target 並列ビルド時は synthesizer / implementer が target ごとに動的並列起動。

**起動コマンド例:**

```bash
agent-cli run --persona-file ./.hestia/personas/fpga-planner.md     --name fpga-planner     &
agent-cli run --persona-file ./.hestia/personas/fpga-designer.md    --name fpga-designer    &
agent-cli run --persona-file ./.hestia/personas/fpga-synthesizer.md --name fpga-synthesizer &
agent-cli run --persona-file ./.hestia/personas/fpga-implementer.md --name fpga-implementer &
agent-cli run --persona-file ./.hestia/personas/fpga-tester.md      --name fpga-tester      &
agent-cli run --persona-file ./.hestia/personas/fpga-programmer.md  --name fpga-programmer  &
```

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター（上流）
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター
- [debug_conductor.md](debug_conductor.md) — デバッグ環境オーケストレーター