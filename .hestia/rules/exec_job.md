---
name: Hestia Agent Execution Guidelines
description: hestia agent が exec_job サイクルで従う実行規約。`.aiprj/rules/exec_job.md` を agent 文脈に解釈変更したもの（Phase 81 P-3）。
---

# Hestia Agent Execution Guidelines (Phase 81)

このファイルは hestia agent が **exec_job サイクル**（上位からの実行依頼 prompt 受信時）で参照する規約です。プロジェクト管理 AI 用の `.aiprj/rules/exec_job.md` とは独立した hestia 文脈の実体です。

---

## Article 1: 作業の根拠

agent は次の優先順で作業の根拠を取得します:

1. **ペルソナ責務**: `.hestia/personas/<self>.md` の `name` / `description` フィールドが示す責務範囲
2. **workspace 状態**: `<workspace>/{requirements,design,tasks}.md`（自己実行サイクルで生成済の 3 文書）および `<workspace>/agent.log`（過去の対話履歴）
3. **上位 prompt**: 直近に受信した `agent-cli send` の text フィールド（上位指示はすべて本経路に統合、Phase 89 用語統一）

`.aiprj/AI_PRJ_REQUIREMENTS.md` 等のプロジェクト管理 AI 用文書は **参照しません**（Phase 22 P-1 / Phase 81 P-3 規約）。

---

## Article 2: workspace 内成果物の整合性

agent は workspace 配下および project root 配下の成果物 (`rtl/`, `hal/`, `fpga/`, `sim/`, `debug/` 等) の **整合性** を維持します。例:

- `hal/register_map.json` のレジスタ定義と `rtl/<top>.sv` のポート信号が整合すること
- `fpga/constraints/<top>.xdc` のピン制約と `rtl/<top>.sv` のトップレベル I/O が整合すること
- `sim/lint_report.json` の lint 結果に基づき `rtl/` 側の品質ゲートを判定すること

不整合を検出した場合、agent は **上位に halt 理由付きで報告** し、独自判断で修正を試みません（責務境界 / Phase 53 ai persona 是正）。

---

## Article 3: 進捗の記録

agent は明示的な fs_write による進捗記録を行いません。代わりに:

- agent-cli の構造化 JSONL ログ（`~/.local/share/agent-cli/logs/<peer>/*.jsonl`）に thinking / tool_call / tool_result が自動記録される
- `hestia mirror <peer>` 経路（Phase 49）で `<workspace>/agent.log` に要約行が real-time mirror される
- aggregate JSON (`<root>/.hestia/run_log/<run-id>.json`) は ai-conductor が一括出力する

ユーザーが進捗を見たい場合は `cat <workspace>/agent.log` または `hestia tail <peer>` で観測可能です。

---

## Article 4: ノンストップ実行 (Phase 50 / autonomous_work feedback 継承)

agent は exec_job サイクル中、user の許可確認を介在せず連続実行します。途中で停止する条件は次のみ:

1. ペルソナ責務範囲を超える作業要求を受信
2. 必要な入力 (params.* / 既存ファイル) が欠落（→ `input_required` 返却）
3. 必要な実ツール (verilator / Vivado / yosys 等) が不在（→ `tool_unavailable` 返却）
4. 上位から明示的な停止指示を受信

それ以外の状況（warning 検出 / iteration 進行中 / 部分成果のみ生成）では halt せず、各 status を honest 返却して上位の判断に委ねます。

---

## Article 5: 失敗時の透明な報告 (Phase 50 継承)

handler が以下のいずれかを返す場合、agent は **理由・次アクション候補・関連ログ抜粋** を 3 点セットで上位に報告します:

| status | 意味 | 報告必須項目 |
|--------|-----|-----------|
| `input_required` | 必要入力欠落 | 欠落 input 名 / 提供方法 |
| `tool_unavailable` | 実ツール不在 | tool 名 / インストール方法 / 代替手段 |
| `skipped` | 既存成果物再利用 | 既存ファイルパス / 再生成方法 |
| `*_failed` (lint_failed / build_failed 等) | 実ツール起動失敗 | 失敗 step / `error_log_excerpt` (50-200 行) |
| `sim_warnings` | warnings 検出 (Phase 50) | warning count / 内訳 / 抑制方法 |

「実行が止まった」だけの報告は禁止です。

---

## Article 6: フラグ位置規約 (Phase 17 継承)

agent が CLI を shell 経由で起動する場合、`--output json` 等の `CommonOpts` フラグは subcommand の **前** に置きます:

```bash
# 正
hestia-rtl-cli --output json lint --project ./
# 誤（clap flatten が拒否）
hestia-rtl-cli lint --project ./ --output json
```

`CommonOpts` 全フラグは `global = true` 設定済 (Phase 17) のため、技術的には後置でも動作しますが、persona 規約として前置を統一します。

---

## Article 7: shell 経由 in-process 実行 (Phase 16 / 方針 X)

ai-conductor LLM がオーケストレーターとして各 `hestia-{domain}-cli` を shell 経由で in-process Handler 呼び出しする設計です（agent-cli IPC 経路ではなく Rust 関数呼び出し）。これにより:

- shell ツール (LLM が `tool_call: shell`) で domain CLI を順次起動
- 各 CLI は自プロセスで domain handler を直接 invoke、構造化 JSON を stdout に返却
- 結果は `<root>/.hestia/run_log/<run-id>.json` に集約

これは Phase 16「方針 X 採択」の中核設計であり、agent はこの経路を変更してはいけません。

---

## Article 8: `.aiprj/` 直接参照の禁止 (Phase 81 新規)

hestia agent は `.aiprj/` ディレクトリへの直接参照を行いません。実行規約は `.hestia/rules/exec_job.md`（本ファイル）を参照対象とします。
