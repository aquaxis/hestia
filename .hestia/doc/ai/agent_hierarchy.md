# ai-conductor サブエージェント構成

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §3.10（1189-1239行目付近）

## 概要

ai-conductor は §3.3.1 のタスク振り分けと §3.6 SpecDriven を支援するために、配下に 2 種類のサブエージェントを持つ。各サブエージェントは独立した agent-cli プロセス（§20）として起動され、`agent-cli send <peer>` IPC（§2.3）で ai-conductor 本体（peer 名 `ai`）と協調する。

## サブエージェント一覧

| サブエージェント | peer 名 | 役割 | 多重度 | persona ファイル |
|----------------|---------|------|-------|-----------------|
| **planner** | `ai-planner` | フロントエンド指示タスクの分解・実行プランニング（DAG 化、依存関係解析、配下 conductor への dispatch 戦略） | 1（高負荷時 N 並列可） | `.hestia/personas/ai-planner.md` |
| **designer** | `ai-designer` | フロントエンド指示に基づく全体仕様（DesignSpec、HW/SW 統合の上位設計、conductor 間連携契約）の作成 | 1 | `.hestia/personas/ai-designer.md` |

## 協調フロー

```
[Frontend (VSCode/Tauri/CLI)]
       │
       │ agent-cli send ai <payload>
       ▼
[ai-conductor (peer "ai")]
       │
       ├── 内部委譲 1: agent-cli send ai-planner '{"method":"plan.v1.create",...}'
       │       │
       │       ▼
       │   [planner サブエージェント]
       │       ↓ プラン応答（DAG / Step リスト / 配下 conductor 割当案）
       │
       ├── 内部委譲 2: agent-cli send ai-designer '{"method":"design.v1.create",...}'
       │       │
       │       ▼
       │   [designer サブエージェント]
       │       ↓ DesignSpec 応答（上位仕様 / conductor 間連携契約）
       │
       ▼
[ai-conductor: planner + designer の出力を統合 → conductor-router で dispatch]
```

## 起動コマンド

```bash
# ai-conductor 起動時に planner / designer を agent-cli で同時起動
agent-cli run --persona-file ./.hestia/personas/ai-planner.md  --name ai-planner  &
agent-cli run --persona-file ./.hestia/personas/ai-designer.md --name ai-designer &
```

## スケーリングと寿命

- planner / designer はいずれも **常駐型** で、ai-conductor の寿命と同期して起動・停止される
- 高負荷時には planner を複数 instance 起動可能（peer 名 `ai-planner-1`, `ai-planner-2` ...）
- `agent-cli list` で discoverable、health-checker（§3.3.2）の対象に含まれる

## RAG 連携

rag-conductor が稼働中（`system.health.v1` で `online` 応答）の場合、task-router は `rag.search_similar.v1` で過去類似タスク事例を取得し、planner 配信時に context に注入する（§13.7.8 自己学習ループ）。

## ConductorManager と ConductorId

ai-conductor は ConductorManager で配下 9 conductor（self + 8 下流）を管理する。

| ConductorId | peer 名 | 対象 |
|-------------|---------|------|
| Ai | `ai` | self（ループバックヘルスチェック用） |
| Rtl | `rtl` | RTL 上流（§4） |
| Fpga | `fpga` | FPGA（§5） |
| Asic | `asic` | ASIC（§6） |
| Pcb | `pcb` | PCB（§7） |
| Hal | `hal` | HAL 生成（§8） |
| Apps | `apps` | アプリ FW（§9） |
| Debug | `debug` | デバッグ（§10） |
| Rag | `rag` | RAG（§13.7） |

## 関連ドキュメント

- [ai/state_machines.md](state_machines.md) — タスク状態遷移
- [ai/skills_system.md](skills_system.md) — SkillSystem 詳細
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine 詳細
- [../common/sub_agent_lifecycle.md](../common/sub_agent_lifecycle.md) — サブエージェントライフサイクル