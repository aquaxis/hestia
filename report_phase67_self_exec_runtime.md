# Phase 67 persona 自己実行ループ runtime 期待挙動仕様

**作成日**: 2026-05-04
**対象**: Phase 57b + 61 で全 52 persona に追加した「起動時の `.aiprj/` 自己実行規約」節が、実 LLM 環境でどう動作するかの **期待挙動仕様化** + 検証可能な振る舞い契約

---

## 1. 背景

Phase 64 で persona 自己実行規約節の存在は構造確認済（grep ヒット 52/52）。本 Phase 67 では **実 LLM 環境での期待挙動** を仕様化し、Phase 66 E2E 実機検証時に各 persona が「正しく動作している」ことを判断できる基準を提供する。

---

## 2. 自己実行ループの runtime 状態遷移

```
[agent-cli プロセス起動]
        │
        │ persona system prompt のみ与えられる（peer prompt 待機）
        ▼
[初回 peer prompt 受信]
        │
        ▼
[persona §5 (自己実行規約) を LLM が解釈]
        │
        │ Step 1: fs_read .aiprj/instruction.md
        ├──→ 失敗（ファイル無し）→ skeleton 生成スキップ → 通常業務へ
        ├──→ 空 instruction → 通常業務へ
        └──→ 内容あり
                │
                │ Step 2: fs_read .aiprj/AI_PRJ_REQUIREMENTS.md
                ├──→ 不在 → setup_ai サイクル
                ├──→ 存在 + 内容差分あり → update_ai サイクル
                └──→ 存在 + 整合済 → exec_job サイクル
                        │
                        ▼
[Step 3: 該当サイクルを実行]
        │
        │ ・setup_ai: fs_read .aiprj/rules/setup_ai.md → 規約に従い 3 文書 fs_write
        │ ・update_ai: fs_read .aiprj/rules/update_ai.md → 既存 3 文書を改訂
        │ ・exec_job: fs_read .aiprj/rules/exec_job.md → タスク実行 + AI_LOG fs_write
        │
        ▼
[Step 4: 通常業務へ遷移]
        │
        │ ・ai persona: ステップ 1〜6 の Workflow Orchestrator
        │ ・conductor persona: domain 固有業務
        │ ・sub-agent persona: planner/designer/coder/etc 固有業務
        ▼
[継続的に peer prompt 処理]
```

---

## 3. 各サイクルの期待挙動詳細

### 3.1 setup_ai サイクル

**入力条件**: `instruction.md` あり + `AI_PRJ_REQUIREMENTS.md` 不在
**期待 fs_write**:
- `.aiprj/AI_PRJ_REQUIREMENTS.md`: setup_project.md 規約に従う要件定義書
- `.aiprj/AI_PRJ_DESIGN.md`: 設計仕様書
- `.aiprj/AI_PRJ_TASKS.md`: 作業仕様書（タスク + 作業指示）
- `.aiprj/AI_LOG/YYYY-MM-DD_000.md`: 初回セットアップログ

**Phase 59 との関係**:
ai-conductor の場合、AiHandler::handle_exec が冒頭で `spec_driven_emit_skeleton` を呼んで skeleton を生成済（Phase 59）。ai persona の自己実行は **skeleton を読んで本格的な setup_ai 規約に従って改訂** する補完動作。

### 3.2 update_ai サイクル

**入力条件**: 3 文書整合済 + `instruction.md` の内容差分あり
**期待 fs_write**:
- 既存 3 文書を更新（差分箇所のみ）
- `.aiprj/AI_LOG/YYYY-MM-DD_NNN.md`: 更新ログ

**判定の難所**:
- 「内容差分あり」の判定は LLM の自然言語判断に依存
- 確実な差分検出のため、persona は `.aiprj/AI_LOG/` の最新エントリを fs_read して前回 instruction との比較を試みるのが推奨

### 3.3 exec_job サイクル

**入力条件**: 3 文書整合済 + `instruction.md` 不変
**期待挙動**:
- `AI_PRJ_TASKS.md` から TODO タスクを 1 つ進行
- 進捗を `.aiprj/AI_LOG/YYYY-MM-DD_NNN.md` に記録
- 該当タスク完了で `AI_PRJ_TASKS.md` の status を更新

