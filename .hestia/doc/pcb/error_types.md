# pcb-conductor エラーコード

**対象 Conductor**: pcb-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）

## エラーコード範囲

pcb-conductor のエラーコードは **-32400 〜 -32499** の範囲を使用する。

## エラーカテゴリ

### Schematic（回路図）

| コード | 名称 | 説明 |
|-------|------|------|
| -32400 | SCHEMATIC_GENERATION_FAILED | 回路図生成失敗 |
| -32401 | SCHEMATIC_PARSE_ERROR | 回路図パースエラー |
| -32402 | NETLIST_GENERATION_FAILED | ネットリスト生成失敗 |
| -32403 | SCHEMATIC_FORMAT_UNSUPPORTED | 未対応の回路図フォーマット |

### DRC / ERC

| コード | 名称 | 説明 |
|-------|------|------|
| -32410 | DRC_FAILED | DRC 実行失敗 |
| -32411 | DRC_VIOLATIONS_FOUND | DRC 違反検出 |
| -32412 | ERC_FAILED | ERC 実行失敗 |
| -32413 | ERC_VIOLATIONS_FOUND | ERC 違反検出（未接続ピン、電源接続、ドライバ競合、ショート） |

### BOM / Placement

| コード | 名称 | 説明 |
|-------|------|------|
| -32420 | BOM_GENERATION_FAILED | BOM 生成失敗 |
| -32421 | BOM_PART_NOT_FOUND | BOM 内の部品がライブラリに存在しない |
| -32422 | PLACEMENT_FAILED | コンポーネント配置失敗 |
| -32423 | PLACEMENT_DRC_ERROR | 配置後 DRC 違反 |

### Gerber / 出力

| コード | 名称 | 説明 |
|-------|------|------|
| -32430 | GERBER_GENERATION_FAILED | ガーバー出力失敗 |
| -32431 | DRILL_DATA_FAILED | ドリルデータ生成失敗 |
| -32432 | OUTPUT_FORMAT_UNSUPPORTED | 未対応の出力フォーマット |

### AI Synthesis

| コード | 名称 | 説明 |
|-------|------|------|
| -32440 | AI_SYNTHESIS_FAILED | AI 駆動回路図合成失敗 |
| -32441 | AI_COT_FAILED | Chain-of-Thought 生成失敗 |
| -32442 | AI_LLM_UNAVAILABLE | LLM バックエンド利用不可 |

### Knowledge Graph

| コード | 名称 | 説明 |
|-------|------|------|
| -32450 | KG_BUILD_FAILED | 知識グラフ構築失敗 |
| -32451 | KG_DATASHEET_FETCH_FAILED | データシート取得失敗 |
| -32452 | KG_NODE_RESOLUTION_FAILED | KG ノード解決失敗 |

### Constraint Verification

| コード | 名称 | 説明 |
|-------|------|------|
| -32460 | CONSTRAINT_VERIFY_SYNTAX | Level 1: 構文検証失敗 |
| -32461 | CONSTRAINT_VERIFY_ERC | Level 2: ERC 検証失敗 |
| -32462 | CONSTRAINT_VERIFY_KG_INTRA | Level 3: KG ベース検証（ピン内）失敗 |
| -32463 | CONSTRAINT_VERIFY_KG_INTER | Level 4: KG ベース検証（ピン間）失敗 |
| -32464 | CONSTRAINT_VERIFY_TOPOLOGY | Level 5: トポロジー検証失敗 |

## 関連ドキュメント

- [pcb/message_methods.md](message_methods.md) — pcb.* メソッド一覧
- [pcb/state_machines.md](state_machines.md) — PCB ビルドステップ
- [pcb/tool_adapter.md](tool_adapter.md) — AI 駆動回路図設計 / KiCad アダプター
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