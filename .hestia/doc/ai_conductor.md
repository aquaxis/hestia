# ai-conductor 全体概要 — メタオーケストレーター

**対象領域**: ai-conductor（メタオーケストレーター）
**ソース**: 設計仕様書 §3（745-1240行目）

---

## 概要

ai-conductor は HESTIA の最上位 conductor であり、フロントエンド（VSCode / Tauri / CLI）からの唯一の入口として機能する。配下の8つの conductor（rtl / fpga / asic / pcb / hal / apps / debug / rag）を統括し、ハードウェア開発の全プロセスを AI によってオーケストレーションする。

ai-conductor 自身は agent-cli プロセス（peer 名 `ai`）として起動され、下流 conductor との通信は全て agent-cli ネイティブ IPC で行われる。

---

## 中核機能

ai-conductor は以下の4つの中核機能を提供する。

| 機能 | 役割 |
|------|------|
| **タスク分解・振り分け** | フロントエンドからの自然言語または構造化指示を理解し、タスクを分解して配下の適切な conductor に振り分ける（task-router） |
| **ヘルスチェック** | 全 conductor を定期ポーリング（既定30秒間隔）し、Online / Offline / Degraded / Upgrading 状態を集約管理。異常時は自動再起動（max 3回）またはフロントエンドへエスカレーション |
| **スキル管理** | SkillRegistry に専門スキル（HDL 生成、制約生成、テストベンチ生成等）をプラグイン登録し、配下 conductor の agent-cli persona に提供 |
| **コンテナ管理** | `container.toml` 宣言に基づく Containerfile 自動生成・ビルド・差分更新・プロビジョニング・レジストリ管理（コンテナ実行を選択した場合のみ） |

---

## 補助機能

| 機能 | 説明 |
|------|------|
| 持続可能アップグレード | WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent によるツールバージョンアップ自動化 |
| DAG ベースワークフロー | トポロジカルソート（Kahn）によるクロス conductor パイプライン、sled で状態永続化 |
| 仕様書駆動開発 | 自然言語仕様書（`REQ:`/`CON:`/`IF:` プレフィックス）から DesignSpec 生成 → 設計データ自動生成 |
| LLM バックエンド切替 | Anthropic / Ollama / LM Studio / vLLM の切替対応 |

---

## ConductorManager

全 conductor のライフサイクルを管理する中核構造体。`ConductorId` 列挙型で8つの配下 conductor を識別し、`ConductorStatus` 列挙型（Online / Offline / Degraded / Upgrading）で状態を追跡する。

```rust
pub struct ConductorManager {
    conductors: Arc<RwLock<HashMap<ConductorId, ConductorInfo>>>,
    pub config: OrchestratorConfig,
}
```

---

## タスク振り分けフロー

1. **意図理解**: 自然言語 → 設計タスク種別判定 / 構造化 JSON → method 名前空間で直接判定
2. **タスク分解**: 単一 conductor 完結 → そのまま dispatch / 複数 conductor 横断 → workflow-engine に委譲し DAG 化 / 仕様書ベース → spec-driven で DesignSpec 生成
3. **振り分け**: conductor-router 経由で `agent-cli send <peer> <payload>` により適切な conductor へルーティング

---

## 起動順序

- **Group 0**: ai-conductor（最高優先度、直列起動）
- **Group 1**: rtl / fpga / asic / pcb / hal / apps / debug / rag（8並列、ai readiness 確認後）

---

## サブエージェント

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| planner | ai-planner | タスク分解・実行プランニング（DAG 化、dispatch 戦略） | 1（高負荷時 N 並列可） |
| designer | ai-designer | 全体仕様（DesignSpec、HW/SW 統合上位設計）作成 | 1 |

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計（フル版）
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター
- [pcb_conductor.md](pcb_conductor.md) — PCB 設計フローオーケストレーター
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター
- [apps_conductor.md](apps_conductor.md) — アプリケーションソフトウェア開発オーケストレーター
- [debug_conductor.md](debug_conductor.md) — デバッグ環境オーケストレーター
- [rag_conductor.md](rag_conductor.md) — 知識基盤オーケストレーター