# asic-conductor エラーコード

**対象 Conductor**: asic-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）

## エラーコード範囲

asic-conductor のエラーコードは **-32300 〜 -32399** の範囲を使用する。

## エラーカテゴリ

### RTL Synthesis（論理合成）

| コード | 名称 | 説明 |
|-------|------|------|
| -32300 | SYNTHESIS_FAILED | Yosys 論理合成失敗 |
| -32301 | SYNTHESIS_TIMEOUT | 合成タイムアウト |
| -32302 | RTL_READ_ERROR | RTL 読み込みエラー |
| -32303 | TECH_MAPPING_FAILED | テクノロジマッピング失敗（ABC） |

### Floorplan（フロアプラン）

| コード | 名称 | 説明 |
|-------|------|------|
| -32310 | FLOORPLAN_FAILED | フロアプラン作成失敗 |
| -32311 | PDN_GENERATION_FAILED | 電源分配網生成失敗 |
| -32312 | IO_PLACEMENT_FAILED | I/O ピン配置失敗 |
| -32313 | MACRO_PLACEMENT_FAILED | マクロ配置失敗 |

### Placement（配置）

| コード | 名称 | 説明 |
|-------|------|------|
| -32320 | PLACEMENT_FAILED | セル配置失敗 |
| -32321 | DENSITY_EXCEEDED | 配置密度超過 |
| -32322 | OVERFLOW_ERROR | 配置オーバーフロー |

### CTS（クロックツリー合成）

| コード | 名称 | 説明 |
|-------|------|------|
| -32330 | CTS_FAILED | クロックツリー合成失敗 |
| -32331 | CTS_SKEW_VIOLATION | スキューバイオレーション |
| -32332 | BUFFER_INSERTION_FAILED | バッファ挿入失敗 |

### Routing（配線）

| コード | 名称 | 説明 |
|-------|------|------|
| -32340 | GLOBAL_ROUTING_FAILED | グローバルルーティング失敗 |
| -32341 | DETAILED_ROUTING_FAILED | 詳細ルーティング失敗 |
| -32342 | CONGESTION_DETECTED | 配線混雑度検出 |
| -32343 | DRC_VIOLATION_ROUTING | 配線DRC 違反 |

### Extraction / Timing

| コード | 名称 | 説明 |
|-------|------|------|
| -32350 | EXTRACTION_FAILED | 寄生抽出失敗（OpenRCX） |
| -32351 | TIMING_SIGNOFF_FAILED | タイミングサインオフ失敗（WNS < 0） |

### DRC / LVS / Signoff

| コード | 名称 | 説明 |
|-------|------|------|
| -32360 | DRC_FAILED | DRC チェック失敗（Magic / KLayout） |
| -32361 | LVS_FAILED | LVS チェック失敗（Netgen） |
| -32362 | DRC_VIOLATIONS_FOUND | DRC 違反検出 |
| -32363 | LVS_MISMATCH_FOUND | LVS ミスマッチ検出 |

### GDSII / PDK

| コード | 名称 | 説明 |
|-------|------|------|
| -32370 | GDSII_GENERATION_FAILED | GDSII ストリーム生成失敗 |
| -32371 | PDK_NOT_INSTALLED | PDK 未インストール |
| -32372 | PDK_VERSION_MISMATCH | PDK バージョン不整合 |

## エラー応答フォーマット

```json
{
  "error": {
    "code": -32351,
    "message": "Timing signoff failed",
    "data": {
      "tool": "opensta",
      "exit_code": 1,
      "log_path": "/workspace/build/timing.log",
      "errors": [
        { "wns": -0.3, "tns": -1.5, "path": "clk -> data_out" }
      ],
      "retry_possible": true,
      "suggested_action": "Add buffer insertion or relax clock period"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123"
}
```

## 関連ドキュメント

- [asic/message_methods.md](message_methods.md) — asic.* メソッド一覧
- [asic/state_machines.md](state_machines.md) — ASIC ビルドステートマシン
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter トレイト
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