# CLI 実行機能

**対象領域**: frontend — CLI クライアント
**ソース**: 設計仕様書 §15

## 概要

HESTIA は統合ランナー `hestia` と 9 個別 CLI で構成される 10 種類の Rust 製 CLI バイナリを提供する。フロントエンド（VSCode / Tauri IDE）なしでもフルフローを実行可能。

## CLI 構成

### 統合ランナー

| バイナリ | 主要サブコマンド |
|---------|----------------|
| `hestia` | `init` / `start [domain]` / `status` / `ai` / `rtl` / `fpga` / `asic` / `pcb` / `hal` / `apps` / `debug` / `rag` / `spec` |

### 個別 CLI（9 種）

| バイナリ | 主要サブコマンド |
|---------|----------------|
| `hestia-ai-cli` | `exec` / `run --file` / `agent ls` / `container ls|start|stop|create` / `workflow run` / `review start` |
| `hestia-rtl-cli` | `init` / `lint` / `simulate` / `formal` / `transpile` / `handoff` / `status` |
| `hestia-fpga-cli` | `init` / `build` / `synthesize` / `implement` / `bitstream` / `simulate` / `program` / `report timing|resource` / `status` |
| `hestia-asic-cli` | `init` / `build` / `pdk install|list` / `advance` / `drc` / `lvs` / `status` |
| `hestia-pcb-cli` | `init` / `build` / `ai-synthesize` / `output kicad|gerber|bom` / `drc` / `erc` / `status` |
| `hestia-hal-cli` | `init` / `parse` / `validate` / `generate c|rust|python|svd` / `export-rtl` / `diff` / `status` |
| `hestia-apps-cli` | `init` / `build` / `flash` / `test sil|hil|qemu` / `size` / `debug` / `status` |
| `hestia-debug-cli` | `create` / `connect` / `disconnect` / `program` / `capture start|stop` / `signals read` / `trigger set` / `reset` / `status` |
| `hestia-rag-cli` | `ingest` / `search` / `cleanup` / `status` |

## 共通オプション（CommonOpts）

| オプション | 説明 |
|----------|------|
| `--output (human|json)` | 出力形式 |
| `--timeout` | タイムアウト |
| `--registry` | agent-cli レジストリ（`$XDG_RUNTIME_DIR/agent-cli/`）|
| `--config` | 設定ファイルパス |
| `--verbose` | 詳細出力 |

## Exit Code

| Code | 意味 |
|------|------|
| 0 | SUCCESS |
| 1 | GENERAL_ERROR |
| 2 | RPC_ERROR |
| 3 | CONFIG_ERROR |
| 4 | TIMEOUT |
| 5 | NOT_CONNECTED |
| 6 | INVALID_ARGS |
| 7 | SOCKET_NOT_FOUND |
| 8 | PERMISSION_DENIED |

## CLI アーキテクチャ

各 CLI は Rust 製クライアントバイナリ（`tokio` + `serde` + `clap`）で、対応する conductor の agent-cli peer に agent-cli ネイティブ IPC で接続する。共通実装は `conductor-sdk::transport` と `AgentCliClient` 仕様に整合する。

## 使用例

```bash
# 統合ランナー
hestia init
hestia start fpga
hestia status

# RTL
hestia rtl init
hestia rtl lint
hestia rtl simulate --tb tb_alu --simulator verilator

# FPGA
hestia fpga build artix7
hestia fpga report timing

# ASIC
hestia asic pdk install sky130A
hestia asic build --pdk sky130A

# PCB
hestia pcb build --board-name "センサーボード"
hestia pcb output gerber --output-dir ./gb

# HAL
hestia hal parse regs/soc.rdl
hestia hal generate c --output-dir build/hal/inc

# Apps
hestia apps build --target thumbv7em-none-eabihf
hestia apps test sil

# Debug
hestia debug create STM32F407 --interface-type swd
hestia debug capture start --session-id 1 --duration-ms 1000

# RAG
hestia rag ingest --source-id stm32_datasheet
hestia rag search "STM32F103 SPI ピン配置" --top-k 5

# AI
hestia ai exec "Artix-7 で UART LED 制御回路を作って"
hestia ai workflow run --workflow fpga-to-pcb-test-board
```

## 関連ドキュメント

- [agent_cli_client.md](agent_cli_client.md) — agent-cli クライアント仕様
- [cargo_workspace.md](../common/cargo_workspace.md) — ワークスペース構成
- [installation.md](../common/installation.md) — ビルド手順