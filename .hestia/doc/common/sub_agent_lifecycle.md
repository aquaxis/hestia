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
├── requirements.md     # setup_ai/update_ai サイクルで agent 自身が fs_write（Phase 89 で改名）
├── design.md           # 同上（Phase 89 で改名）
├── tasks.md            # 同上、詳細タスク / DAG 専用（Phase 89 改名 / Phase 107 で状態ログを task_status.md に分離）
├── task_status.md      # exec_job サイクルで自エージェントが状態のみ fs_write（Phase 107 新設）
├── agent.log           # agent-cli mirror 経由で自動記録（Phase 49）
└── （作業生成物）
```

**重要規約**:

- `<workspace>/instruction.md` placeholder は **生成しない**（Phase 92 で廃止）。指示は peer prompt 経由のみで受信
- 起動規約は project root の `.hestia/rules/{setup_project,update_project,exec_job}.md` から参照（Phase 81〜92）
- `.aiprj/` は project 管理 AI 専有領域で、各 sub-agent workspace には作成しない（Phase 91 規約 / Phase 102 で runtime 整合化）
- 3 文書 (`requirements.md` / `design.md` / `tasks.md`) は per-agent / 共用ではない（Phase 92 明確化）
- **Phase 107: `tasks.md` と `task_status.md` の責務分離**
  - `tasks.md` = setup_ai/update_ai サイクルで agent 自身が書く詳細タスク / DAG 専用、exec_job 中は不変
  - `task_status.md` = exec_job サイクルで agent 自身が書く担当タスクの状態（「未着手」「進行中」「完了」「ブロック」）専用
  - Phase 106 で `task.md`（Phase 103 由来 = 状態ログ）と `tasks.md`（Phase 89 由来 = 3 文書）を「意味的に同じ」と誤判定して統合した結果、`tasks.md` の DAG が状態更新で full-overwrite される衝突が発生していた問題を解消

## ヘルスチェック対象

全サブエージェントは ai-conductor の health-checker（§3.3.2）の対象に含まれる。30 秒間隔で `system.health.v1` をポーリングし、異常時は自動再起動（max 3 回）。

## 稼働監視ループ（Phase 108）

`hestia start ai` 実行時、`agent-cli run --persona ai` の LLM peer と常駐サブエージェント（ai-designer / ai-reviewer）に加えて、**`hestia monitor-daemon`** 子プロセスが自動 spawn される（`mirror` helper と同パターン）。監視デーモンは ai-conductor LLM peer と独立に稼働し、配下サブエージェント + 起動中 domain conductor の稼働状況を 30 秒周期で polling する。

### 監視対象

- ai-designer / ai-reviewer（常駐サブエージェント）
- rtl / fpga / asic / pcb / hal / apps / debug / rag のうち起動中のもの
- `ai` 自身は監視対象から除外（自己監視はしない）

### 判定ロジック

| 状態 | 監視ループでの扱い |
|-----|------------------|
| BUSY    | 稼働中 |
| WAITING | 稼働中 |
| STARTING| 稼働中（起動直後の誤再開を防止） |
| IDLE    | 停止扱い |
| ERROR   | 停止扱い |
| UNKNOWN | 停止扱い |
| プロセス不在 | 停止扱い |

「全停止」とは **同一周期内** に全監視対象が停止扱いであることを指す。1 体でも稼働中なら次の周期まで待機する。

### タスク残存判定

各 peer の `<workspace>/<peer>/task_status.md` を fs_read し、状態列が「未着手」「進行中」「ブロック」のいずれかである行が存在すれば残存とみなす。`task_status.md` 不在時は残存なしとして扱う（誤抑制優先 / 誤再開回避）。

### 再開指示

`agent-cli send <peer> "<指示文>"` で対象 peer の persona「作業再開」セクションに従った再開を指示する。指示文には `<workspace>/<peer>/{tasks.md, task_status.md}` を fs_read し未消化タスクから再開せよ、という旨を含める。指示送信後は **60 秒の cooldown** 中に追加送信しない。

### 関連 CLI コマンド

- `hestia monitor` (Phase 108): 人間ユーザー向けに稼働状況を定期更新表示する CLI（`--interval N` で更新間隔指定、`--once` で 1 回出力、`--all` で SKILLS 列表示）。ai-conductor の監視ループとは独立して動作し、再開指示は出さない。
- `hestia status`: 既存の 1 回出力。`hestia monitor --once` と等価。
- `hestia monitor-daemon`: (内部用 / hidden) `hestia start ai` から自動 spawn される。`hestia kill` で agent-cli / mirror と一括 SIGKILL。

### 環境変数

| 変数 | 既定値 | 範囲 | 用途 |
|------|------|------|------|
| `HESTIA_MONITOR_INTERVAL_SECS` | 30 | 5..=600 | 監視周期 |
| `HESTIA_MONITOR_COOLDOWN_SECS` | 60 | 0..=600 | 再開指示後の cooldown |
| `HESTIA_MONITOR_DISABLED` | unset | `1` で監視ループを無効化 | 検証 / debug 用 |

### 実装

- `.hestia/tools/clis/hestia/src/monitor.rs` — 純関数群（`is_all_stopped` / `resolve_monitor_targets` / `parse_task_status` / `has_pending_tasks` / `summarize_statuses` 等）+ `run_monitor_daemon()` / `run_monitor()` + 単体テスト 22 件。
- `.hestia/tools/clis/hestia/src/main.rs` — `Commands::Monitor` / `Commands::MonitorDaemon` / `start_conductor("ai")` での子プロセス spawn / `KILL_PATTERNS` への `hestia monitor-daemon` 追記。

## 自動終了ロジック（Phase 109）

監視デーモンは Phase 108 の「再開指示」に加えて、以下の自動終了ロジックを同周期内で評価する。順序は **① サブエージェント終了 → ② Conductor 終了 → ③ 既存の再開指示** の 3 段階。

### サブエージェント終了

各 peer の `<workspace>/<peer>/task_status.md` の全行が「完了」であり、かつ当該 peer の status が IDLE / ERROR / UNKNOWN のいずれかであれば、当該 peer に SIGTERM を送る（graceful 終了）。`HESTIA_MONITOR_TERMINATE_GRACE_SECS`（既定 10 秒、0..=60 にクランプ）の猶予後にまだ生存していれば SIGKILL に escalate。

対象には以下の両方が含まれる:

- 静的サブエージェント: `ai-designer` / `ai-reviewer`
- 動的サブエージェント: `<domain>-*` 形式（例: `rtl-coder-uart` / `asic-signoff` / `hal-designer`）

### Domain Conductor 終了（順序保証）

domain conductor について、以下を満たす時のみ終了させる:

1. 当該 conductor の `task_status.md` 全行が「完了」。
2. 当該 conductor の動的 sub-agent (`<domain>-*` peer) が `agent-cli list` に **1 件も存在しない**。

順序保証のため、サブエージェントが残存している間は conductor を終了させない。サブエージェントは ① で終了 → 次周期で `agent-cli list` から消失 → 続く周期の ② で conductor 終了、という 2 周期にまたがる遷移を取る。

### ai-conductor の除外

peer 名 `ai` は本ロジックの対象外。終了は人間ユーザの明示的な `hestia stop ai` または `hestia kill` のみ。

### 重複 spawn 防止（Phase 109 関連）

`spawn_agent_cli` および `start_conductor` は spawn 直前に `agent-cli list` を確認し、対象 peer 名が既登録なら warn ログを出して skip する。`hestia monitor-daemon` の子プロセス spawn も `pgrep -f "hestia monitor-daemon"` で重複を防ぐ。これにより `hestia start ai` 多重実行などで生じる peer 重複起動（Phase 108 smoke test で観察された ai-reviewer × 5 件のような状況）が再発しない。

### 環境変数（Phase 109）

| 変数 | 既定値 | 範囲 | 用途 |
|------|------|------|------|
| `HESTIA_MONITOR_TERMINATE_GRACE_SECS` | 10 | 0..=60 | SIGTERM → SIGKILL escalate 猶予秒数 |

### 実装（Phase 109）

- `.hestia/tools/clis/hestia/src/monitor.rs` — `classify_peer` / `peer_tasks_all_complete` / `conductors_ready_to_terminate` / `is_terminable_status` / `terminate_peer` / `pgrep_agent_cli_pids` を新規追加、`MonitorTarget.parent_conductor` フィールドを追加、`run_monitor_daemon` を 3 段階処理に拡張。新規単体テスト 16 件追加。
- `.hestia/tools/clis/hestia/src/main.rs` — `registered_peer_names` 純関数 / `peer_already_registered` / `monitor_daemon_already_running` ヘルパ追加、`spawn_agent_cli` / `start_conductor` / monitor-daemon spawn の 3 経路に重複 check 追加。新規単体テスト 6 件追加。

## 関連ドキュメント

- [backend_switching.md](backend_switching.md) — LLM バックエンド切替
- [health_check_orchestration.md](health_check_orchestration.md) — ヘルスチェック
- [conductor_startup.md](conductor_startup.md) — デーモン起動順序