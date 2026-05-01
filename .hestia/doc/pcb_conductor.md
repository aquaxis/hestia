# PCB 設計フローオーケストレーター

**対象領域**: pcb-conductor
**ソース**: 設計仕様書 §7（1982-2174行目）

---

## 概要

pcb-conductor は PCB（プリント基板）設計フローを統合管理するオーケストレーターである。最大の特徴は AI 駆動回路図設計パイプラインであり、自然言語仕様から知識グラフを構築し、Chain-of-Thought プロンプトで回路図を自動合成する。KiCad を主要ツールとして統合し、SKiDL / Freerouting との連携により設計から製造出力までを一貫して支援する。

---

## クレート構成

```
pcb-conductor/
├── Cargo.toml
├── crates/
│   ├── conductor-core/             # agent-cli persona・main.rs
│   ├── project-model/              # pcb.toml パーサー
│   ├── plugin-registry/            # ツール登録・解決
│   ├── adapter-kicad/              # KiCad 統合
│   ├── schematic-ai/               # AI 回路図設計エンジン (Rust)
│   │   └── src/
│   │       ├── lib.rs              # SchematicAiEngine
│   │       ├── cot_prompt.rs       # Chain-of-Thought プロンプト生成
│   │       └── requirements.rs     # CircuitRequirements パーサー
│   ├── knowledge-graph/            # データシート知識グラフ
│   │   └── src/
│   │       ├── lib.rs              # KnowledgeGraph
│   │       ├── node.rs             # IC/ピン/外部部品ノード
│   │       └── edge.rs             # 接続制約エッジ
│   ├── constraint-verifier/        # 多段階検証エンジン
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── syntax.rs           # Level 1: 構文検証
│   │       ├── erc.rs              # Level 2: ERC
│   │       ├── kg_intra.rs         # Level 3: KG ベース検証（ピン内）
│   │       ├── kg_inter.rs         # Level 4: KG ベース検証（ピン間）
│   │       └── topology.rs         # Level 5: トポロジー検証
│   └── podman-runtime/             # コンテナ管理
├── packages/
│   └── pcb-ai/                     # LangChain 統合 (TypeScript)
│       └── src/
│           └── schematic_synthesizer.ts  # LLM 回路図合成
├── pcb-cli/                        # Rust 製 CLI
└── conductor-sdk/
```

---

## AI 駆動回路図設計パイプライン

```
自然言語仕様入力
    │  「STM32F103 + BME280 + USB Type-C の温湿度センサボード」
    ↓
[Step 1] Requirements Parser (要件パーサー)
    │  自然言語 → 構造化要件 (CircuitRequirements)
    ↓
[Step 2] BOM Generator (BOM 生成器)
    │  要件 → 部品リスト (型番・数量・メーカー)
    ↓
[Step 3] Datasheet Fetcher (データシート取得・解析)
    │  各 IC のデータシートをダウンロード・解析
    ↓
[Step 4] Knowledge Graph Builder (知識グラフ構築)
    │  データシート → KG (IC ノード + ピンノード + エッジ)
    ↓
[Step 5] Schematic Synthesizer (回路図合成) ← LLM コア
    │  Chain-of-Thought (CoT) 6ステージ:
    │    Stage 1: RequirementsAnalysis — 回路目的・入出力・電源・制約を分析
    │    Stage 2: BlockDiagram — 機能ブロックと信号フローを定義
    │    Stage 3: ComponentSelection — 入手性・コスト・性能バランスで部品選定
    │    Stage 4: CircuitTopology — バイパスCap/プルアップ/ESD保護含む詳細回路設計
    │    Stage 5: NetlistGeneration — KiCad 互換ネットリスト出力
    │    Stage 6: Verification — 電源/GND/デカップリング/信号整合性検証
    ↓
[Step 6] Constraint Verifier (多段階検証)
    │  Level 1: Python / SKiDL 構文チェック、ライブラリ存在確認
    │  Level 2: ERC — 未接続ピン、電源接続、ドライバ競合、ショート検出
    │  Level 3: KG ベース検証 (ピン内) — VDD/VSS 接続、バイパスコンデンサ
    │  Level 4: KG ベース検証 (ピン間) — IC 間インターフェース整合性
    │  Level 5: トポロジー検証 — サブグラフ同型性、信号パス完全性
    ↓  ← フィードバックループ（不合格時は Step 5 へ戻る、最大3回）
[Step 7] Output Generator (出力生成)
    │  KiCad / Altium 形式のネットリスト出力
    ↓
設計完了
```

