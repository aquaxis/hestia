# VSCode 拡張

**対象領域**: frontend — VSCode 統合
**ソース**: 設計仕様書 §16.1

## 概要

Hestia の VSCode 拡張機能。TypeScript で実装され、Monaco Editor 統合、HDL LSP、波形ビューア、conductor 管理を VSCode 内で提供する。

## パッケージ情報

| 項目 | 値 |
|------|-----|
| パッケージ名 | `hestia-vscode` |
| パブリッシャー | `aquaxis` |
| VSCode バージョン | >= 1.85.0 |

## アクティベーション

### onCommand トリガー

30+ のコマンドが登録されている。主なもの:

- `hestia.start` / `hestia.stop` / `hestia.status`
- `hestia.ai` / `hestia.spec` / `hestia.fpga` / `hestia.debug` / `hestia.rag`

### onView トリガー

- `hestia-conductor`
- `agents`
- `specs`

### onLanguage トリガー

- `verilog` / `vhdl` / `systemverilog` / `xdc` / `pcf` / `toml`

## ビュー（5 種）

| ビュー | 用途 |
|-------|------|
| `ConductorStatusView` | 全 conductor のステータス一覧・操作 |
| `AgentListView` | サブエージェント一覧・管理 |
| `SpecViewer` | 仕様書の構造化表示・編集 |
| `DesignFlowView` | 設計フロー（DAG）の可視化 |
| `LogViewer` | リアルタイムログストリーム |

## Monaco Editor 統合

- HDL シンタックスハイライト（Verilog / SystemVerilog / VHDL）
- コード補完（HDL LSP Broker 経由、§13.1）
- 診断情報のインライン表示
- Go to Definition / Find References / Rename（LSP 機能）

## 波形ビューア（WebView）

- VSCode WebView 内で WASM レンダリング
- `waveform-core` クレートを WASM にコンパイルして使用（§13.2）
- WebWorker + SharedArrayBuffer でパフォーマンス確保
- VCD / FST / GHW / EVCD 対応

## agent-cli IPC

- `conductor-sdk` の `AgentCliClient`（TypeScript 版）を使用
- `agent-cli list` / `agent-cli send` を spawn またはネイティブバインディング経由で呼出
- `ConductorId = 'ai' | 'rtl' | 'fpga' | 'asic' | 'pcb' | 'hal' | 'apps' | 'debug' | 'rag'`

## 設定スキーマ

`hestia.*` 設定一覧は `config_schema.md` を参照。

## 関連ドキュメント

- [config_schema.md](config_schema.md) — VSCode 設定スキーマ
- [agent_cli_client.md](agent_cli_client.md) — agent-cli クライアント仕様
- [ui_components.md](ui_components.md) — UI コンポーネントライブラリ
- [hdl_lsp_broker.md](../common/hdl_lsp_broker.md) — HDL LSP Broker
- [wasm_waveform_viewer.md](../common/wasm_waveform_viewer.md) — WASM 波形ビューア