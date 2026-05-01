# サブエージェントライフサイクル

**対象領域**: common — エージェント管理
**ソース**: 設計仕様書 §3.3, §4, §13.7.7, §20.5

## 概要

各 conductor はサブエージェントを動的に起動・終了し、`agent-cli list` で生存管理を行う。サブエージェントは独立した agent-cli プロセスとして動作し、親 conductor と `agent-cli send <peer>` IPC で協調する。

## サブエージェント起動・終了

### 起動コマンド

```bash
agent-cli run \
    --persona-file ./.hestia/personas/<peer>.md \
    --name <peer> \
    --workdir .hestia/workspaces/<peer>
```

### 終了条件

- 割当タスクの完了・検証完了後
- 親 conductor からの終了指示
- アイドルタイムアウト（規定値: 300 秒）
- 異常終了（health-checker §3.3.2 が検知）

### 生存管理

```bash
agent-cli list    # 稼働中 peer 一覧取得
```

親 conductor は定期（30 秒間隔）に `agent-cli list` を実行し、サブエージェントの生存を確認。

## 代表的サブエージェント構成

### rtl-conductor

| サブエージェント | Peer 名 | 多重度 | 動的起動 |
|----------------|---------|-------|---------|
| planner | `rtl-planner` | 1 | 常駐 |
| designer | `rtl-designer` | 1 | 常駐 |
| coder | `rtl-coder-{module}` | N | モジュール数だけ動的 |
| tester | `rtl-tester` | 1 | 常駐 |

### fpga-conductor

| サブエージェント | Peer 名 | 多重度 |
|----------------|---------|-------|
| planner | `fpga-planner` | 1 |
| designer | `fpga-designer` | 1 |
| synthesizer | `fpga-synthesizer` | 1 |
| implementer | `fpga-implementer` | 1 |
| tester | `fpga-tester` | 1 |
| programmer | `fpga-programmer` | 1 |

### rag-conductor

| サブエージェント | Peer 名 | 多重度 |
|----------------|---------|-------|
| planner | `rag-planner` | 1 |
| designer | `rag-designer` | 1 |
| ingest | `rag-ingest-{source}` | N（ソース並列）|
| search | `rag-search` | 1（高負荷時 N）|
| quality_gate | `rag-quality` | 1 |
| archivist | `rag-archivist` | 1（高負荷時 N）|

## スケーリングポリシー

| 項目 | ポリシー |
|------|---------|
| 常駐エージェント | conductor の寿命と同期（1 instance）|
| 動的エージェント | タスク数だけ起動・終了 |
| 最大並列数 | 16 並列（超過時はキューイング）|
| リソース解放 | タスク完了後、agent-cli プロセスを終了 |

## ワークスペース

各サブエージェントは `.hestia/workspaces/<peer>/` に専用ワークスペースを持つ:

```
.hestia/workspaces/<peer>/
├── .aiprj/
│   ├── instruction.md
│   ├── AI_PRJ_REQUIREMENTS.md
│   ├── AI_PRJ_DESIGN.md
│   ├── AI_PRJ_TASKS.md
│   ├── AI_LOG/YYYY-MM-DD_NNN.md
│   └── rules/{setup_ai,update_ai,exec_job,close_ai}.md
└── （作業生成物）
```

## ヘルスチェック対象

全サブエージェントは ai-conductor の health-checker（§3.3.2）の対象に含まれる。30 秒間隔で `system.health.v1` をポーリングし、異常時は自動再起動（max 3 回）。

## 関連ドキュメント

- [backend_switching.md](backend_switching.md) — LLM バックエンド切替
- [health_check_orchestration.md](health_check_orchestration.md) — ヘルスチェック
- [conductor_startup.md](conductor_startup.md) — デーモン起動順序