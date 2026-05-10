# Hestia

[![CI](https://github.com/aquaxis/hestia/actions/workflows/ci.yml/badge.svg)](https://github.com/aquaxis/hestia/actions/workflows/ci.yml)

## Hardware Engineering Stack for Tool Integration and Automation

Hestia（ヘスティア）は、FPGA・ASIC・PCB・HAL・組込みソフトウェア開発ツールを統合する AI 駆動のハードウェア開発環境です。9 つの Conductor（ドメイン特化 AI エージェント）が agent-cli IPC で連携し、仕様書からの設計自動生成・ベンダーツールの統一的オーケストレーション・コンテナによる再現性のあるビルドを実現します。

**日本語** | [English](./README_en.md) | [Workflow Reference](./WORKFLOWS.md)

## 特徴

- **9 Conductor アーキテクチャ** — RTL・FPGA・ASIC・PCB・HAL・Apps・Debug・RAG のドメイン特化 AI エージェント
- **統一 IPC** — 全 Conductor 間通信を agent-cli 互換 IPC で統一（`agent-cli send <peer> <payload>`）
- **Engine 切替（Phase 113、案 C）** — `[engine] type = "agent_cli" | "claude_cli_shim"` で peer 駆動エンジンを選択可能。後者は Claude Code (`claude` CLI) を子プロセスで保持する `claude-cli-shim` wrapper
- **仕様書駆動開発** — 自然言語仕様書から HDL コード・制約ファイル・テストベンチを **LLM が動的生成**（テンプレート埋め込み禁止、Phase 42）
- **ベンダー非依存の抽象化** — `ToolAdapter`/`VendorAdapter` トレイトによる統一インターフェース。`adapter.toml` を書くだけでツール追加可能
- **コンテナ & ローカル実行** — Podman rootless コンテナまたはローカル実行を選択可能。lock ファイルによるビルド再現性
- **AI エージェントパイプライン** — WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent によるツールバージョンアップ自動追従
- **サブエージェント並列度制御（Phase 126）** — 3 段階階層 Semaphore + acquire timeout（global / ai-dispatch / per-conductor）で spawn 並列度を制御し、reviewer 用 reserved slot で重要 path の starvation を防止。`.hestia/config.toml` `[concurrency]` で調整可能
- **自己アップデート & version 同期（Phase 124 / 127）** — `hestia upgrade` でソースから再ビルド + `~/.local/bin/` へ install。`build.rs` の `git describe` で `--version` 表示が GitHub TAG と自動同期

> **Hestia 設計原則**: Hestia は AI 駆動システムです。LLM が指示を解析して **HDL / 制約 / TCL を動的に生成** し、handler が処理します。テンプレートを並べて handler に渡すアーキテクチャは禁止です（[WORKFLOWS.md](./WORKFLOWS.md) 参照）。

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

### 自己アップデート（Phase 124）

```bash
hestia upgrade            # git pull → cargo build --release → ~/.local/bin/hestia 更新
hestia upgrade --no-pull  # 現在の作業ツリーから再ビルドのみ（pull スキップ）
hestia --version          # 例: "hestia 0.1.5-17-gaf88400" (Phase 127 で git describe 同期)
```

`hestia --version` は `build.rs` が build 時に `git describe --tags --dirty=-dirty` を取得して埋め込むため、tag 一致 commit なら `0.1.5`、tag からの diff があれば `0.1.5-17-gaf88400[-dirty]` を表示します。配布バイナリ（git 不在環境）では `[workspace.package] version` にフォールバック。

## 動作要件

- **Rust** 1.75+（[rustup](https://rustup.rs) でインストール）
- **Linux** x86_64（カーネル 5.x 以降）
- **Engine（いずれか 1 種を選択、Phase 113）**:
  - `agent-cli`（既定、Conductor 間 IPC を 4 backend で駆動）
  - `claude-cli-shim` + `claude` CLI + `ANTHROPIC_API_KEY`（案 C wrapper、Claude Code を peer 駆動エンジンとして使用）

### Engine 切替（Phase 113）

`.hestia/config.toml` の `[engine]` セクションで peer 駆動エンジンを選択:

```toml
[agent_cli]
backend = "claude"           # agent-cli 内部の --provider
model   = "claude-opus-4-7"

[engine]
# "agent_cli" (既定、後方互換) | "claude_cli_shim"
type = "claude_cli_shim"
# binary = "/path/to/claude-cli-shim"   # 省略時は type 既定
# registry_path = "/custom/registry"    # 省略時は engine 既定
# log_path      = "/custom/logs"        # 省略時は engine 既定
```

- `[engine]` 未設定時は `agent-cli` 既定で従来挙動と完全互換。
- `claude_cli_shim` 選択時は `~/.local/bin/claude-cli-shim`（または `binary` で指定したパス）が使用される。`claude` バイナリと `ANTHROPIC_API_KEY` が必須。
- registry / log path を agent-cli と共有する場合は、両 engine の peer 名衝突に注意（`hestia kill` で停止後に切替推奨）。
- 詳細仕様は [`./report_claude.md`](./report_claude.md)（Phase 112 調査報告）と `.aiprj/AI_PRJ_DESIGN.md` §10〜§12 を参照。

### サブエージェント並列度の制御（Phase 126）

サブエージェントが立ち上がりすぎることによる PC / LLM の過負荷とデッドロックを防ぐため、
`.hestia/config.toml` の `[concurrency]` セクションで 3 段階階層の並列度上限を設定できます。

```toml
[concurrency]
global_max = 8                       # ai-conductor が把握する全エージェント合計 (HESTIA_GLOBAL_MAX_AGENTS)
ai_conductor_dispatch_max = 2        # ai-conductor が同時 dispatch する domain conductor 数 (HESTIA_AI_DISPATCH_MAX)
per_conductor_max = 4                # 各 conductor が同時 spawn できるサブエージェント数 (HESTIA_PER_CONDUCTOR_MAX)
acquire_timeout_secs = 600           # slot 待機タイムアウト秒（デッドロック検知）(HESTIA_ACQUIRE_TIMEOUT_SECS)
```

- 階層 Semaphore で **L1 → L2 → L3 の取得順序を固定**することで circular wait を排除。
- `acquire_timeout_secs` 経過で **hold-and-wait を打切**り、デッドロックを検知（`dispatch_acquire_timeout` エラーを記録）。
- `global_max` のうち 1 slot を **reviewer 用 reserved slot** として予約し、Phase 77 の auto-spawn ai-reviewer が cap 限界下でも起動できるようにする。
- 設定の優先順位（Phase 128 で配線追加）: **`hestia start` 親プロセス env > `.hestia/config.toml [concurrency]` > library 既定**。
  - `hestia start` 起動時に `[concurrency]` の値を子 conductor process の env に export。`config.toml` の値が反映されるには **`hestia kill && hestia start` で再起動が必要**。
  - 親プロセスに既に `HESTIA_*` env が設定されていれば、それが config.toml より優先される（テスト / CI で一時 override する用途）。
- 既定値（8 / 2 / 4 / 600s）は中規模ワークロードを想定。LLM rate limit が厳しい環境は `global_max` を下げ、強力な workstation では上げる。

> **既知の制限事項**: `per_conductor_max` は **単一 `dispatch_coders.v1` 呼出内** の同時 spawn 上限であり、
> 複数 dispatch 呼出を跨いだ累積 alive agent 数の上限ではありません。
> 例えば `per_conductor_max=1` でも、`dispatch_coders.v1` が 4 回呼ばれると累計 4 個の rtl-coder agent が
> alive で残り得ます。これは現実装では `ConductorLimiter` の permit が spawn API call (~ms) のみ保持され、
> agent process lifetime に連動しないためです。累積 cap が必要な場合は `global_max=1` 等で
> 完全 sequential 化するか、`hestia kill` で定期的に集約してください。agent lifetime 連動の累積 cap
> 実装は将来 phase の検討事項です。

#### サブエージェント起動数の最小値

各上限値は実装側 (`ConductorLimiter::new` / `AgentManager::with_caps`) で `max(1)` の
下限ガードがあり、以下の最小値が保証されます。

| 設定 | 最小値 | 実装上の挙動 |
|------|------|-------------|
| `global_max` | **1**（一般 spawn 用 1 slot + reviewer 予約 1 slot で実質 2） | `general = global_max.saturating_sub(1).max(1)` で、`global_max=0/1` でも一般 limiter は 1 slot 確保。reviewer 予約 slot は別 Semaphore（capacity 1）で常に 1 slot 確保 |
| `ai_conductor_dispatch_max` | **1** | `Semaphore::new(max.max(1))` で必ず 1 slot 以上 |
| `per_conductor_max` | **1** | 同上 |
| `acquire_timeout_secs` | 任意（推奨 1 秒以上） | 0 設定時は acquire が即時 timeout |

**最小値設定で動かすケース**:

- `HESTIA_GLOBAL_MAX_AGENTS=1 HESTIA_AI_DISPATCH_MAX=1 HESTIA_PER_CONDUCTOR_MAX=1`
  で実質的に sequential 実行となり、PC / LLM 負荷を最小化できます（一般 1 + reviewer 1 で
  最大 2 サブエージェント）。
- 完全 sequential 化により dispatch 単位の I/O 振る舞いがデバッグしやすくなる一方、
  全体スループットは大きく低下します。CI 等で時系列を再現したい場合に有用。

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
hestia upgrade                 # ソースから再ビルド + 再 install (Phase 124)
hestia kill                    # 全 conductor を停止し registry を cleanup (Phase 123)

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

### リリース手順（Phase 127）

`scripts/release.sh` で `[workspace.package] version` の書換 + commit + tag を原子的に実施。

```bash
./scripts/release.sh 0.1.6                  # version 0.1.6 のリリースを準備
git push origin main --follow-tags          # main + tag v0.1.6 を push
```

スクリプトは (1) `Cargo.toml` の workspace version 書換 → (2) `cargo build --release` で
`Cargo.lock` 再生成 → (3) `git commit "Release vX.Y.Z" + git tag vX.Y.Z` を実行します。
`build.rs` が tag 作成を `.git/refs/tags` watch 経由で検知し、次回ビルドで `hestia --version`
が新 tag に追従します。push は安全プロトコル準拠で常に手動。

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

HESTIA は **トリプルライセンス**（HESTIA ソフトウェアライセンス Version 1.0）の下で配布されます。
詳細条項は [`LICENSE.md`](./LICENSE.md) を参照してください。

### 関連ドキュメント

| ドキュメント | 内容 |
|------------|------|
| [`LICENSE.md`](./LICENSE.md) | HESTIA ソフトウェアライセンス（トリプルライセンス、日本語正文） |
| [`CLA.md`](./CLA.md) | Contributor License Agreement |
| [`SELF-DECLARATION.md`](./SELF-DECLARATION.md) | 小規模事業者救済 自己申告制度 運用マニュアル |
| [`FAQ.md`](./FAQ.md) | ライセンス・CLA・自己申告制度に関する FAQ |
| [`SUPPORT.md`](./SUPPORT.md) | サポート窓口・問合せ手順 |

Copyright (C) 2026 AQUAXIS TECHNOLOGY. All Rights Reserved.
