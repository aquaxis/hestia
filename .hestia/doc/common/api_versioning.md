# メソッド名前空間・バージョニング

**対象領域**: common — API バージョニング
**ソース**: 設計仕様書 §14.2

## 概要

agent-cli IPC 上の構造化メッセージのメソッド名は、ドメイン x 機能 x バージョンの3階層で命名される。後方互換性と段階的廃止を保証するため、セマンティックバージョニングに基づくバージョンプレフィックスを採用する。

## 命名規則

```
{domain}.{method_group}.{version_prefix}.{action}
```

- 例: `fpga.build.v1.synthesize`
- 簡略形 `{domain}.{action}` も同義（v1 既定）

## 主要型

### ApiVersion

```rust
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}
```

### MethodGroup

```rust
pub struct MethodGroup {
    pub domain: String,
    pub group: String,
    pub current_version: ApiVersion,
    pub supported_versions: Vec<ApiVersion>,
    pub deprecated_versions: Vec<ApiVersion>,
    pub methods: Vec<String>,
}
```

### DeprecationNotice

```rust
pub struct DeprecationNotice {
    pub deprecated_since: ApiVersion,
    pub removal_scheduled: ApiVersion,
    pub replacement: String,
}
```

## 互換性規則

| 変更種別 | バージョン影響 | 後方互換 |
|---------|-------------|---------|
| 必須パラメータ追加 | major バンプ | 不可 |
| 既存パラメータ型変更 | major バンプ | 不可 |
| メソッド削除 | major バンプ | 不可 |
| 任意パラメータ追加 | minor バンプ | 可能 |
| 応答フィールド追加 | minor バンプ | 可能 |

## ドメイン一覧

| ドメイン | 例 |
|---------|----|
| `ai.*` | `ai.spec.init` / `ai.spec.update` / `ai.spec.review` / `ai.exec` / `agent_spawn` / `agent_list` |
| `fpga.*` | `fpga.synthesize` / `fpga.implement` / `fpga.bitstream` / `fpga.simulate` / `fpga.program` |
| `asic.*` | `asic.synthesize` / `asic.floorplan` / `asic.place` / `asic.cts` / `asic.route` / `asic.gdsii` / `asic.drc` / `asic.lvs` |
| `pcb.*` | `pcb.generate_schematic` / `pcb.run_drc` / `pcb.run_erc` / `pcb.generate_bom` / `pcb.place_components` / `pcb.route_traces` / `pcb.generate_output` / `pcb.ai_synthesize` / `pcb.status` |
| `debug.*` | `debug.connect` / `debug.disconnect` / `debug.program` / `debug.start_capture` / `debug.stop_capture` / `debug.read_signals` / `debug.set_trigger` / `debug.reset` / `debug.status` |
| `rag.*` | `rag.ingest` / `rag.search` / `rag.cleanup` / `rag.status` |
| `meta.*` | `meta.dualBuild` / `meta.boardWithFpga` ほかクロス Conductor ワークフロー |
| `system.*` | `system.readiness` / `system.health` / `system.shutdown` |

## 廃止予告フロー

1. 新バージョンのメソッドを追加
2. 旧メソッドに `DeprecationNotice` を付与
3. 移行期間中は旧メソッドも動作（警告ログ出力）
4. `removal_scheduled` バージョン到達時に旧メソッドを削除

## 関連ドキュメント

- [agent_message.md](agent_message.md) — メッセージペイロード形式
- [error_registry.md](error_registry.md) — エラーコード規約
- [agent_cli_messaging.md](agent_cli_messaging.md) — 完全なメッセージング仕様