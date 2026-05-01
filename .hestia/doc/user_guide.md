# ユーザーガイド（User Guide）

**対象領域**: Hestia 全体
**ソース**: 設計仕様書 §15（CLI 実行機能）, §16（フロントエンド）

---

## 1. はじめに

Hestia は統合ハードウェア開発環境であり、9つの Conductor（ai / rtl / fpga / asic / pcb / hal / apps / debug / rag）が連携して FPGA・ASIC・PCB 設計フローを統括します。本ガイドでは CLI・VSCode 拡張・Tauri IDE の使用方法を説明します。

---

## 2. インストール

### 2.1 ビルド手順

```bash
# ワークスペース（.hestia/tools/Cargo.toml、resolver = "2"）
cd .hestia/tools

# 全バイナリ（9 conductor + 10 CLI）
cargo build --release

# 特定の conductor のみビルド
cargo build --release -p hestia-fpga-conductor

# テスト実行
cargo test                          # 全テスト
cargo test -p hestia-fpga-conductor # 特定 conductor
```

### 2.2 デーモン起動順序

```bash
# Group 0（直列・最高優先度）
hestia-ai-conductor &
# system.health.v1 が status="online" を返すまで待機
hestia status --conductor ai

# Group 1（8 並列）
hestia-rtl-conductor &
hestia-fpga-conductor &
hestia-asic-conductor &
hestia-pcb-conductor &
hestia-hal-conductor &
hestia-apps-conductor &
hestia-debug-conductor &
hestia-rag-conductor &

# 推奨: systemd ユーザーユニット経由
systemctl --user start hestia-ai hestia-rtl hestia-fpga hestia-asic \
  hestia-pcb hestia-hal hestia-apps hestia-debug hestia-rag
```

---

## 3. CLI 使用方法

### 3.1 統合ランナー（hestia）

```bash
hestia init                     # .hestia/ 構築
hestia start                    # 全 9 conductor 起動
hestia start fpga               # 指定 conductor のみ
hestia status                   # 全 conductor 状態
```

### 3.2 RTL 設計フロー

```bash
hestia rtl init                                             # rtl.toml テンプレート
hestia rtl lint                                             # Verilator/Verible で Lint
hestia rtl simulate --tb tb_alu --simulator verilator
hestia rtl formal --properties properties.sv               # SymbiYosys
hestia rtl handoff --target fpga                           # 下流へハンドオフ
```

### 3.3 FPGA 設計フロー

```bash
hestia fpga init                                            # fpga.toml テンプレート
hestia fpga build artix7                                   # ビルド開始
hestia fpga status --job-id 1
hestia fpga report timing
hestia fpga simulate --tb tb_top --simulator iverilog
```

### 3.4 ASIC 設計フロー

```bash
hestia asic init
hestia asic pdk install sky130A
hestia asic build --pdk sky130A
hestia asic advance --job-id 1                              # 13 ステップを 1 段ずつ進行
```

### 3.5 PCB 設計フロー

```bash
hestia pcb init
hestia pcb build --board-name "センサーボード"               # AI 回路図合成
hestia pcb output kicad --output-dir ./out
hestia pcb output gerber --output-dir ./gb
```

### 3.6 HAL 生成フロー

```bash
hestia hal init                                             # hal.toml テンプレート
hestia hal parse regs/soc.rdl                               # SystemRDL レジスタマップ解析
hestia hal validate                                         # アドレス重複・型整合性チェック
hestia hal generate c --output-dir build/hal/inc           # C ヘッダ生成
hestia hal generate rust --output-dir build/hal/rust       # Rust crate 生成
hestia hal generate svd --output build/hal/svd/soc.svd    # CMSIS SVD
hestia hal export-rtl --target rtl-conductor               # SystemRDL エクスポート
```

### 3.7 アプリケーション開発フロー

```bash
hestia apps init                                            # apps.toml テンプレート
hestia apps build --target thumbv7em-none-eabihf           # クロスコンパイル
hestia apps test sil                                       # QEMU SIL テスト
hestia apps test hil --probe stlink-v3                     # 実機 HIL テスト
hestia apps size                                           # バイナリサイズ解析
hestia apps flash --probe stlink-v3                        # フラッシュ書込
```

### 3.8 デバッグフロー

```bash
hestia debug create STM32F407 --interface-type swd
hestia debug connect --session-id 1
hestia debug capture start --session-id 1 --duration-ms 1000
hestia debug program --board fpga_board --bitstream out.bit
```

### 3.9 RAG（知識検索）

