# RTL 設計フローオーケストレーター

**対象領域**: rtl-conductor
**ソース**: 設計仕様書 §4（1241-1397行目）

---

## 概要

`rtl-conductor` は FPGA / ASIC 実装の上流に位置し、HDL 言語レベル（SystemVerilog / VHDL / Chisel / SpinalHDL / Amaranth / MyHDL）でのデザイン・検証・解析を担う共有レイヤーオーケストレーターである。下流の `fpga-conductor` / `asic-conductor` に合成可能 RTL を引き渡すハンドオフ機構を持ち、ベンダーツール非依存の RTL 開発プロセスを統一インターフェースで提供する。

---

## クレート構成

```
crates/rtl-conductor/
├── src/
│   ├── lib.rs              # Conductor 本体・agent-cli メッセージハンドラ
│   ├── adapter.rs          # RtlToolAdapter トレイト
│   ├── language.rs         # HDL 言語識別 / トランスパイル管理
│   ├── lint.rs             # Lint / フォーマット / 静的解析
│   ├── simulation.rs       # シミュレーション統合
│   ├── formal.rs           # 形式検証統合
│   ├── repository.rs       # RTL モジュール台帳
│   ├── handoff.rs          # 下流 conductor へのハンドオフ
│   └── fsm_states.rs       # ビルドステートマシン
└── Cargo.toml
```

---

## RtlToolAdapter トレイト

統一インターフェース。全ての RTL ツールアダプターが実装すべきトレイトである。

```rust
#[async_trait]
pub trait RtlToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_languages(&self) -> &[HdlLanguage];   // SystemVerilog / VHDL / Chisel / SpinalHDL / Amaranth
    fn capabilities(&self) -> RtlCapability;            // Lint | Sim | Formal | Transpile

    async fn lint(&self, project: &RtlProject) -> Result<LintReport, AdapterError>;
    async fn simulate(&self, project: &RtlProject, tb: &TestBench) -> Result<SimReport, AdapterError>;
    async fn formal_verify(&self, project: &RtlProject, props: &[Property])
        -> Result<FormalReport, AdapterError>;
    async fn transpile(&self, src: &Path, target_lang: HdlLanguage)
        -> Result<PathBuf, AdapterError>;
}
```

---

## ビルドステートマシン（7状態）

```
Idle → Resolving → Linting → Compiling → Simulating → FormalChecking → Reporting → Done
                                              ↓ (失敗時)
                                          Failed → Diagnosing → 修正提案
```

---

## 統一プロジェクトフォーマット（rtl.toml）

```toml
[project]
name = "core_v"
top = "Cv32e40p"
language = "systemverilog"            # "systemverilog" | "vhdl" | "chisel" | "spinalhdl" | "amaranth"

[sources]
rtl = ["src/**/*.sv"]
testbench = ["tb/**/*.sv"]
constraints_shared = ["constraints/timing_shared.sdc"]

[adapters]
lint = "verilator-lint"
simulation = "verilator"
formal = "symbiyosys"

[handoff]
fpga = ["build/synth_ready.sv"]       # fpga-conductor の [sources] に渡す
asic = ["build/asic_ready.sv"]        # asic-conductor の [sources] に渡す
hal_bus_decl = "build/bus_iface.rdl"  # hal-conductor のバス定義入力
```

---

## 主要11アダプター

| アダプター | 役割 | 対応言語 |
|----------|------|---------|
| `verilator-lint` | Lint | SystemVerilog / Verilog |
| `verible` | Format / Lint | SystemVerilog |
| `verilator` | サイクル精度シミュレーション | SystemVerilog / Verilog |
| `iverilog` | シミュレーション | Verilog |
| `ghdl` | VHDL シミュレーション | VHDL |
| `symbiyosys` | 形式検証（プロパティ）| SystemVerilog |
| `riscof` | RISC-V 命令セット適合性 | RV32I/M/A/F/D/C |
| `cocotb` | Python テストベンチ | 全言語 |
| `chisel-bridge` | Chisel → Verilog | Chisel |
| `spinalhdl-bridge` | SpinalHDL → Verilog | SpinalHDL |
| `amaranth-bridge` | Amaranth → Verilog | Amaranth (Python) |

---

## 下流連携（ハンドオフ）

rtl-conductor のビルド完了時に `meta.handoff` イベントを発行し、ai-conductor が下流ワークフロー（fpga-conductor / asic-conductor / hal-conductor）をトリガーする。ハンドオフ成果物は rtl.toml の `[handoff]` セクションで明示的に指定される。

---

## 公開 method

`rtl.lint.v1` / `rtl.simulate.v1` / `rtl.formal.v1` / `rtl.transpile.v1` / `rtl.handoff.v1` の5系統。agent-cli IPC 上の構造化 JSON ペイロードとして送信される。

---

## サブエージェント構成

rtl-conductor は **planner / designer / coder（複数）/ tester** の4種類のサブエージェントを持ち、機能モジュール単位に複数の coder を並列割当することで RTL 開発を効率化する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で rtl-conductor 本体（peer 名 `rtl`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `rtl-planner` | RTL 開発全体のプランニング（モジュール分割計画、開発順序、検証戦略、coder への作業割当案）| 1 |
| **designer** | `rtl-designer` | RTL 開発の詳細仕様（モジュールインタフェース、信号定義、ステートマシン、タイミング制約）| 1 |
| **coder** | `rtl-coder-{module}` | 割り当てられた機能モジュール単位の HDL コード実装 | **N**（モジュール数だけ動的並列起動、最大16） |
| **tester** | `rtl-tester` | RTL の検証（lint / シミュレーション / 形式検証 / テストベンチ実行 / カバレッジ集計）| 1（必要に応じて並列化可）|

**並列開発フロー:**

1. planner にプラン作成依頼（モジュール一覧 + 依存関係 + 開発順序）
2. designer にモジュール詳細仕様作成依頼
3. モジュール数 N 個の coder を並列起動・割当
4. tester に検証依頼（モジュール完成後 / 全体統合後）
5. 全成果物を集約 → ai-conductor へ完了通知 + 下流 conductor へ handoff

**起動コマンド例:**

```bash
# 常駐サブエージェント
agent-cli run --persona-file ./.hestia/personas/rtl-planner.md  --name rtl-planner  &
agent-cli run --persona-file ./.hestia/personas/rtl-designer.md --name rtl-designer &
agent-cli run --persona-file ./.hestia/personas/rtl-tester.md   --name rtl-tester   &
# coder は planner の出力を見て rtl-conductor が動的に起動
```

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [ai_conductor.md](ai_conductor.md) — ai-conductor 全体概要
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター（下流）
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター（下流）
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター（下流）