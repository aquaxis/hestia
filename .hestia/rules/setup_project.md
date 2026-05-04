---
name: Hestia Agent Setup Guidelines
description: hestia agent (各 conductor / sub-agent) がワークスペース立ち上げ時に従う規約。`.aiprj/rules/setup_project.md` を agent 文脈に解釈変更したもの（Phase 81 P-3）。
---

# Hestia Agent Setup Guidelines (Phase 81)

このファイルは hestia agent（ai-conductor / 9 ドメイン conductor / 50+ sub-agent persona）が **起動時の setup フェーズ** で参照する規約です。プロジェクト管理 AI 用の `.aiprj/rules/setup_project.md` とは独立した実体で、hestia ランタイムが `.aiprj/` 不在環境でも動作するよう設計されています。

---

## Article 1: 入力の取得

agent は起動時に以下の優先順で入力を取得します。

1. `<workspace>/instruction.md`（peer 固有の作業指示。`init_hestia_workspace` で peer 起動時に placeholder が生成される）
2. 上位 conductor から `agent-cli send` 経由で受信した prompt（runtime 中の動的指示）
3. ペルソナ自身の責務範囲（`name` フィールドが示すドメイン責務）

入力が空または欠落の場合、agent は idle 状態に遷移し、上位からの指示を待機します。エラー終了してはいけません。

---

## Article 2: 成果物の作成範囲

agent はペルソナの責務範囲内で **workspace 内および project root 配下の成果物** を fs_write で作成します。

| ペルソナ階層 | 主な作成対象 |
|------------|------------|
| ai-conductor | `<root>/.hestia/run_log/<run-id>.json` aggregate / persona 自身の `<workspace>/agent.log` |
| domain conductor (rtl/fpga/asic/pcb/hal/apps/debug/rag) | `<root>/<domain>/...` 成果物（rtl/<top>.sv 等）/ `<workspace>/agent.log` |
| sub-agent (planner/designer/coder/tester/...) | 担当モジュールの設計 / 実装 / テスト成果物 |

`.aiprj/` ディレクトリへの書込は禁止（プロジェクト管理 AI の専有領域）。

---

## Article 3: テンプレート埋め込み禁止（Phase 42 継承）

`.hestia/tools/` 配下の handler ソースに HDL / 制約 / TCL / レジスタマップ等のドメイン固有テンプレートを埋め込んではいけません。LLM (ai-conductor / designer 等) が `fs_write` で動的生成し、handler は実ツール起動 (verilator / Vivado / yosys 等) のみ担当します。

詳細は `.hestia/personas/ai.md` の「絶対規約」節および Phase 42/47 の persona 規約を参照。

---

## Article 4: 自己実行ループ (Phase 57b/68/71 継承)

agent は次の 4 サイクルを自身で判定・実行します:

| サイクル | 契機 | 参照規約 |
|---------|------|---------|
| setup_ai | peer 起動直後 | 本ファイル (`.hestia/rules/setup_project.md`) |
| update_ai | `<workspace>/instruction.md` 更新検出 | `.hestia/rules/update_project.md` |
| exec_job | 上位からの実行依頼 prompt 受信 | `.hestia/rules/exec_job.md` |
| close_ai | セッション終了通知 | `.hestia/rules/close_ai.md`（Phase 82+ で追加予定）|

各サイクルの判定分岐ロジックは persona 内の「起動時の `.hestia/rules/` 自己実行規約」節に記載されています。

---

## Article 5: 進捗の可視化 (Phase 49 mirror 継承)

agent の活動 (thinking / tool_call / tool_result / peer_prompt / assistant) は agent-cli の構造化 JSONL に書き出されます。`hestia start` が detached helper として `hestia mirror <peer>` を spawn しており、ユーザーは `cat .hestia/workspaces/<peer>/agent.log` で real-time に活動を観測できます。

agent は明示的なログ出力を行う必要はありません — agent-cli の構造化イベントが mirror 経由で workspace agent.log に到達します。

---

## Article 6: 失敗時の透明な報告 (Phase 50 継承)

handler が `input_required` / `tool_unavailable` / `skipped` / `*_failed` を返した場合、agent は **次の判断材料となる粒度の理由** を上位に報告します。aggregate JSON の `halted_reason` フィールドおよび各 step の `error_log_excerpt` に inline で含めます。

「実行が止まった」だけの報告は禁止。ユーザーが next action を判断できる情報粒度を必須とします。

---

## Article 7: `.aiprj/` 直接参照の禁止 (Phase 81 新規)

hestia agent (persona / handler / Rust ソース) は `.aiprj/` ディレクトリへの直接参照を行ってはいけません。本規約 (`.hestia/rules/setup_project.md`) を含む `.hestia/rules/` 配下のみを参照対象とします。

例外: プロジェクト管理 AI（本リポジトリのトップレベルで動作する Claude Code セッション）は `.aiprj/` を継続利用しますが、これは hestia agent ではないため本規約の対象外です。
