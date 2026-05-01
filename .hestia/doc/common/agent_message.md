# agent-cli メッセージ仕様

**対象領域**: common — メッセージング
**ソース**: 設計仕様書 §14.1

## 概要

HESTIA の全通信は agent-cli ネイティブ IPC に統一されている。本章は agent-cli IPC 上で送受される構造化メッセージ（JSON ペイロード）の仕様を詳述する。自然言語ペイロードは agent-cli の persona / LLM が直接処理するため対象外。

## トランスポートとフレーム

- **トランスポート**: agent-cli ネイティブ IPC（`$XDG_RUNTIME_DIR/agent-cli/` 配下の Unix Domain Socket、agent-cli が自動管理）
- **権限**: レジストリディレクトリは `0700`（owner のみ）、各 peer ソケットは `0600`
- **フレーミング**: agent-cli ネイティブフレーム（length-delimited、本体最大 16 MiB）
- **接続**: peer 探索は `agent-cli list`、送信は `agent-cli send <peer> <payload>` または REPL 内 `/send <peer> <payload>`
- **ペイロード判定**: 先頭 `{` の場合は構造化 JSON、それ以外は自然言語テキストとして解釈（§2.3）

## 構造化ペイロード形式

### Request

```json
{
  "method": "fpga.build.v1.synthesize",
  "params": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Success Response

```json
{
  "result": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Error Response

```json
{
  "error": { "code": -32200, "message": "...", "data": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### Notification（id なし、応答なし）

```json
{
  "method": "agent.status_update",
  "params": { ... },
  "trace_id": "trace_xyz789"
}
```

### Batch（同順応答）

```json
[
  { "method":"...", "params":{}, "id":"msg_1" },
  { "method":"...", "params":{}, "id":"msg_2" }
]
```

## ID 規約

- `id` 形式: `msg_{ISO8601 timestamp}_{random}`（agent-cli の慣習に整合）
- `trace_id`: ワークフロー横断のトレース ID（§19 観測性プラクティスと連鎖）
- レガシー JSON-RPC 2.0 の `"jsonrpc": "2.0"` フィールドは不要（agent-cli IPC 自体がトランスポートを規定）

## エラー応答 data フィールド

エラー応答 `data` には以下を含める:

| フールド | 説明 |
|---------|------|
| `tool` | 発生元ツール名 |
| `exit_code` | プロセス終了コード |
| `log_path` | ログファイルパス |
| `errors[]` | エラー詳細配列 |
| `retry_possible` | リトライ可否 |
| `suggested_action` | 推奨対応 |

## ペイロード選択指針

| 通信種別 | 推奨ペイロード | 理由 |
|---------|-------------|------|
| 構造化操作（build / test / query / status） | 構造化 JSON | 型安全、エラーコード規約、SDK 互換 |
| conductor 間の構造化ツール呼出 | 構造化 JSON | 再現性、トレース ID 連鎖 |
| conductor 間の自然言語協調 | 自然言語テキスト | 自由形式、CoT 文脈共有 |
| イベント通知 | 構造化 JSON（id なし） | 購読 / フィルタ可能 |

## 関連ドキュメント

- [api_versioning.md](api_versioning.md) — メソッド名前空間・バージョニング
- [error_registry.md](error_registry.md) — エラーコード全一覧
- [agent_cli_messaging.md](agent_cli_messaging.md) — 完全なメッセージング仕様
- [observability.md](observability.md) — trace_id 連鎖