# fpga-conductor エラーコード

**対象 Conductor**: fpga-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）

## エラーコード範囲

fpga-conductor のエラーコードは **-32200 〜 -32299** の範囲を使用する。

## エラーカテゴリ

### Synthesis（合成）

| コード | 名称 | 説明 |
|-------|------|------|
| -32200 | SYNTHESIS_FAILED | RTL 合成失敗（Vivado / Quartus / Efinity / Yosys） |
| -32201 | SYNTHESIS_TIMEOUT | 合成タイムアウト |
| -32202 | SYNTHESIS_INVALID_HDL | 合成不可能な HDL 検出 |

### Implementation（配置配線）

| コード | 名称 | 説明 |
|-------|------|------|
| -32210 | IMPLEMENTATION_FAILED | 配置配線失敗 |
| -32211 | PLACEMENT_FAILED | セル配置失敗 |
| -32212 | ROUTING_FAILED | 配線失敗（混雑度過大等） |

### Bitstream

| コード | 名称 | 説明 |
|-------|------|------|
| -32220 | BITSTREAM_GENERATION_FAILED | bitstream 生成失敗 |
| -32221 | BITSTREAM_INVALID | 生成された bitstream 不正 |

### Timing（タイミング）

| コード | 名称 | 説明 |
|-------|------|------|
| -32230 | TIMING_VIOLATION | タイミング違反（WNS < 0 / TNS < 0） |
| -32231 | TIMING_ANALYSIS_FAILED | タイミング解析失敗 |

### Debug / On-Chip

| コード | 名称 | 説明 |
|-------|------|------|
| -32240 | DEBUG_SESSION_FAILED | オンチップデバッグセッション失敗 |
| -32241 | ILA_CONFIGURATION_ERROR | ILA 設定エラー |

### HLS

| コード | 名称 | 説明 |
|-------|------|------|
| -32245 | HLS_COMPILE_FAILED | HLS コンパイル失敗 |

### Device（デバイス）

| コード | 名称 | 説明 |
|-------|------|------|
| -32250 | DEVICE_NOT_FOUND | ターゲットデバイス未検出 |
| -32251 | DEVICE_PROGRAM_FAILED | デバイスプログラミング失敗 |
| -32252 | DEVICE_COMPATIBILITY_ERROR | デバイス互換性エラー |

### Simulation

| コード | 名称 | 説明 |
|-------|------|------|
| -32255 | SIMULATION_FAILED | シミュレーション失敗 |
| -32256 | SIMULATION_TIMEOUT | シミュレーションタイムアウト |

### Constraints（制約）

| コード | 名称 | 説明 |
|-------|------|------|
| -32260 | CONSTRAINT_PARSE_ERROR | 制約ファイル（XDC/SDC/PCF）パースエラー |
| -32261 | CONSTRAINT_CONVERSION_ERROR | 制約形式変換エラー（XDC ⇔ SDC ⇔ PCF） |

### Adapter

| コード | 名称 | 説明 |
|-------|------|------|
| -32270 | ADAPTER_NOT_FOUND | 指定されたアダプターが未登録 |
| -32271 | ADAPTER_VERSION_MISMATCH | アダプターの API バージョン不整合 |
| -32272 | ADAPTER_MANIFEST_INVALID | アダプターマニフェスト不正 |

## エラー応答フォーマット

```json
{
  "error": {
    "code": -32230,
    "message": "Timing violation detected",
    "data": {
      "tool": "vivado",
      "exit_code": 1,
      "log_path": "/workspace/build/vivado_synth.log",
      "errors": [
        { "wns": -0.5, "tns": -3.2, "path": "clk -> ff1" }
      ],
      "retry_possible": true,
      "suggested_action": "Add pipeline registers or relax timing constraints"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123"
}
```

## 関連ドキュメント

- [fpga/message_methods.md](message_methods.md) — fpga.* メソッド一覧
- [fpga/state_machines.md](state_machines.md) — ビルドステートマシン
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter トレイト
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