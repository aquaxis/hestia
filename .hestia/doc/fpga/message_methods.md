# fpga-conductor メッセージメソッド一覧

**対象 Conductor**: fpga-conductor
**ソース**: 設計仕様書 §14（3492-3630行目付近）, §5（1398-1760行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `fpga`。

## fpga.* メソッド一覧

### ビルドフロー

| メソッド | 方向 | 説明 |
|---------|------|------|
| `fpga.synthesize` | Request | RTL 合成（adapter.synthesize 呼出） |
| `fpga.implement` | Request | 配置配線（adapter.implement 呼出） |
| `fpga.bitstream` | Request | bitstream 生成（adapter.generate_bitstream 呼出） |
| `fpga.simulate` | Request | シミュレーション実行 |

### プログラミング

| メソッド | 方向 | 説明 |
|---------|------|------|
| `fpga.program` | Request | FPGA へ bitstream 書込（debug-conductor §10 連携） |

### レポート

| メソッド | 方向 | 説明 |
|---------|------|------|
| `fpga.build.v1.start` | Request | フルビルド開始（target 指定） |
| `fpga.build.v1.cancel` | Request | ビルドキャンセル |
| `fpga.build.v1.status` | Request | ビルド状況取得 |

### conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |
| `system.readiness` | Request | 準備状態確認 |
| `project_open` | Request | プロジェクトオープン |
| `project_targets` | Request | ターゲット一覧 |
| `report_timing` | Request | タイミングレポート |
| `report_resource` | Request | リソースレポート |
| `report_messages` | Request | ビルドメッセージ一覧 |

## ペイロード例

### fpga.build.v1.start リクエスト

```json
{
  "method": "fpga.build.v1.start",
  "params": {
    "target": "artix7_dev",
    "steps": ["synthesize", "implement", "bitstream"]
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### system.health.v1 応答

```json
{
  "result": {
    "status": "online",
    "uptime_secs": 12345,
    "tools_ready": ["vivado", "yosys"],
    "load": { "cpu_pct": 12, "mem_mb": 512 },
    "active_jobs": 3,
    "last_error": null
  },
  "id": "hc_2026-05-01T12:00:00Z_abc123"
}
```

## 関連ドキュメント

- [fpga/binary_spec.md](binary_spec.md) — hestia-fpga-cli バイナリ仕様
- [fpga/error_types.md](error_types.md) — fpga-conductor エラーコード
- [fpga/state_machines.md](state_machines.md) — ビルドステートマシン
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter トレイト
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様