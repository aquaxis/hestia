# agent-cli 完全メッセージング仕様

**対象領域**: common — 通信基盤
**ソース**: 設計仕様書 §14, §2.3, §20

## 概要

HESTIA の全通信は agent-cli ネイティブ IPC に統一されている。本ドキュメントはトランスポート、フレーム、ペイロード形式、判定ロジックの完全な仕様を定義する。

## トランスポート

- **基盤**: agent-cli ネイティブ IPC
- **ソケット**: `$XDG_RUNTIME_DIR/agent-cli/` 配下の Unix Domain Socket（agent-cli が自動管理）
- **権限**: レジストリディレクトリ `0700`、各 peer ソケット `0600`
- **プロトコル**: length-delimited フレーミング、本体最大 16 MiB

## Peer モデル

### Conductor Peer

| Peer 名 | Conductor | 役割 |
|---------|-----------|------|
| `ai` | ai-conductor | メタオーケストレーター |
| `rtl` | rtl-conductor | RTL 設計フロー |
| `fpga` | fpga-conductor | FPGA 設計フロー |
| `asic` | asic-conductor | ASIC 設計フロー |
| `pcb` | pcb-conductor | PCB 設計フロー |
| `hal` | hal-conductor | HAL 生成 |
| `apps` | apps-conductor | アプリ FW |
| `debug` | debug-conductor | デバッグ |
| `rag` | rag-conductor | 知識基盤 |

### 共有サービス Peer

| Peer 名 | サービス |
|---------|---------|
| `lsp` | HDL LSP Broker |
| `constraint-bridge` | Constraint Bridge |
| `ip-manager` | IP Manager |
| `cicd` | CI/CD API |
| `observability` | Observability |
| `waveform` | WASM 波形ビューア |
| `mcp` | MCP サーバー |

### フロントエンド Peer

| Peer 名 | クライアント |
|---------|------------|
| `vscode` | VSCode 拡張 |
| `tauri` | Tauri デスクトップアプリ |
| `cli` | CLI クライアント（任意）|

## ペイロード形式

### 構造化 JSON ペイロード

先頭が `{` の場合、構造化 JSON として解釈:

```json
{
  "method": "fpga.build.v1.synthesize",
  "params": { "target": "artix7" },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### 自然言語ペイロード

先頭が `{` 以外の場合、自然言語テキストとして解釈:

```
Vivado で build を開始してください、target=artix7
```

### 判定ロジック

```
受信ペイロード
  │
  ├── 先頭が '{' → 構造化 JSON として tool 呼出に変換
  │                  method 名前空間規約（§14.2）に従い dispatch
  │
  └── それ以外   → 自然言語テキストとして agent-cli の LLM に直接渡す
```

## 操作 API

| API | 説明 |
|-----|------|
| `agent-cli list` | 稼働中の peer 一覧取得 |
| `agent-cli send <peer> <payload>` | 指定 peer へペイロード送信 |
| REPL: `/send <peer> <payload>` | REPL 内からの送信 |

## ConductorRpc 共通 API

全 conductor が実装する共通 RPC トレイト:

| メソッド群 | メソッド例 |
|----------|----------|
| プロジェクト管理 | `project_open` / `project_targets` / `project_files` |
| ビルド | `build_start` / `build_cancel` / `build_status` |
| レポート | `report_timing` / `report_resource` / `report_messages` |
| プログラミング | `program_targets` / `program_flash` |
| ツールチェーン | `toolchain_list` / `toolchain_install` / `toolchain_select` |
| エージェント | `agent_status` / `agent_patch_list` / `agent_apply_patch` |
| コンテナ | `container_list` / `container_start` / `container_stop` / `container_update` |
| システム | `system_readiness` / `system_health` |

## ペイロード選択指針

| 通信種別 | 推奨ペイロード | 理由 |
|---------|-------------|------|
| 構造化操作（build / test / status） | 構造化 JSON | 型安全、SDK 互換 |
| conductor 間の構造化ツール呼出 | 構造化 JSON | 再現性、トレース ID 連鎖 |
| conductor 間の自然言語協調 | 自然言語テキスト | 自由形式、CoT 文脈共有 |
| 進捗・CoT・思考過程の共有 | 自然言語テキスト | 軽量伝搬 |
| イベント通知 | 構造化 JSON（id なし） | 購読 / フィルタ可能 |
| エラーエスカレーション | 自然言語で ai-conductor に集約 → 構造化通知で UI へ | 文脈詳細集約 → 即時反映 |

## 関連ドキュメント

- [agent_message.md](agent_message.md) — ペイロード形式詳細
- [api_versioning.md](api_versioning.md) — メソッド名前空間・バージョニング
- [error_registry.md](error_registry.md) — エラーコード規約
- [backend_switching.md](backend_switching.md) — LLM バックエンド切替