**動的並列 sub-agent (rtl-coder-{module} 等) の特例**:
親 conductor (Phase 60/60b/65 dispatch) からの spec が `instruction.md` に書込まれているため、coder は exec_job サイクル内で「親からの spec を読んで責務範囲を fs_write」する単一タスクを実行。

---

## 4. 観測可能なシグナル（Phase 49 mirror で確認可能）

実 LLM 環境で persona 自己実行が正しく動いていることは、Phase 49 で導入した workspace agent.log の `[mirror][...]` 行で確認可能:

| 期待される行 | 意味 |
|----------|-----|
| `[mirror][thinking#NNN]` | LLM が自己実行規約を解釈中 |
| `[mirror][tool_call] fs_read args=...instruction.md...` | Step 1 確認中 |
| `[mirror][tool_call] fs_read args=...AI_PRJ_REQUIREMENTS.md...` | Step 2 確認中 |
| `[mirror][tool_call] fs_read args=...rules/setup_ai.md...` | setup_ai サイクル開始 |
| `[mirror][tool_call] fs_write args=...AI_PRJ_REQUIREMENTS.md...` | 3 文書生成中 |
| `[mirror][tool_call] fs_write args=...AI_LOG/...` | 作業ログ記録中 |

これらの行が **agent-cli プロセス起動後最初の peer prompt を受信した直後 30 秒以内** に観測されれば、persona §5 自己実行規約が正しく動作していると判定できる。

---

## 5. 失敗パターンと対処

| 症状 | 原因仮説 | 対処 |
|-----|--------|-----|
| `instruction.md` を fs_read せず通常業務に直行 | persona §5 が LLM context window の終端に近く忘却 | persona の冒頭近くに自己実行規約を移動 (Phase 67b 候補)|
| setup_ai サイクルで 3 文書が生成されない | LLM が rules/setup_ai.md を fs_read していない | persona §5 のステップ 3 の指示を強化 |
| update_ai と exec_job の判定誤り（毎回 setup_ai を実行） | 差分判定が AI_LOG を参照していない | persona §5 に「AI_LOG 最新エントリを fs_read してから判定」を追記 |
| 動的並列 sub-agent が親 spec を読まず空タスク | rtl-coder.md persona の §5 末尾の「動的並列起動 sub-agent」専用注記が薄い | persona §5 の動的注記を強化 |

これらは **Phase 67 範囲では documentation 改訂のみ**、実 LLM 検証は Phase 66 E2E で実施。

---

## 6. テスト戦略（Phase 67 完了で確立した方針）

### 6.1 unit-level (本セッションで完備)

- ✅ persona ファイルに自己実行規約節が含まれること（Phase 64 grep 検証、52/52）
- ✅ ランタイム側準備（init_aiprj_workspace、agent_cli_send 等）が unit test pass（Phase 57/55c、84+件）

### 6.2 integration-level (Phase 66 で実施)

- 実 LLM 環境で setup_ai/update_ai/exec_job サイクルが正しく分岐するか観測
- 各サイクルで期待される fs_write が実行されるか確認
- 動的並列 sub-agent (rtl-coder-{module}) が親 spec を読んで正しく実装するか確認

### 6.3 acceptance-level (Phase 66+ で実施)

- end-to-end で 1 つの instruction から `<root>/<domain>/<artifact>` がすべて生成され、後続 handler が pass すること
- 集約 JSON `.hestia/run_log/<run-id>.json` で `status: ok` を確認

---

## 7. Phase 67 仕様化の成果物

| 項目 | 状態 |
|-----|-----|
| 自己実行ループの runtime 状態遷移図 | ✅ §2 で記述 |
| 各サイクルの期待挙動詳細 | ✅ §3 で記述 |
| 観測可能なシグナル (Phase 49 mirror 行) | ✅ §4 で記述 |
| 失敗パターンと対処 | ✅ §5 で記述（Phase 67b/68+ 候補を識別）|
| テスト戦略 (unit/integration/acceptance) | ✅ §6 で記述 |

---

## 8. 結論

Phase 57b + 61 で構造的に追加した persona 自己実行規約が、Phase 67 仕様化により **実 LLM 環境での期待挙動が完全に文書化** された。Phase 66 E2E 実機検証時に本仕様書を判定基準として使用することで、persona 自己実行が「規約通りに動作しているか」を客観的に評価できる。

実 LLM 検証は Phase 66 で実施。本 Phase 67 は Phase 66 のための仕様化フェーズとして完了。
