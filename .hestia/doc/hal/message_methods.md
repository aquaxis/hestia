# hal-conductor メッセージメソッド一覧

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §8.7（2260-2262行目付近）, §14（3492-3630行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `hal`。

## hal.* メソッド一覧

### レジスタマップ操作

| メソッド | 方向 | 説明 |
|---------|------|------|
| `hal.parse.v1` | Request | レジスタ定義ファイルのパース（SystemRDL / IP-XACT / TOML → RegisterMap） |
| `hal.validate.v1` | Request | レジスタマップのバリデーション（アドレス重複・型整合性・バス境界チェック） |
| `hal.generate.v1` | Request | 指定言語のコード生成（C / Rust / Python / Markdown / SVD） |
| `hal.export.v1` | Request | rtl/asic 向けエクスポート（SystemVerilog テンプレート出力） |
| `hal.diff.v1` | Request | レジスタマップ差分表示（2 バージョン間の比較） |

### conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |
| `system.readiness` | Request | 準備状態確認 |

## ペイロード例

### hal.parse.v1 リクエスト

```json
{
  "method": "hal.parse.v1",
  "params": {
    "input_format": "systemrdl",
    "sources": ["regs/core.rdl", "regs/peripheral.rdl"]
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### hal.generate.v1 リクエスト

```json
{
  "method": "hal.generate.v1",
  "params": {
    "target_lang": "rust",
    "output_path": "build/hal/rust/soc-hal"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

### hal.diff.v1 リクエスト

```json
{
  "method": "hal.diff.v1",
  "params": {
    "baseline": "build/hal/v1.0/registers.json",
    "current": "build/hal/v1.1/registers.json"
  },
  "id": "msg_2026-05-01T12:00:00Z_ghi789"
}
```

## 上流・下流連携

- **上流（rtl-conductor §4）**: rtl-conductor が定義したバスインターフェース宣言を入力に取り、レジスタマップを SystemRDL 形式でエクスポート可能。`hal.handoff` イベントで rtl-conductor からトリガーされる
- **下流（apps-conductor §9）**: 生成された C ヘッダ / Rust crate / Python モジュールを apps-conductor の `[hal] import = "..."` で取り込む
- **横断（debug-conductor §10）**: 同一レジスタマップを debug-conductor が再利用し、ライブデバッグ時のレジスタ表示・編集 UI に活用
- **横断（asic/fpga conductor）**: レジスタブロックの SystemVerilog テンプレート出力を、対応する conductor の `[sources]` に直接渡せる

## 関連ドキュメント

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli バイナリ仕様
- [hal/register_map.md](register_map.md) — レジスタマップ定義
- [hal/codegen.md](codegen.md) — 多言語コード生成
- [hal/state_machines.md](state_machines.md) — ビルドステートマシン
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様