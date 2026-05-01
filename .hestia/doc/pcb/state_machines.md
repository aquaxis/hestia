# pcb-conductor ビルドステップ

**対象 Conductor**: pcb-conductor
**ソース**: 設計仕様書 §7.4（2085-2098行目付近）

## 9 ステップビルドフロー

| ステップ | 説明 | 主要コンポーネント |
|---------|------|------------------|
| `ParseRequirements` | 要件パース | RequirementsParser（自然言語 → CircuitRequirements） |
| `GenerateBom` | BOM 生成 | BOM Generator（要件 → 部品リスト） |
| `AnalyzeDatasheet` | データシート解析 | Datasheet Fetcher（各 IC のデータシートダウンロード・解析） |
| `BuildKnowledgeGraph` | ナレッジグラフ構築 | Knowledge Graph Builder（データシート → KG） |
| `SynthesizeSchematic` | 回路図合成（LLM コア） | Schematic Synthesizer（CoT 6 ステージ） |
| `Verify` | 検証（DRC/ERC/KG 5段階） | Constraint Verifier（多段階検証エンジン） |
| `PlaceComponents` | コンポーネント配置 | KiCad アダプター（`pcb export pos`） |
| `RouteTraces` | 配線 | KiCad アダプター（`pcb export drill`）+ Freerouting 連携 |
| `GenerateOutput` | 製造出力生成（ガーバー等） | KiCad アダプター（`pcb export gerbers`） |

## 状態遷移

```
ParseRequirements
       │
       ▼
GenerateBom
       │
       ▼
AnalyzeDatasheet
       │
       ▼
BuildKnowledgeGraph
       │
       ▼
SynthesizeSchematic ← フィードバックループ（最大3回）
       │              ↑
       ▼              │
Verify ───── 不合格 →─┘
       │
       │ 合格
       ▼
PlaceComponents
       │
       ▼
RouteTraces
       │
       ▼
GenerateOutput
       │
       ▼
Done
```

## フィードバックループ

Verify ステップで不合格の場合、SynthesizeSchematic ステップに戻り再生成を試行する（最大3回）。これにより AI 生成回路図の品質を反復的に向上させる。

## 各ステップの詳細

### ParseRequirements

自然言語仕様入力を CircuitRequirements（概要、I/O、電源電圧、制約）に変換する。

### GenerateBom

CircuitRequirements から部品リスト（型番・数量・メーカー）を生成する。

### AnalyzeDatasheet

各 IC のデータシートをダウンロード・解析し、ピン配置・電気特性・推奨回路を抽出する。rag-conductor（§13.7）のデータシート知識ベースを活用。

### BuildKnowledgeGraph

データシートからナレッジグラフ（IC ノード + ピンノード + エッジ）を構築する。

### SynthesizeSchematic

LLM による Chain-of-Thought 6 ステージで回路図を合成する。

### Verify

5 段階の多段階検証:
- Level 1: 構文検証
- Level 2: ERC
- Level 3: KG ベース検証（ピン内）
- Level 4: KG ベース検証（ピン間）
- Level 5: トポロジー検証

## 関連ドキュメント

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli バイナリ仕様
- [pcb/error_types.md](error_types.md) — pcb-conductor エラーコード
- [pcb/tool_adapter.md](tool_adapter.md) — AI 駆動回路図設計 / KiCad アダプター
- [pcb/config_schema.md](config_schema.md) — pcb.toml スキーマ