---

## 知識グラフ構造

```
ノード (Node):
  ├── IC: {型番, メーカー, カテゴリ, パッケージ}
  ├── ピン: {番号, 名前, ピン役割}
  └── 外部部品: {種類, 値, 接続先}

ピン役割 (PinRole — 全11種):
  ├── PrimaryVdd / PrimaryVss (メイン電源 / GND)
  ├── AnalogVdd / AnalogVss (アナログ電源 / GND)
  ├── SignalInput / SignalOutput / Bidirectional
  ├── ClockInput / Reset / BootConfig
  └── NoConnect

エッジ (Edge):
  ├── must_connect_to: {ピン → 外部部品/ネット}
  ├── requires_bypass_cap: {VDD ピン → コンデンサ値}
  ├── pull_up_required / pull_down_required: {ピン → 抵抗値}
  └── crystal_pair: {OSC_IN ↔ OSC_OUT, 周波数, 負荷容量}
```

---

## PCB ビルドステップ（9ステップ）

| ステップ | 説明 |
|---------|------|
| `ParseRequirements` | 要件パース |
| `GenerateBom` | BOM 生成 |
| `AnalyzeDatasheet` | データシート解析 |
| `BuildKnowledgeGraph` | ナレッジグラフ構築 |
| `SynthesizeSchematic` | 回路図合成（LLM コア） |
| `Verify` | 検証（DRC/ERC/KG 5段階） |
| `PlaceComponents` | コンポーネント配置 |
| `RouteTraces` | 配線 |
| `GenerateOutput` | 製造出力生成（ガーバー等） |

---

## KiCad アダプター

| フィールド | 値 |
|-----------|---|
| アダプター ID | `org.kicad.kicad8` |
| 対応フォーマット | `kicad*`, `*.kicad_pcb`, `*.kicad_sch` |
| API バージョン | 1 |

**KiCad CLI サブコマンド対応表:**

| メソッド | サブコマンド | 用途 |
|---------|------------|------|
| `generate_schematic` | `sch export netlist` | ネットリスト出力 |
| `run_drc` | `pcb drc` | DRC 実行 |
| `run_erc` | `sch erc` | ERC 実行 |
| `generate_bom` | `sch export bom` | BOM 生成 |
| `place_components` | `pcb export pos` | 部品配置データ |
| `route_traces` | `pcb export drill` | ドリルデータ |
| `generate_output` | `pcb export gerbers` | ガーバー出力 |

---

## pcb.toml 設定例

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

---

## サブエージェント構成

pcb-conductor は **planner / designer / schematic / layout / tester** の5種類のサブエージェントを持ち、回路図設計 → 配置配線 → 検証フローを分担する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で pcb-conductor 本体（peer 名 `pcb`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `pcb-planner` | PCB 開発プランニング（基板規模、層数、コネクタ配置、部品調達戦略）| 1 |
| **designer** | `pcb-designer` | PCB 詳細仕様（回路ブロック構成、IO 配置、電源プラン、信号品質要件）| 1 |
| **schematic** | `pcb-schematic` | AI 駆動回路図設計（SKiDL / KiCad、知識グラフ活用）| 1 |
| **layout** | `pcb-layout` | アートワーク（配置 + 配線、Freerouting 連携）| 1 |
| **tester** | `pcb-tester` | DRC / ERC / BOM 検証 + Gerber 出力検証 | 1 |

**フロー**: planner → designer → schematic → layout → tester の順次実行。AI 駆動回路図生成は schematic サブエージェントが担当。

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター