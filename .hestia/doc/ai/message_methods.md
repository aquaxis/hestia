# ai-conductor メッセージメソッド一覧

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §14（3492-3630行目付近）, §3（745-1240行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。`agent-cli send <peer> <payload>` で送信。ペイロードが先頭 `{` の場合は構造化 JSON、それ以外は自然言語として解釈される。

## ai.* メソッド一覧

### 仕様書駆動開発（SpecDriven）

| メソッド | 方向 | 説明 |
|---------|------|------|
| `ai.spec.init` | Request | 仕様書セッション初期化。自然言語仕様から DesignSpec 生成を開始 |
| `ai.spec.update` | Request | 既存 DesignSpec の更新。REQ/CON/IF プレフィックスで要件・制約・インターフェースを追記 |
| `ai.spec.review` | Request | 仕様書レビュー開始。レビュー結果 + 修正提案を返却 |

### 実行・制御

| メソッド | 方向 | 説明 |
|---------|------|------|
| `ai.exec` | Request | 自然言語または構造化指示の直接実行。task-router が意図理解→タスク分解→振り分けを実行 |

### エージェント管理

| メソッド | 方向 | 説明 |
|---------|------|------|
| `agent_spawn` | Request | サブエージェント（planner/designer/coder-N/tester）の新規起動 |
| `agent_list` | Request | 登録済みサブエージェント一覧取得。agent-cli list に相当 |
| `agent.status_update` | Notification | エージェント状態変化通知（id なし、応答なし） |

### コンテナ管理

| メソッド | 方向 | 説明 |
|---------|------|------|
| `container.list` | Request | コンテナ一覧取得 |
| `container.start` | Request | コンテナ起動 |
| `container.stop` | Request | コンテナ停止 |
| `container.create` | Request | container.toml から Containerfile 生成・ビルド |
| `container.update` | Request | コンテナイメージ差分更新 |

### ワークフロー

| メソッド | 方向 | 説明 |
|---------|------|------|
| `meta.dualBuild` | Request | 複数 conductor 並列ビルド（DAG: fpga.build ‖ asic.synth → meta.collect） |
| `meta.boardWithFpga` | Request | クロス conductor ワークフロー（FPGA + PCB 連携等） |
| `meta.handoff` | Notification | conductor 間ハンドオフイベント（rtl → fpga/asic 等） |

### システム共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.readiness` | Request | ai-conductor の準備状態確認（`{ ready: bool }`） |
| `system.health` | Request | ヘルスチェック（Online / Offline / Degraded / Upgrading） |
| `system.shutdown` | Request | ai-conductor シャットダウン |
| `agent.alert` | Notification | フロントエンドへのアラート通知（連続ヘルスチェック失敗時等） |

## ペイロードフォーマット

### リクエスト

```json
{
  "method": "ai.spec.init",
  "params": { "spec_text": "...", "format": "natural" },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### 成功応答

```json
{
  "result": { "design_spec": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### エラー応答

```json
{
  "error": { "code": -32100, "message": "...", "data": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

## メソッド名前空間規約

`{domain}.{method_group}.{version_prefix}.{action}`（例: `fpga.build.v1.synthesize`）。簡略形 `{domain}.{action}` も同義（v1 既定）。

- `ApiVersion { major, minor }`
- 互換性: 必須パラメータ追加・既存型変更・メソッド削除は `major` バンプ
- 廃止予告: `DeprecationNotice { deprecated_since, removal_scheduled, replacement }`

## 関連ドキュメント

- [ai/binary_spec.md](binary_spec.md) — hestia-ai-cli バイナリ仕様
- [ai/error_types.md](error_types.md) — ai-conductor エラーコード
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine 詳細
- [ai/skills_system.md](skills_system.md) — SkillSystem 詳細
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様