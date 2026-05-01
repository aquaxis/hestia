# ai-conductor WorkflowEngine 詳細

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §3.5（1052-1077行目付近）, §1.3.5（148-177行目付近）

## 概要

WorkflowEngine は DAG（有向非巡回グラフ）ベースのクロス conductor パイプラインエンジンである。カーンのアルゴリズム（Kahn's algorithm）によるトポロジカルソートで実行順序を決定し、sled で状態を永続化する。

## クレート構成

```
workflow-engine/
└── src/
    ├── lib.rs      # WorkflowEngine 本体
    ├── dag.rs      # DAG 定義・実行
    └── pipeline.rs # クロス conductor パイプライン
```

## WorkflowStep 構造体

```rust
pub struct WorkflowStep {
    pub id: String,              // ステップ ID
    pub name: String,            // ステップ名
    pub conductor: String,       // 対象 conductor（peer 名）
    pub method: String,          // 実行する agent-cli メッセージ method（§14）
    pub params: Option<Value>,   // パラメータ
    pub depends_on: Vec<String>, // 依存ステップ ID（DAG 構造）
    pub status: StepStatus,      // 現在の状態
}
```

## StepStatus 状態一覧

| 状態 | 説明 |
|------|------|
| Pending | 依存ステップ未完了 |
| Ready | 依存ステップ完了、実行可能 |
| Running | 実行中 |
| Completed | 成功完了 |
| Failed | 失敗 |
| Skipped | スキップ（依存ステップ失敗等） |

## DAG 定義とトポロジカルソート

カーンのアルゴリズム（Kahn）により依存関係を解決し、実行可能なステップから順に実行する。依存関係を満たしたステップは並列実行可能。

### ワークフロー定義例（YAML 形式）

```yaml
steps:
  - id: fpga_synth
    conductor: fpga
    method: build/start
    params: { target: artix7 }
  - id: pcb_design
    conductor: pcb
    method: build/start
    depends_on: [fpga_synth]
  - id: debug_setup
    conductor: debug
    method: connect
    depends_on: [fpga_synth, pcb_design]
```

## ダイヤモンド型依存関係

分岐→合流のパターンにも対応する。

```
        [A: FPGA 合成]
       /              \
[B: ASIC 合成]    [C: PCB 設計]
       \              /
        [D: 統合検証]
```

A 完了後、B・C は並列実行。D は B・C 両方完了後に実行。

## sled 永続化

ワークフローの実行状態（各 StepStatus）は sled（Rust ネイティブ組み込み KV ストア）に永続化される。これにより:

- プロセス再起動後の実行状態復元
- 長時間ワークフローの中断・再開
- 実行履歴のトレーサビリティ

## クロス conductor パイプライン

WorkflowEngine は `meta.*` メソッド群を通じて、複数 conductor 間の連携を自動化する。

| メソッド | 説明 |
|---------|------|
| `meta.dualBuild` | 複数 conductor 並列ビルド（例: fpga.build ‖ asic.synth → meta.collect） |
| `meta.boardWithFpga` | FPGA + PCB 連携ワークフロー |
| `meta.handoff` | conductor 間ハンドオフイベント（rtl → fpga/asic 等） |

## 実行フロー

```
1. ワークフローを DAG として定義（YAML / 構造化 JSON）
2. WorkflowEngine がトポロジカルソートで実行順序を決定
3. 依存関係を満たしたステップから順に実行（並列実行対応）
4. 各ステップは対象 conductor の agent-cli peer に対して構造化メッセージ送信
5. ダイヤモンド型依存関係（分岐→合流）にも対応
6. ステップ間メッセージは構造化 JSON ペイロードまたは自然言語テキストとして伝搬
```

## 関連ドキュメント

- [ai/state_machines.md](state_machines.md) — タスク状態遷移
- [ai/agent_hierarchy.md](agent_hierarchy.md) — サブエージェント構成
- [ai/message_methods.md](message_methods.md) — ai.* / meta.* メソッド一覧
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様