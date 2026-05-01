# ASIC 設計フローオーケストレーター

**対象領域**: asic-conductor
**ソース**: 設計仕様書 §6（1761-1981行目）

---

## 概要

asic-conductor は RTL-to-GDSII の全13ステップパイプラインを自動化するオーケストレーターである。オープンソースツールチェーン（Yosys / OpenROAD / Magic / Netgen / OpenLane 2）を中心に、PDK 管理（Sky130 / GF180MCU / IHP SG13G2）を統合し、論理合成から物理設計・サインオフ検証までを一貫して実行する。

---

## クレート構成

```
asic-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona・main.rs
│   ├── project-model/              # asic.toml パーサー
│   ├── plugin-registry/            # ツール登録・解決（AsicToolAdapter トレイト）
│   ├── adapter-openlane/           # OpenLane 2 統合
│   ├── adapter-yosys/              # Yosys 論理合成（RTLIL 経由）
│   ├── adapter-openroad/           # OpenROAD 配置配線
│   ├── pdk-manager/                # PDK 管理（Sky130 / GF180MCU / IHP SG13G2）
│   ├── podman-runtime/             # コンテナ管理
│   └── conductor-sdk/              # 共有 SDK
├── asic-cli/                       # Rust 製 CLI クライアント
└── conductor-sdk/
```

---

## RTL-to-GDSII 13ステップパイプライン

```
RTL (SystemVerilog / Verilog)
    │
    ▼ 1. Yosys (論理合成)
    │   read_verilog → RTLIL → proc → opt → fsm → memory → abc
    │
    ▼ 2. OpenSTA (初期タイミング解析)
    │   セットアップ/ホールド違反の早期検出
    │
    ▼ 3. OpenROAD フロアプラン
    │   PDN Generation / I/O Placement / Macro Placement
    │
    ▼ 4. RePlAce (グローバル配置)
    │
    ▼ 5. OpenDP (詳細配置)
    │
    ▼ 6. TritonCTS (クロックツリー合成)
    │   バッファ挿入・スキュー最小化
    │
    ▼ 7. FastRoute (グローバルルーティング)
    │
    ▼ 8. TritonRoute (詳細ルーティング)
    │   DRC 準拠の金属配線
    │
    ▼ 9. OpenRCX (寄生容量抽出)
    │
    ▼ 10. OpenSTA (最終タイミング解析)
    │    SPEF ベースの正確なタイミング検証
    │
    ▼ 11. Magic (DRC) / Netgen (LVS)
    │    設計ルールチェック / レイアウト対回路図検証
    │
    ▼ 12-13. GDSII 出力
```

---

## サポート PDK

| PDK | プロセス | 提供元 | 用途 |
|-----|---------|--------|------|
| Sky130 | 130nm CMOS | SkyWater Technology | デジタル・混合信号、最も安定 |
| GF180MCU | 180nm CMOS | GlobalFoundries | MCU 向け、高信頼性 |
| IHP SG13G2 | 130nm BiCMOS | IHP | 高速アナログ・高周波設計 |

---

## AsicToolAdapter トレイト

ASIC 固有のツールアダプターインターフェース。FPGA の VendorAdapter と異なり、物理設計ステップ（フロアプラン、CTS、寄生抽出等）を網羅している。

```rust
#[async_trait]
pub trait AsicToolAdapter: Send + Sync + 'static {
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> &AsicCapabilitySet;

    // コアフロー（7ステップ）
    async fn synthesize(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn floorplan(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn place(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn cts(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn route(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn extract(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_gdsii(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;

    // サインオフ
    async fn timing_signoff(&self, ctx: &AsicBuildContext) -> Result<TimingReport, AdapterError>;
    async fn drc(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
    async fn lvs(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
}
```

---

## 13状態ビルドステートマシン

