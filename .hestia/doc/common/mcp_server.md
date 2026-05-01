# MCP サーバー仕様

**対象領域**: common — AI ツール連携
**ソース**: 設計仕様書 §17.2, §13.7.5, §18.9

## 概要

MCP（Model Context Protocol）サーバーは、LLM から外部ツールを呼び出すための標準化されたインターフェースを提供する。HESTIA では共有サービス層の peer `mcp` として実装され、AI エージェント（agent-cli プロセス）が各種ツールを LLM の Tool Use 機能経由で呼び出せるようにする。

## アーキテクチャ

```
[agent-cli (LLM)] → Tool Use 要求 → [MCP Server] → ツール実行 → 結果返却
                                              │
                                              ├── HDL LSP Broker
                                              ├── Constraint Bridge
                                              ├── IP Manager
                                              ├── CI/CD API
                                              ├── Observability
                                              ├── RAG (hestia_rag_search)
                                              └── kicad-mcp-python
```

## 提供ツール

| ツール名 | 対象サービス | 機能 |
|---------|------------|------|
| `hestia_rag_search` | rag-conductor | ナレッジベース検索（`rag.search` と同等）|
| `hestia_lsp_diagnostics` | HDL LSP Broker | HDL 診断情報取得 |
| `hestia_constraint_convert` | Constraint Bridge | 制約ファイル変換 |
| `hestia_ip_resolve` | IP Manager | IP 依存関係解決 |
| `hestia_pipeline_run` | CI/CD API | パイプライン実行 |
| `hestia_health_check` | Observability | ヘルスチェック実行 |

## MCP と agent-cli バックエンド切替の違い

| 項目 | MCP サーバー | agent-cli バックエンド切替 |
|------|------------|-------------------------|
| 用途 | AI からの外部ツール呼出 | agent-cli 自身の LLM バックエンド選択 |
| 経路 | Tool Use → MCP Server → ツール | agent-cli → LLM API |
| 設計 | 独立 | §20 `[agent_cli]` |

両者は独立した設計であり、MCP サーバーは「AI が呼び出すツールの経路」、バックエンド切替は「AI 自身の推論エンジンの選択」をそれぞれ担当する。

## 実装クレート

```
hestia-mcp-server/
├── Cargo.toml
└── src/
    ├── lib.rs          # MCP サーバーエントリ
    ├── tools.rs        # ツール定義・ディスパッチ
    └── transport.rs    # MCP プロトコル処理
```

## kicad-mcp-python 連携

KiCad との連携には `kicad-mcp-python`（MIT ライセンス）を使用。PCB conductor が KiCad 操作を MCP 経由で AI に公開する。

## 関連ドキュメント

- [backend_switching.md](backend_switching.md) — LLM バックエンド切替
- [agent_cli_messaging.md](agent_cli_messaging.md) — メッセージング仕様
- [observability.md](observability.md) — 監視