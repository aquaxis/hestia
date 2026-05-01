# asic-conductor メッセージメソッド一覧

**対象 Conductor**: asic-conductor
**ソース**: 設計仕様書 §14（3492-3630行目付近）, §6（1761-1981行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `asic`。

## asic.* メソッド一覧

### RTL-to-GDSII フロー

| メソッド | 方向 | 説明 |
|---------|------|------|
| `asic.synthesize` | Request | 論理合成実行（Yosys） |
| `asic.floorplan` | Request | フロアプラン作成（OpenROAD） |
| `asic.place` | Request | セル配置実行（RePlAce + OpenDP） |
| `asic.cts` | Request | クロックツリー合成（TritonCTS） |
| `asic.route` | Request | 配線実行（FastRoute + TritonRoute） |
| `asic.gdsii` | Request | GDSII ストリーム生成 |

### サインオフ

| メソッド | 方向 | 説明 |
|---------|------|------|
| `asic.drc` | Request | デザインルールチェック（Magic / KLayout） |
| `asic.lvs` | Request | レイアウト対回路図検証（Netgen / KLayout） |
| `asic.timing_signoff` | Request | タイミングサインオフ（OpenSTA） |

### PDK 管理

| メソッド | 方向 | 説明 |
|---------|------|------|
| `asic.pdk.install` | Request | PDK インストール（volare 経由） |
| `asic.pdk.list` | Request | インストール済み PDK 一覧 |

### AI エージェント連携

| メソッド | 方向 | 説明 |
|---------|------|------|
| `asic.ai.timing_fix` | Request | タイミング違反自動修復提案 |
| `asic.ai.drc_fix` | Request | DRC 違反自動修復パッチ生成 |
| `asic.ai.floorplan_optimize` | Request | フロアプラン最適化提案 |
| `asic.ai.pdk_migrate` | Request | PDK マイグレーション支援 |

### conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |
| `system.readiness` | Request | 準備状態確認 |

## ペイロード例

### asic.synthesize リクエスト

```json
{
  "method": "asic.synthesize",
  "params": {
    "pdk": "sky130_fd_sc_hd",
    "strategy": "area",
    "abc_script": "resyn2"
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### asic.drc リクエスト

```json
{
  "method": "asic.drc",
  "params": {
    "tool": "magic",
    "gds_path": "build/results/final.gds"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

## 関連ドキュメント

- [asic/binary_spec.md](binary_spec.md) — hestia-asic-cli バイナリ仕様
- [asic/error_types.md](error_types.md) — asic-conductor エラーコード
- [asic/state_machines.md](state_machines.md) — ASIC ビルドステートマシン
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter トレイト
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様