| 状態 | 進捗 | 説明 |
|------|------|------|
| `Idle` | 0% | 初期状態 |
| `PdkResolving` | 3% | PDK バージョン解決・パス検証中 |
| `Synthesizing` | 10% | 論理合成実行中 (Yosys) |
| `Floorplanning` | 20% | フロアプラン作成中 |
| `Placing` | 30% | セル配置実行中 |
| `CTS` | 45% | クロックツリー合成実行中 |
| `Routing` | 60% | 配線実行中 |
| `Extraction` | 70% | 寄生抽出実行中 |
| `TimingSignoff` | 75% | タイミングサインオフ検証中 |
| `DRC` | 80% | デザインルールチェック実行中 |
| `LVS` | 90% | レイアウト対回路図検証実行中 |
| `GDSII` | 95% | GDSII ストリーム生成中 |
| `Success` | 100% | ビルド成功 |

---

## AsicCapabilityRouter（ルーティング戦略）

| 戦略 | 説明 |
|------|------|
| `PreferOpenLane` | OpenLane2 が対応可能なステップは OpenLane2 に委譲する |
| `StepOptimal` | 各ステップごとに最適なアダプターを個別選択する |
| `Explicit` | asic.toml で明示的に指定されたアダプターを使用する |

---

## SignoffChecker

テープアウト前の最終検証を担当する。

```rust
pub struct SignoffResult {
    pub tool: SignoffTool,
    pub check_type: CheckType,     // DRC or LVS
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub summary: SignoffSummary,
}
```

| ツール | 検証種別 | 説明 |
|--------|---------|------|
| Magic | DRC | レイアウト DRC エンジン |
| Netgen | LVS | SPICE レベルの回路比較 |
| KLayout | DRC + LVS | スクリプタブルなレイアウト検証 |

**AI エージェント連携機能:**

| 機能 | 説明 |
|------|------|
| タイミング違反自動修復 | タイミングサインオフ失敗時に制約緩和またはバッファ挿入を提案 |
| DRC 違反自動修復 | DRC 違反パターンに基づきレイアウト修正パッチを生成 |
| PDK マイグレーション | 異なる PDK ファミリー間の設計移行を支援 |
| フロアプラン最適化 | 配置密度・配線混雑度に基づきフロアプラン改善を提案 |

---

## asic.toml 設定例

```toml
[project]
name = "my-asic-project"
version = "0.1.0"
rtl_files = ["src/*.v"]
top = "top_module"

[target]
pdk = "sky130_fd_sc_hd"
clock_period_ns = 10.0

[synthesis]
flatten = true
abc_script = "resyn2"
strategy = "area"

[placement]
target_density = 0.6

[cts]
max_skew_ns = 0.5

[routing]
min_layer = "met1"
max_layer = "met5"
```

---

## Hestia との統合方式

Hestia は OpenLane 2 を Podman コンテナ内で実行し、conductor-core から agent-cli IPC 経由で制御する。OpenLane 2 の Python ベースの Step-based Execution を活用し、各工程の個別再実行が可能である。PDK は pdk-manager で自動解決され、volare との統合による自動ダウンロードをサポートする。

---

## サブエージェント構成

asic-conductor は **planner / designer / synthesizer / implementer / signoff_checker / tester** の6種類のサブエージェントを持ち、13ステップ RTL-to-GDSII フローを分担する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で asic-conductor 本体（peer 名 `asic`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `asic-planner` | ASIC 開発プランニング（PDK 選定、ステップ実行戦略、signoff 計画）| 1 |
| **designer** | `asic-designer` | ASIC 詳細仕様（フロアプラン方針、制約、電源プラン、Tape-out 要件）| 1 |
| **synthesizer** | `asic-synthesizer` | logic synthesis（Yosys、SDC タイミング制約適用）| 1 |
| **implementer** | `asic-implementer` | floorplan + placement + CTS + routing（OpenROAD / TritonCTS / TritonRoute）| 1 |
| **signoff_checker** | `asic-signoff` | DRC / LVS / timing signoff / EM/IR drop 解析（Magic / Netgen / OpenSTA）| 1 |
| **tester** | `asic-tester` | post-layout sim、形式検証（SymbiYosys）、Ngspice によるアナログ検証 | 1 |

**フロー**: planner → designer → synthesizer → implementer → signoff_checker → tester の順次実行。OpenLane 2 の Step-based Execution と整合し、特定ステップの再実行は対応サブエージェントを再呼出可能。

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター（上流）
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター
- [debug_conductor.md](debug_conductor.md) — デバッグ環境オーケストレーター