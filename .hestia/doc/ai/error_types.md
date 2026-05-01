# ai-conductor エラーコード

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）

## エラーコード範囲

ai-conductor のエラーコードは **-32100 〜 -32199** の範囲を使用する。

## エラーカテゴリ

### Orchestration（オーケストレーション）

| コード | 名称 | 説明 |
|-------|------|------|
| -32100 | ORCHESTRATION_ERROR | タスク振り分け・ワークフロー実行中の一般エラー |
| -32101 | TASK_DECOMPOSITION_FAILED | タスク分解失敗（意図理解不可・依存関係解析エラー） |
| -32102 | WORKFLOW_EXECUTION_FAILED | ワークフロー（DAG）実行失敗 |
| -32103 | CONDUCTOR_UNREACHABLE | 配下 conductor への到達不可（agent-cli IPC タイムアウト） |
| -32104 | DAG_CYCLE_DETECTED | DAG 内の循環依存検出 |

### Agent Management（エージェント管理）

| コード | 名称 | 説明 |
|-------|------|------|
| -32110 | AGENT_SPAWN_FAILED | サブエージェント起動失敗 |
| -32111 | AGENT_NOT_FOUND | 指定されたエージェントが存在しない |
| -32112 | AGENT_COMMUNICATION_FAILED | エージェント間通信失敗（IPC エラー） |
| -32113 | AGENT_TIMEOUT | エージェント応答タイムアウト |
| -32114 | MAX_AGENTS_EXCEEDED | エージェント並列数上限超過 |

### Spec-Driven（仕様書駆動）

| コード | 名称 | 説明 |
|-------|------|------|
| -32120 | SPEC_PARSE_ERROR | 仕様書パース失敗（REQ/CON/IF プレフィックス不正） |
| -32121 | SPEC_VALIDATION_FAILED | DesignSpec バリデーション失敗（必須要件不足等） |
| -32122 | SPEC_REVIEW_FAILED | レビューセッション失敗 |
| -32123 | DESIGN_SPEC_CONFLICT | 複数仕様間の矛盾検出 |

### Version Tracking（バージョン追跡）

| コード | 名称 | 説明 |
|-------|------|------|
| -32130 | VERSION_INCOMPATIBLE | セマンティックバージョニング非互換 |
| -32131 | ROLLOUT_FAILED | 段階的ロールアウト失敗（Canary/Staging） |
| -32132 | ROLLBACK_FAILED | 自動ロールバック失敗 |
| -32133 | UPGRADE_CHECK_FAILED | 新バージョン確認失敗（WatcherAgent） |

### LLM（大規模言語モデル）

| コード | 名称 | 説明 |
|-------|------|------|
| -32140 | LLM_BACKEND_UNAVAILABLE | LLM バックエンド接続不可（Ollama / Anthropic / LM Studio / vLLM） |
| -32141 | LLM_INFERENCE_FAILED | LLM 推論失敗 |
| -32142 | LLM_TIMEOUT | LLM 応答タイムアウト |
| -32143 | TOOL_USE_EXECUTION_FAILED | Tool Use 機能のツール実行失敗 |

## 共通エラーコード（HESTIA 全体共通）

| 範囲 | 領域 |
|------|------|
| -32700 | Parse Error（JSON ペイロードのパース失敗） |
| -32600 〜 -32603 | リクエスト標準エラー（Invalid Request / Method not found / Invalid params / Internal） |
| -32000 〜 -32099 | HESTIA 共通（Timeout / NotFound / AlreadyExists / PermissionDenied / InvalidState 等） |

## エラー応答フォーマット

`data` フィールドには以下を含める:

| フィールド | 説明 |
|-----------|------|
| `tool` | エラー発生元ツール名 |
| `exit_code` | プロセス終了コード |
| `log_path` | ログファイルパス |
| `errors[]` | 詳細エラーリスト |
| `retry_possible` | リトライ可能性 |
| `suggested_action` | 推奨される対応 |

```json
{
  "error": {
    "code": -32101,
    "message": "Task decomposition failed",
    "data": {
      "tool": "task-router",
      "retry_possible": true,
      "suggested_action": "Specify target conductor explicitly"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

## 関連ドキュメント

- [ai/message_methods.md](message_methods.md) — ai.* メソッド一覧
- [ai/state_machines.md](state_machines.md) — タスク状態遷移
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ
- [../common/error_handling_strategy.md](../common/error_handling_strategy.md) — エラーハンドリング戦略