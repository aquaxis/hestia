# pcb-conductor ツールアダプター

**対象 Conductor**: pcb-conductor
**ソース**: 設計仕様書 §7.2（2021-2061行目付近）, §7.5（2099-2118行目付近）

## AI 駆動回路図設計パイプライン

自然言語仕様から KiCad / SKiDL 互換の回路図を自動合成する AI パイプライン。

### パイプラインフロー

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
    │  Chain-of-Thought (CoT) 6 ステージ:
    │    Stage 1: RequirementsAnalysis — 回路目的・入出力・電源・制約を分析
    │    Stage 2: BlockDiagram — 機能ブロックと信号フローを定義
    │    Stage 3: ComponentSelection — 入手性・コスト・性能バランスで部品選定
    │    Stage 4: CircuitTopology — バイパスCap/プルアップ/ESD保護含む詳細回路設計
    │    Stage 5: NetlistGeneration — KiCad 互換ネットリスト出力
    │    Stage 6: Verification — 電源/GND/デカップリング/信号整合性検証
    │  入力: CircuitRequirements
    │  出力: GeneratedSchematic（ネットリスト、部品一覧、接続情報、CoT ログ）
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

## 知識グラフ構造

### ノード

| ノード種別 | 属性 |
|-----------|------|
| IC | 型番、メーカー、カテゴリ、パッケージ |
| ピン | 番号、名前、ピン役割 |
| 外部部品 | 種類、値、接続先 |

### ピン役割（PinRole — 全11種）

| 役割 | 説明 |
|------|------|
| PrimaryVdd / PrimaryVss | メイン電源 / GND |
| AnalogVdd / AnalogVss | アナログ電源 / GND |
| SignalInput / SignalOutput / Bidirectional | 信号入力 / 出力 / 双方向 |
| ClockInput / Reset / BootConfig | クロック / リセット / ブート設定 |
| NoConnect | 未接続 |

### エッジ

| エッジ種別 | 説明 |
|-----------|------|
| must_connect_to | ピン → 外部部品/ネットへの必須接続 |
| requires_bypass_cap | VDD ピン → バイパスコンデンサ値 |
| pull_up_required / pull_down_required | ピン → プルアップ/プルダウン抵抗値 |
| crystal_pair | OSC_IN ↔ OSC_OUT、周波数、負荷容量 |

## KiCad アダプター

| フィールド | 値 |
|-----------|---|
| アダプター ID | `org.kicad.kicad8` |
| 対応フォーマット | `kicad*`, `*.kicad_pcb`, `*.kicad_sch` |
| API バージョン | 1 |

### KiCad CLI サブコマンド対応表

| メソッド | サブコマンド | 用途 |
|---------|------------|------|
| `generate_schematic` | `sch export netlist` | ネットリスト出力 |
| `run_drc` | `pcb drc` | DRC 実行 |
| `run_erc` | `sch erc` | ERC 実行 |
| `generate_bom` | `sch export bom` | BOM 生成 |
| `place_components` | `pcb export pos` | 部品配置データ |
| `route_traces` | `pcb export drill` | ドリルデータ |
| `generate_output` | `pcb export gerbers` | ガーバー出力 |

## クレート構成

```
pcb-conductor/
├── crates/
│   ├── conductor-core/             # agent-cli persona・main.rs
│   ├── project-model/              # pcb.toml パーサー
│   ├── plugin-registry/            # ツール登録・解決
│   ├── adapter-kicad/              # KiCad 統合
│   ├── schematic-ai/               # AI 回路図設計エンジン (Rust)
│   ├── knowledge-graph/            # データシート知識グラフ
│   ├── constraint-verifier/        # 多段階検証エンジン
│   └── podman-runtime/             # コンテナ管理
├── packages/
│   └── pcb-ai/                     # LangChain 統合 (TypeScript)
└── pcb-cli/                        # Rust 製 CLI
```

## 関連ドキュメント

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli バイナリ仕様
- [pcb/config_schema.md](config_schema.md) — pcb.toml スキーマ
- [pcb/state_machines.md](state_machines.md) — PCB ビルドステップ
- [pcb/error_types.md](error_types.md) — pcb-conductor エラーコード