```bash
hestia rag ingest --source-id stm32_datasheet                # PDF/Web のソース投入
hestia rag search "STM32F103 SPI ピン配置" --top-k 5
hestia rag cleanup                                            # quarantine / 古キャッシュ整理
```

### 3.10 AI エージェント

```bash
hestia ai exec "Artix-7 で UART LED 制御回路を作って"        # 自然言語ジョブ
hestia ai run --file .aiprj/instructions.md                  # 仕様書駆動実行
hestia ai container ls
hestia ai container create --conductor fpga --tool vivado:2025.2
hestia ai workflow run --workflow fpga-to-pcb-test-board
hestia ai review start --project ./my-project --target artix7
```

### 3.11 共通オプション

`CommonOpts`: `--output (human|json)` / `--timeout` / `--registry` / `--config` / `--verbose`

### 3.12 Exit Code

| Exit Code | 意味 |
|-----------|------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

---

## 4. VSCode 拡張

### 4.1 インストール

VSIX パッケージ `hestia-vscode`（publisher: `aquaxis`、engines.vscode >= 1.85.0）をインストール。

### 4.2 アクティベーション

以下のイベントで自動アクティベーション:
- `onCommand`: `hestia.start|stop|status|ai|spec|fpga|debug|rag` ほか 30+ コマンド
- `onView`: `hestia-conductor` / `agents` / `specs`
- `onLanguage`: `verilog`, `vhdl`, `systemverilog`, `xdc`, `pcf`, `toml`

### 4.3 ビュー

| ビュー | 内容 |
|-------|------|
| ConductorStatusView | 9 conductor の状態表示 |
| AgentListView | サブエージェント一覧 |
| SpecViewer | 仕様書ビューア |
| DesignFlowView | 設計フロー可視化 |
| LogViewer | ログ表示 |

### 4.4 設定

主な設定項目（`hestia.*`）:

| 設定キー | 型 | 既定値 | 内容 |
|---------|-----|-------|------|
| `agentCliRegistryDir` | string | `$XDG_RUNTIME_DIR/agent-cli/` | agent-cli レジストリ |
| `autoConnect` | bool | `true` | 起動時自動接続 |
| `reconnectInterval` | number | `3000` | 再接続間隔(ms) |
| `requestTimeout` | number | `30000` | リクエストタイムアウト(ms) |
| `ai.model` | string | `claude-sonnet-4-6` | LLM モデル選択 |
| `ai.maxTokens` | number | `4096` | 最大トークン数 |
| `ai.apiKeyEnv` | string | `ANTHROPIC_API_KEY` | API キー環境変数名 |
| `ai.baseUrl` | string | `""` | API エンドポイント URL |

### 4.5 エディタ機能

- Monaco Editor 統合: HDL ハイライト・補完・診断（HDL LSP Broker 経由）
- 波形ビューア: WebView 内 WASM レンダリング

---

## 5. Tauri デスクトップアプリ

### 5.1 設定

- バージョン: `0.1.0`
- 識別子: `dev.hestia.ide`
- バンドルターゲット: `deb`, `rpm`, `appimage`

### 5.2 ウィンドウ

| ウィンドウ | サイズ | 用途 |
|-----------|-------|------|
| main | 1440×900 | メイン IDE |
| waveform | 1200×600 | 波形ビューア |
| settings | 800×600 | 設定パネル |

### 5.3 セキュリティ

CSP: `connect-src 'self' ipc: ws://localhost:*`

### 5.4 Shell プラグイン

`hestia` / `hestia-{ai,rtl,fpga,asic,pcb,hal,apps,debug,rag}-cli` の 10 コマンドが Tauri Shell 経由で実行可能。

---

## 6. UI コンポーネント（hestia-ui）

React + TypeScript 製コンポーネントライブラリ:

| コンポーネント | 用途 |
|-------------|------|
| ConductorStatusCard | conductor 状態表示 |
| AgentList | サブエージェント一覧 |
| SpecViewer | 仕様書表示 |
| LogViewer | ログ表示 |
| WaveformViewer | 波形表示 |
| ConfigPanel | 設定パネル |
| TaskProgress | タスク進捗 |

ブランド色: プライマリ akane `#e84d2c`、セカンダリ deep green `#2d8f5e`

---

## 関連ドキュメント

- [architecture_overview.md](architecture_overview.md) — アーキテクチャ概要
- [agent_communication.md](agent_communication.md) — 通信仕様
- [frontend/cli_clients.md](frontend/cli_clients.md) — CLI 詳細仕様
- [frontend/vscode_extension.md](frontend/vscode_extension.md) — VSCode 拡張詳細
- [frontend/tauri_ide.md](frontend/tauri_ide.md) — Tauri IDE 詳細