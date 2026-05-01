# pcb-conductor メッセージメソッド一覧

**対象 Conductor**: pcb-conductor
**ソース**: 設計仕様書 §14（3492-3630行目付近）, §7（1982-2174行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `pcb`。

## pcb.* メソッド一覧

### 回路図設計

| メソッド | 方向 | 説明 |
|---------|------|------|
| `pcb.generate_schematic` | Request | KiCad ネットリスト出力（`kicad-cli sch export netlist`） |
| `pcb.ai_synthesize` | Request | AI 駆動回路図合成（LLM コア、Chain-of-Thought 6 ステージ） |

### 検証

| メソッド | 方向 | 説明 |
|---------|------|------|
| `pcb.run_drc` | Request | DRC 実行（`kicad-cli pcb drc`） |
| `pcb.run_erc` | Request | ERC 実行（`kicad-cli sch erc`） |

### BOM・配置

| メソッド | 方向 | 説明 |
|---------|------|------|
| `pcb.generate_bom` | Request | BOM 生成（`kicad-cli sch export bom`） |
| `pcb.place_components` | Request | コンポーネント配置データ出力（`kicad-cli pcb export pos`） |
| `pcb.route_traces` | Request | 配線・ドリルデータ出力（`kicad-cli pcb export drill`） |

### 製造出力

| メソッド | 方向 | 説明 |
|---------|------|------|
| `pcb.generate_output` | Request | ガーバー出力（`kicad-cli pcb export gerbers`） |
| `pcb.status` | Request | ビルド状態取得 |

### conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |
| `system.readiness` | Request | 準備状態確認 |

## ペイロード例

### pcb.ai_synthesize リクエスト

```json
{
  "method": "pcb.ai_synthesize",
  "params": {
    "spec": "STM32F103 + BME280 + USB Type-C の温湿度センサボード",
    "input_format": "natural_language",
    "output_format": "kicad"
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### pcb.run_drc リクエスト

```json
{
  "method": "pcb.run_drc",
  "params": {
    "pcb_file": "output/motor_controller.kicad_pcb"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

## 関連ドキュメント

- [pcb/binary_spec.md](binary_spec.md) — hestia-pcb-cli バイナリ仕様
- [pcb/error_types.md](error_types.md) — pcb-conductor エラーコード
- [pcb/state_machines.md](state_machines.md) — PCB ビルドステップ
- [pcb/tool_adapter.md](tool_adapter.md) — AI 駆動回路図設計 / KiCad アダプター
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様