# rtl-conductor 下流連携ハンドオフ

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §4.6（1328-1331行目付近）, §3.3（854-914行目付近）

## 概要

rtl-conductor のビルド完了時に `meta.handoff` イベントを発行し、ai-conductor（§3）が下流ワークフローをトリガーする。ハンドオフ成果物は rtl.toml の `[handoff]` セクションで明示的に指定される。

## ハンドオフ先

| ハンドオフ先 | 対象 conductor | 成果物 | 説明 |
|-------------|--------------|--------|------|
| fpga | fpga-conductor（§5） | 合成可能 RTL（`build/synth_ready.sv` 等） | fpga.toml の `[sources]` に渡す |
| asic | asic-conductor（§6） | 合成可能 RTL（`build/asic_ready.sv` 等） | asic.toml の RTL 入力に渡す |
| hal | hal-conductor（§8） | バス定義（`build/bus_iface.rdl`） | レジスタマップ定義の入力 |

## ハンドオフフロー

```
[rtl-conductor: ビルド完了]
       │
       │ meta.handoff イベント発行
       ▼
[ai-conductor: conductor-router]
       │
       ├── agent-cli send fpga '{"method":"fpga.build.v1.start",...}'
       │
       ├── agent-cli send asic '{"method":"asic.synthesize",...}'
       │
       └── agent-cli send hal '{"method":"hal.parse.v1",...}'
```

## rtl.toml [handoff] セクション

```toml
[handoff]
fpga = ["build/synth_ready.sv"]       # fpga-conductor の [sources] に渡す
asic = ["build/asic_ready.sv"]        # asic-conductor の [sources] に渡す
hal_bus_decl = "build/bus_iface.rdl"  # hal-conductor のバス定義入力
```

## 成果物の前提条件

- 全成果物は rtl-conductor のビルド完了後に生成される
- 合成可能 RTL は Lint およびシミュレーションを通過したものであること
- バス定義ファイル（RDL）は rtl-conductor がバスインターフェース宣言から生成する

## 下流 conductor との連携

### fpga-conductor との連携

rtl-conductor からハンドオフされた RTL は、fpga.toml の `hdl_files` に自動追加される。fpga-conductor は合成可能 RTL としてそのまま使用する。

### asic-conductor との連携

rtl-conductor からハンドオフされた RTL は、asic.toml の `rtl_files` に自動追加される。asic-conductor は Yosys で論理合成を開始する。

### hal-conductor との連携

rtl-conductor が定義したバスインターフェース宣言（RDL 形式）を hal-conductor が入力に取り、レジスタマップを生成する。

## 関連ドキュメント

- [rtl/config_schema.md](config_schema.md) — rtl.toml [handoff] セクション
- [rtl/state_machines.md](state_machines.md) — ビルド完了からハンドオフへの遷移
- [../fpga/config_schema.md](../fpga/config_schema.md) — fpga.toml スキーマ
- [../asic/config_schema.md](../asic/config_schema.md) — asic.toml スキーマ
- [../hal/config_schema.md](../hal/config_schema.md) — hal.toml スキーマ