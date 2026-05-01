# Rust ワークスペース構成

**対象領域**: common — ビルド構成
**ソース**: 設計仕様書 §15.4, §1.6, §2.1

## 概要

HESTIA の Rust プロジェクトは `.hestia/tools/Cargo.toml` をルートとする Cargo ワークスペースで構成される。resolver = "2" を採用し、9 conductor デーモン + 10 CLI バイナリを一元管理する。

## ワークスペース構成

```
.hestia/tools/
├── Cargo.toml                    # ワークスペースルート（resolver = "2"）
├── conductors/                   # Rust デーモン x9
│   ├── hestia-ai-conductor/
│   ├── hestia-rtl-conductor/
│   ├── hestia-fpga-conductor/
│   ├── hestia-asic-conductor/
│   ├── hestia-pcb-conductor/
│   ├── hestia-hal-conductor/
│   ├── hestia-apps-conductor/
│   ├── hestia-debug-conductor/
│   └── hestia-rag-conductor/
├── clis/                         # CLI x10
│   ├── hestia/                   # 統合ランナー
│   ├── hestia-ai-cli/
│   ├── hestia-rtl-cli/
│   ├── hestia-fpga-cli/
│   ├── hestia-asic-cli/
│   ├── hestia-pcb-cli/
│   ├── hestia-hal-cli/
│   ├── hestia-apps-cli/
│   ├── hestia-debug-cli/
│   └── hestia-rag-cli/
└── crates/                       # 共通クレート
    ├── conductor-sdk/            # transport / message / agent / config / error
    ├── adapter-core/             # ToolAdapter / VendorAdapter トレイト
    ├── hestia-mcp-server/        # MCP サーバー
    └── project-model/            # TOML パーサー・設定モデル
```

## バイナリ一覧（19 バイナリ）

### Conductor デーモン（9）

| バイナリ | 対応 conductor |
|---------|---------------|
| `hestia-ai-conductor` | ai-conductor |
| `hestia-rtl-conductor` | rtl-conductor |
| `hestia-fpga-conductor` | fpga-conductor |
| `hestia-asic-conductor` | asic-conductor |
| `hestia-pcb-conductor` | pcb-conductor |
| `hestia-hal-conductor` | hal-conductor |
| `hestia-apps-conductor` | apps-conductor |
| `hestia-debug-conductor` | debug-conductor |
| `hestia-rag-conductor` | rag-conductor |

### CLI クライアント（10）

| バイナリ | 主要サブコマンド |
|---------|----------------|
| `hestia` | `init` / `start [domain]` / `status` / `ai` / `rtl` / `fpga` / `asic` / `pcb` / `hal` / `apps` / `debug` / `rag` |
| `hestia-ai-cli` | `exec` / `run --file` / `agent ls` / `container ls|start|stop|create` / `workflow run` |
| `hestia-rtl-cli` | `init` / `lint` / `simulate` / `formal` / `transpile` / `handoff` / `status` |
| `hestia-fpga-cli` | `init` / `build` / `synthesize` / `implement` / `bitstream` / `simulate` / `program` / `report` |
| `hestia-asic-cli` | `init` / `build` / `pdk install|list` / `advance` / `drc` / `lvs` / `status` |
| `hestia-pcb-cli` | `init` / `build` / `ai-synthesize` / `output` / `drc` / `erc` / `status` |
| `hestia-hal-cli` | `init` / `parse` / `validate` / `generate` / `export-rtl` / `diff` / `status` |
| `hestia-apps-cli` | `init` / `build` / `flash` / `test` / `size` / `debug` / `status` |
| `hestia-debug-cli` | `create` / `connect` / `disconnect` / `program` / `capture` / `signals` / `trigger` / `reset` |
| `hestia-rag-cli` | `ingest` / `search` / `cleanup` / `status` |

## 共通依存

| クレート | 用途 |
|---------|------|
| `tokio` | 非同期ランタイム（multi_thread, 4 workers）|
| `serde` | TOML/JSON シリアライゼーション |
| `tracing` | 構造化ログ |
| `thiserror` / `anyhow` | エラー処理（ライブラリ / バイナリ使い分け）|
| `clap` | CLI パーサー |
| `sled` | Rust ネイティブ KV ストア |
| `minijinja` | テンプレートエンジン |
| `petgraph` | DAG 解決 |

## ビルドコマンド

```bash
cd .hestia/tools
cargo build --release                                    # 全バイナリ
cargo build --release -p hestia-fpga-conductor           # 特定 conductor
cargo test                                               # 全テスト
cargo test -p hestia-fpga-conductor                      # 特定 conductor
cargo test -p container-manager                          # 特定クレート
```

## 関連ドキュメント

- [installation.md](installation.md) — ビルド手順詳細
- [error_handling_strategy.md](error_handling_strategy.md) — エラー処理戦略
- [conductor_startup.md](conductor_startup.md) — デーモン起動順序