# rtl-conductor メッセージメソッド一覧

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §4.7（1332-1334行目付近）, §14（3492-3630行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `rtl`。ペイロードは構造化 JSON（先頭 `{`）または自然言語テキスト。

## rtl.* メソッド一覧

### Lint

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rtl.lint.v1` | Request | HDL ソースの Lint / フォーマット / 静的解析を実行 |
| `rtl.lint.v1.format` | Request | コードフォーマット実行（Verible 等） |

### シミュレーション

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rtl.simulate.v1` | Request | シミュレーション実行（テストベンチ指定、サイクル精度 / 動作シミュレーション） |

### 形式検証

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rtl.formal.v1` | Request | 形式検証実行（プロパティベース、SymbiYosys） |

### トランスパイル

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rtl.transpile.v1` | Request | HDL 言語間トランスパイル（Chisel/SpinalHDL/Amaranth → Verilog/VHDL） |

### ハンドオフ

| メソッド | 方向 | 説明 |
|---------|------|------|
| `rtl.handoff.v1` | Request | 下流 conductor（fpga / asic / hal）への成果物引き渡し |

## conductor-core 共通 API

rtl-conductor も `ConductorRpc` トレイトの共通メソッドを実装する。

| メソッド | 説明 |
|---------|------|
| `system.health.v1` | ヘルスチェック（Online / Offline / Degraded / Upgrading） |
| `system.readiness` | 準備状態確認 |

## ペイロード例

### rtl.lint.v1 リクエスト

```json
{
  "method": "rtl.lint.v1",
  "params": {
    "project": "core_v",
    "adapter": "verilator-lint",
    "flags": ["--warn-no-UNDRIVEN"]
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### rtl.handoff.v1 リクエスト

```json
{
  "method": "rtl.handoff.v1",
  "params": {
    "target": "fpga",
    "artifacts": ["build/synth_ready.sv"]
  },
  "id": "msg_2026-05-01T12:00:00Z_def456",
  "trace_id": "trace_xyz789"
}
```

## 関連ドキュメント

- [rtl/binary_spec.md](binary_spec.md) — hestia-rtl-cli バイナリ仕様
- [rtl/error_types.md](error_types.md) — RTL 固有エラー型
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter トレイト
- [rtl/handoff.md](handoff.md) — 下流連携ハンドオフ
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様