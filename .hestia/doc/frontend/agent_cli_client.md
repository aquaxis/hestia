# agent-cli クライアント仕様

**対象領域**: frontend — クライアント通信
**ソース**: 設計仕様書 §16.4

## 概要

Rust / TypeScript 共通の `AgentCliClient` 仕様。VSCode 拡張、Tauri IDE、CLI クライアントが agent-cli ネイティブ IPC 経由で conductor と通信するための共通インターフェース。

## メッセージ型

### AgentCliRequest

```typescript
interface AgentCliRequest {
  method: string;
  params?: Record<string, unknown>;
  id: string;
  trace_id: string;
}
```

### AgentCliResponse

```typescript
type AgentCliResponse =
  | AgentCliSuccessResponse
  | AgentCliErrorResponse;

interface AgentCliSuccessResponse {
  result: Record<string, unknown>;
  id: string;
  trace_id: string;
}

interface AgentCliErrorResponse {
  error: {
    code: number;
    message: string;
    data?: Record<string, unknown>;
  };
  id: string;
  trace_id: string;
}
```

### AgentCliNotification

```typescript
interface AgentCliNotification {
  method: string;
  params?: Record<string, unknown>;
  trace_id: string;
}
```

## HestiaClientConfig

| パラメータ | 型 | 既定値 | 説明 |
|----------|-----|-------|------|
| `agentCliRegistryDir` | string | `$XDG_RUNTIME_DIR/agent-cli/` | レジストリディレクトリ |
| `requestTimeout` | number | 30000 | リクエストタイムアウト（ms）|
| `reconnectInterval` | number | 5000 | 再接続間隔（ms）|
| `maxReconnectAttempts` | number | 10 | 最大再接続試行数 |
| `logLevel` | string | `"info"` | ログレベル |
| `retryPolicy` | RetryPolicy | 下記参照 | リトライポリシー |
| `maxFrameLength` | number | 16777216 | 最大フレーム長（16 MiB）|

## RetryPolicy

| パラメータ | 型 | 既定値 | 説明 |
|----------|-----|-------|------|
| `maxRetries` | number | 3 | 最大リトライ回数 |
| `initialBackoffMs` | number | 1000 | 初期バックオフ（ms）|
| `maxBackoffMs` | number | 60000 | 最大バックオフ（ms）|
| `multiplier` | number | 2.0 | バックオフ倍率 |
| `retryableCodes` | number[] | [-32001, -32006] | リトライ対象エラーコード |

## ConnectionState

| 状態 | 意味 |
|------|------|
| `disconnected` | 未接続 |
| `connecting` | 接続中 |
| `connected` | 接続済み |
| `reconnecting` | 再接続中 |
| `error` | エラー状態 |

## 内部実装

- peer 探索: 起動時に `agent-cli list` を実行
- 送信: `agent-cli send <peer> <payload>` を spawn または FFI 経由で呼出
- Rust 版: `conductor-sdk::transport` に実装
- TypeScript 版: VSCode 拡張 / Tauri IDE に実装

## 関連ドキュメント

- [cli_clients.md](cli_clients.md) — CLI クライアント
- [vscode_extension.md](vscode_extension.md) — VSCode 拡張
- [agent_cli_messaging.md](../common/agent_cli_messaging.md) — メッセージング仕様