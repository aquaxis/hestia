# HDL LSP Broker

**対象領域**: common — HDL 開発支援
**ソース**: 設計仕様書 §13.1

## 概要

Verilog / SystemVerilog / VHDL / Verilog-AMS の LSP サーバ群を統一インターフェースで提供する LSP プロキシ。フロントエンド（VSCode 拡張 / Tauri IDE）からは単一接続で複数言語の補完・診断・ジャンプ・参照・リネームを利用できる。agent-cli peer `lsp` として提供される。

## 対応 LSP サーバ

| LSP サーバ | バージョン | 対応言語 |
|-----------|----------|---------|
| svls | v0.2.x | SystemVerilog |
| vhdl_ls | v0.3.x | VHDL |
| verilog-ams-ls | v0.1.x | Verilog-AMS |

## 主要型

### HdlLanguage

```rust
pub enum HdlLanguage {
    Verilog,
    SystemVerilog,
    Vhdl,
    VerilogAms,
}
```

### LspServerConfig

LSP サーバごとの起動設定。

### RoutingTable

拡張子 → LSP サーバのルーティングテーブル。

## 拡張子マップ

| 拡張子 | 言語 | ルーティング先 |
|--------|------|-------------|
| `.v` | Verilog | svls（Verilog モード）|
| `.sv` / `.svh` | SystemVerilog | svls |
| `.vhd` / `.vhdl` | VHDL | vhdl_ls |
| `.va` / `.vams` | Verilog-AMS | verilog-ams-ls |

## パラメータ既定値

| パラメータ | 既定値 | 説明 |
|----------|-------|------|
| `max_instances` | 4 | 同時起動可能な LSP サーバインスタンス数 |
| `idle_timeout_secs` | 300 | アイドル時の自動終了タイムアウト |

## 動作フロー

```
[VSCode / Tauri IDE] → 単一 LSP 接続 → HDL LSP Broker
                                              │
                                              ├── 拡張子判定 → HdlLanguage
                                              ├── RoutingTable で LSP サーバ選択
                                              ├── サーバ未起動なら起動（max_instances 制限内）
                                              └── LSP リクエスト転送 → 応答返却
```

## 統合先

- VSCode 拡張: Monaco Editor に HDL ハイライト・補完・診断を統合（§16.1）
- Tauri IDE: 同エディタ機能を提供

## 関連ドキュメント

- [wasm_waveform_viewer.md](wasm_waveform_viewer.md) — WASM 波形ビューア
- [constraint_bridge.md](constraint_bridge.md) — 制約ファイル変換
- [observability.md](observability.md) — 監視