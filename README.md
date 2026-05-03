# Hestia

[![CI](https://github.com/aquaxis/hestia/actions/workflows/ci.yml/badge.svg)](https://github.com/aquaxis/hestia/actions/workflows/ci.yml)

## Hardware Engineering Stack for Tool Integration and Automation

Hestia（ヘスティア）は、FPGA・ASIC・PCB・HAL・組込みソフトウェア開発ツールを統合する AI 駆動のハードウェア開発環境です。9 つの Conductor（ドメイン特化 AI エージェント）が agent-cli IPC で連携し、仕様書からの設計自動生成・ベンダーツールの統一的オーケストレーション・コンテナによる再現性のあるビルドを実現します。

**日本語** | [English](./README_en.md) | [Workflow Reference](./WORKFLOWS.md)

## 特徴

- **9 Conductor アーキテクチャ** — RTL・FPGA・ASIC・PCB・HAL・Apps・Debug・RAG のドメイン特化 AI エージェント
- **統一 IPC** — 全 Conductor 間通信を agent-cli ネイティブ IPC で統一（`agent-cli send <peer> <payload>`）
- **仕様書駆動開発** — 自然言語仕様書から HDL コード・制約ファイル・テストベンチを自動生成
- **ベンダー非依存の抽象化** — `ToolAdapter`/`VendorAdapter` トレイトによる統一インターフェース。`adapter.toml` を書くだけでツール追加可能
- **コンテナ & ローカル実行** — Podman rootless コンテナまたはローカル実行を選択可能。lock ファイルによるビルド再現性
- **AI エージェントパイプライン** — WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent によるツールバージョンアップ自動追従

## アーキテクチャ

```text
                    ┌─────────────────────────────────────┐
                    │          hestia (CLI ランナー)         │
                    └──────────────┬──────────────────────┘
                                   │
                    ┌──────────────▼──────────────────────┐
                    │        ai-conductor（メタオーケストレーター） │
                    │  ConductorManager │ WorkflowEngine   │
                    │  SpecDriven      │ SkillSystem       │
                    │  ContainerMgr    │ UpgradeManager    │
                    └──────────────┬──────────────────────┘
                                   │ agent-cli IPC
          ┌────────┬────────┬─────┼─────┬────────┬────────┐
          │        │        │     │     │        │        │
   ┌──────▼──┐ ┌──▼───┐ ┌─▼──┐ ┌▼──┐ ┌▼──────┐ ┌▼──────┐
   │  RTL    │ │ FPGA │ │ASIC│ │PCB│ │  HAL   │ │ Apps  │
   │ Cond.   │ │ Cond.│ │C. │ │C. │ │  Cond. │ │ Cond. │
   └─────────┘ └──────┘ └────┘ └───┘ └────────┘ └───────┘
   ┌────────┐ ┌──────┐                                     フロントエンド
   │ Debug  │ │ RAG  │    共有サービス層                    ┌──────────┐
   │ Cond.   │ │ Cond.│    hdl-lsp-broker  waveform-core    │  VSCode   │
   └────────┘ └──────┘    constraint-bridge  ip-manager    │  hestia-  │
                              cicd-api  observability         │  ui       │
                              hestia-mcp-server               │ Tauri IDE │
                                                               └──────────┘
```

## クイックスタート

### ワンライナーインストール

```bash
curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh
```

カスタムプレフィックスへのインストール:

```bash
curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh -s -- --prefix ~/.local/bin
```

### ソースからビルド

```bash
git clone https://github.com/AQUAXIS/hestia.git
cd hestia/.hestia/tools
make build
make install PREFIX=~/.local/bin
```

### プロジェクトの初期化

```bash
hestia init          # .hestia/ ディレクトリ構造を作成
hestia start         # 全 Conductor デーモンを起動
hestia status        # デーモンステータスを表示
```

## 動作要件

- **Rust** 1.75+（[rustup](https://rustup.rs) でインストール）
- **Linux** x86_64（カーネル 5.x 以降）
- **agent-cli**（Conductor 間の IPC 通信に使用）

## ワークスペース構成

```text
.hestia/tools/
├── Cargo.toml                  # ワークスペースルート (resolver = "2")
├── conductors/                 # 9 Conductor デーモン
│   ├── hestia-ai-conductor/     # メタオーケストレーター
│   ├── hestia-rtl-conductor/    # RTL 設計フロー
│   ├── hestia-fpga-conductor/   # FPGA 設計フロー
│   ├── hestia-asic-conductor/   # ASIC 設計フロー
│   ├── hestia-pcb-conductor/    # PCB 設計フロー
│   ├── hestia-hal-conductor/    # HAL コード生成
│   ├── hestia-apps-conductor/   # 組込みソフトウェア開発
│   ├── hestia-debug-conductor/  # デバッグ環境
│   └── hestia-rag-conductor/    # ナレッジ検索
├── clis/                       # 10 CLI バイナリ
│   ├── hestia/                  # 統合ランナー
│   └── hestia-{domain}-cli/    # ドメイン別 CLI
├── crates/                     # 共通・共有クレート
│   ├── conductor-sdk/           # トランスポート / メッセージ / エージェント / 設定
│   ├── adapter-core/            # ToolAdapter / VendorAdapter トレイト
│   ├── project-model/           # TOML パーサー / 設定モデル
│   ├── hdl-lsp-broker/          # HDL LSP プロキシ (svls / vhdl_ls / verilog-ams-ls)
│   ├── waveform-core/          # VCD / FST / GHW / EVCD パーサー (WASM + ネイティブ)
│   ├── constraint-bridge/      # XDC / SDC / PCF / Efinity XML / QSF / UCF 変換
│   ├── ip-manager/              # IP コア登録・DAG 依存解決
│   ├── cicd-api/                # CI/CD パイプライン (GitHub / GitLab / Local)
│   ├── observability/           # Prometheus + tracing + OTLP
│   └── hestia-mcp-server/       # MCP サーバー (LLM Tool Use)
└── packages/                    # フロントエンド
    ├── hestia-ui/                # React コンポーネントライブラリ
    ├── hestia-vscode/            # VSCode 拡張
    └── hestia-ide/               # Tauri デスクトップ IDE
```

## Conductor 一覧

| Conductor | ドメイン | 説明 |
| --------- | -------- | ---- |
| **ai** | メタオーケストレーション | 全 Conductor・AI エージェント・コンテナ・ワークフローを管理 |
| **rtl** | RTL 設計 | Lint・シミュレーション・形式検証・トランスパイル・ハンドオフ |
| **fpga** | FPGA | Vivado / Quartus / Efinity ビルド・合成・ビットストリーム生成 |
| **asic** | ASIC | OpenLane / Yosys / OpenROAD・PDK 管理 |
| **pcb** | PCB | KiCad 回路図・レイアウト・AI 合成・DRC/ERC |
| **hal** | HAL | レジスタマップ・コード生成 (C/Rust/Python/SVD)・バスプロトコル |
| **apps** | 組込みソフトウェア | ツールチェーン・RTOS・HIL/SIL・フラッシュ・デバッグ |
| **debug** | デバッグ | JTAG / SWD / ILA・波形キャプチャ・プロトコル解析 |
| **rag** | ナレッジ検索 | ベクトル検索・埋め込み・引用・6 サブエージェント |

## CLI 使用例

```bash
# 統合ランナー
hestia init                    # プロジェクトを初期化
hestia start fpga              # FPGA Conductor を起動
hestia status                  # 全 Conductor のステータスを表示
hestia ai -- exec "review"     # ai-cli にディスパッチ

# ドメイン別 CLI
hestia-fpga-cli init           # FPGA プロジェクトを初期化
hestia-fpga-cli build          # FPGA プロジェクトをビルド
hestia-rtl-cli lint            # RTL ソースをリント
hestia-asic-cli pdk install   # PDK をインストール
hestia-pcb-cli drc             # DRC を実行
hestia-hal-cli generate        # HAL コードを生成
hestia-apps-cli flash          # ファームウェアをフラッシュ
hestia-debug-cli capture       # 波形をキャプチャ
hestia-rag-cli search "FIFO"  # ナレッジベースを検索
```

## ビルドターゲット

```bash
make build          # リリースビルド（全19バイナリ）
make test           # テスト実行
make lint           # clippy 実行
make fmt            # フォーマットチェック
make install        # ~/.local/bin にインストール（デフォルト）
make install PREFIX=/usr/local/bin  # システム全体にインストール
make clean          # ビルド成果物を削除
```

## 設計原則

1. **置き換えではなく抽象化** — ベンダーツールはそのままに、統一インターフェースでオーケストレート
2. **ゼロ変更での拡張** — `adapter.toml` を書くだけでツール追加。Rust コードの変更不要
3. **持続可能な維持管理** — AI エージェントがバージョンアップ対応を自動化
4. **セキュリティ** — Podman rootless によるコンテナ隔離、API キーの環境変数経由管理
5. **再現性** — lock ファイルによるビルドの完全再現性
6. **メーカー非依存** — OSS ツール優先、プラグインシステムで任意のベンダーツールを統合
7. **AI 活用** — 仕様書駆動開発・Tool Use による設計プロセス全体の AI 支援
8. **統一インターフェース** — 全通信を agent-cli IPC に統一

## ライセンス

MIT OR Apache-2.0
