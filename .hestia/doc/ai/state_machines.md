# ai-conductor タスク状態遷移

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §3.3.1（916-955行目付近）, §3.3.2（957-1010行目付近）

## タスク処理状態遷移

ai-conductor の task-router は、フロントエンドからの指示を受領してから配下 conductor への振り分け完了まで、以下の状態遷移を経る。

```
[受領] → 意図理解 → タスク分解 → ルーティング → 完了
                            │                │
                            │                └→ 失敗（conductor 到達不可）
                            │
                            └→ 分解失敗
```

### 状態定義

| 状態 | 説明 |
|------|------|
| Received | フロントエンドから指示を受領（自然言語 or 構造化 JSON） |
| IntentClassified | 意図理解完了（設計タスク種別判定: fpga build / asic synth / pcb route 等） |
| Decomposed | タスク分解完了（単一 conductor / 複数 conductor 横断 DAG / 仕様書ベース） |
| Routed | 配下 conductor へ dispatch 完了（`agent-cli send <peer> <payload>`） |
| Completed | 結果集約完了 → フロントエンドへ通知 |
| Failed | 処理失敗（エスカレーション / リトライ判定） |

### 分岐パターン

**単一 conductor で完結の場合:**
```
Received → IntentClassified → Decomposed → Routed → Completed
```

**複数 conductor 横断の場合:**
```
Received → IntentClassified → Decomposed → WorkflowEngine に委譲 → DAG 実行 → Completed
```

**仕様書ベースの場合:**
```
Received → IntentClassified → SpecDriven で DesignSpec 生成 → DAG 化 → WorkflowEngine 実行 → Completed
```

## ヘルスチェック状態遷移

health-checker は全 conductor を定期ポーリング（既定 30 秒間隔）し、ConductorStatus を更新する。

```
           ┌─────────────────────────────────────┐
           │                                     │
           ▼                                     │
  Online ──→ Offline ──→ 自動再起動(max 3) ─────┘
    │           │                       │
    │           │                  連続3回失敗
    ▼           │                       │
  Degraded     │                       ▼
    │           │              フロントエンド通知
    ▼           │           (agent.alert.v1)
  Upgrading     │
    │           │
    └───────────┘
```

### ConductorStatus

| 状態 | 説明 | 遷移トリガー |
|------|------|------------|
| Online | 正常稼働中 | 3 秒以内に "online" 応答 |
| Offline | 停止中 | タイムアウト (3 秒) |
| Degraded | 劣化状態（一部機能制限あり） | "degraded" 応答 |
| Upgrading | アップグレード中 | "upgrading" 応答 |

### 状態変化時アクション

| 遷移 | アクション |
|------|----------|
| Online → Offline / Degraded | Observability log + 自動再起動試行 (max 3) |
| 連続 3 回失敗 | フロントエンド通知 (`agent.alert.v1`) |
| Upgrading → Online | upgrade-manager に成功通知 |
| 任意 → 状態履歴を sled に永続化 | §19 オブザーバビリティ連携 |

## ワークフロー実行状態遷移

WorkflowEngine による DAG ベース実行のステップ状態遷移。

| StepStatus | 説明 |
|-----------|------|
| Pending | 依存ステップ未完了 |
| Ready | 依存ステップ完了、実行可能 |
| Running | 実行中 |
| Completed | 成功完了 |
| Failed | 失敗 |
| Skipped | スキップ（依存ステップ失敗等） |

### ダイヤモンド型依存関係の例

```
        [A: FPGA 合成]
       /              \
[B: ASIC 合成]    [C: PCB 設計]
       \              /
        [D: 統合検証]
```

A → B, A → C, B → D, C → D。A 完了後 B・C は並列実行、D は B・C 両方完了後に実行。

## 起動順序オーケストレーション

| Group | Conductor | 起動方式 |
|-------|-----------|---------|
| Group 0 | ai-conductor | 直列・最高優先度 |
| Group 1 | rtl / fpga / asic / pcb / hal / apps / debug / rag | 8 並列（ai readiness 確認後） |

## 関連ドキュメント

- [ai/message_methods.md](message_methods.md) — ai.* メソッド一覧
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine 詳細
- [ai/agent_hierarchy.md](agent_hierarchy.md) — サブエージェント構成
- [../common/conductor_startup.md](../common/conductor_startup.md) — 起動順序詳細