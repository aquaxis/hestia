# Hestia Workflows

`hestia ai run --file <instructions.md>` で AI Workflow Orchestrator を起動した際の振る舞い、各ステップの返却 status、診断情報の読み方をまとめます。

## 設計原則: Hestia は AI 駆動システム（テンプレート埋め込み禁止）

**Hestia は AI 駆動のハードウェア開発環境です。LLM である AI orchestrator が指示を解析して必要な成果物（HDL / 制約 / TCL / レジスタマップ等）を `fs_write` で生成し、それを handler が処理します。**

以下は **明確に禁止**されています:

- ❌ Hestia core（`.hestia/tools/`）への特定アプリ・特定ボード固有データの埋め込み（Phase 21 で除去済）
- ❌ プロジェクト側 `<root>/.hestia/<domain>/templates/` ディレクトリへのテンプレート配置（Phase 42 で完全廃止）
- ❌ AI が「テンプレートを配置してから再実行してください」とユーザーに指示すること（AI 駆動の根本理念に反する）

**正しいフロー**:

1. AI orchestrator が指示を LLM 推論で解析
2. 必要な成果物を `fs_write` で project root（`<root>/hal/`, `<root>/rtl/`, `<root>/fpga/{constraints,scripts}/` 等）に書き出し
3. `hestia-{domain}-cli` を invoke、handler は project root の既存ファイルを読んで処理

handler が `status: "input_required"` を返した場合、それは「**AI が事前に成果物を fs_write しなかった**」シグナル。テンプレート不在を意味するのではない。

## ワークフロー実行フロー

```
hestia ai run --file instructions.md
  └─ hestia-ai-cli が agent-cli send で AI conductor (ai persona) に prompt
       └─ AI conductor LLM が自然言語指示を解析して DAG 構築
            └─ 各 step: send_to(<conductor>) + shell(hestia-{domain}-cli)
                 └─ in-process Handler が成果物生成 + JSON 返却
       └─ LLM が集約 JSON を fs_write(.hestia/run_log/<run-id>.json)
  └─ hestia-ai-cli がファイル出現を検出 → stdout 出力 → exit code 決定
```

## status 値域（Phase 25/26/31 で確立）

### 汎用 status（exit_code 0 = aggregate ok）

| status | 意味 | 典型ケース |
|--------|-----|----------|
| `ok` | 期待通り完了 | hal.parse がレジスタマップを生成 |
| `started` | 非同期ジョブ開始済（dry-run） | `--execute` なしで TCL のみ生成 |
| `skipped` | 入力不在で何もせず（正常経路） | RTL ソースなしで rtl.lint をスキップ |
| `tool_unavailable` | 実ツール未インストール（環境依存） | verilator / vivado / probe-rs 不在 |
| `input_required` | `--execute` 要求あり、必須入力不足 | `fpga.build artix7 --execute` で RTL/制約/part 不足 |

### 汎用 status（exit_code 1 = aggregate error）

| status | 意味 |
|--------|-----|
| `build_failed` / `program_failed` | 実ツールが exit ≠ 0 |
| `error` | handler 内部 fatal error |

### ドメイン固有 status（exit_code 0 = aggregate ok、レポート上は失敗を示す）

| handler | status | 意味 |
|---------|--------|------|
| `rtl.lint.v1` | `lint_failed` | linter が違反検出（diagnostics > 0、project-side コード品質課題）|
| `rtl.simulate.v1` | `sim_failed` | シミュレーション失敗（testbench 修正対象）|
| `debug.uart_loopback` | `sent` | write 成功（read_back なし）|
| `debug.uart_loopback` | `no_response` | read_back タイムアウト（プロジェクト側 RTL 課題）|
| `debug.uart_loopback` | `mismatch` | バイト列不一致（テスト失敗）|
| `debug.uart_loopback` | `write_failed` | シリアル write 失敗（環境失敗）|
| `fpga.program` | `ready` | execute=false、bitstream 揃っている dry-run |

## 入力完備性ゲート（`--execute` 時）

`--execute=true` を渡した step は実ツールが invoke される直前に **入力完備性** をチェックします:

| handler | 必須入力 | 不足時 |
|---------|---------|-------|
| `fpga.build.v1.start` | RTL sources + XDC constraints + part number | `input_required`（Vivado 不呼出）|
| `fpga.program` | bitstream | `input_required` |
| `debug.uart_loopback` | device 存在 | `tool_unavailable` |

入力不足時は **Vivado batch 等を起動しない** ため、リソースの無駄遣いを回避。response の診断フィールド (`inputs_complete` / `rtl_sources_count` / `constraints_present` / `part_resolved` / `bitstream_present`) で何が足りないかを即座に確認できます。

## 成果物の保存場所

成果物はプロジェクトルート配下の **内容適合ディレクトリ** に書き出されます（Phase 20 規約）:

```
<root>/                      ← hestia init を実行したディレクトリ
├── instructions.md
├── hal/                     ← HAL レジスタマップ
│   └── register_map.json
├── rtl/                     ← HDL ソース
│   ├── uart_led.sv
│   └── tb_uart_led.sv
├── sim/                     ← Lint / Sim レポート
│   ├── lint_report.json
│   ├── lint.log
│   ├── sim_report.json
│   └── sim.log
├── fpga/
│   ├── constraints/         ← XDC / SDC
│   ├── scripts/             ← TCL
│   ├── reports/             ← ビルドレポート
│   └── output/              ← .bit / .bin（実 invoke 時）
├── debug/                   ← デバッグセッション・波形
└── .hestia/                 ← Hestia 内部メタデータのみ
    ├── workspaces/<conductor>/agent.log
    ├── run_log/<run-id>.json
    └── personas/<conductor>.md
```

## 成果物の解決順序（Phase 42 — テンプレートなし）

handler は以下の順序で入力を解決します:

1. params で明示指定（`{"sources": [...]}`, `{"constraints": "..."}`, `{"part": "..."}` 等）
2. プロジェクトルート直下の既存ファイル（`<root>/rtl/<top>.sv`, `<root>/fpga/constraints/*.xdc` 等 — **AI orchestrator が `fs_write` で書いた成果物**）
3. 解決不可 → status: `input_required`（hal.parse 等）または `skipped`（fpga.build dry-run 等）

**Phase 42 で `<root>/.hestia/<domain>/templates/` のテンプレートディレクトリは完全廃止**。Hestia は AI 駆動システムであり、設計の生成責任は常に AI orchestrator にあります。テンプレートに頼らず、LLM 推論で成果物を生成してください。

## エラー診断

### `aborted_reason: "timeout"` の synthetic JSON

LLM が agent-cli の `max_iterations = 8` に到達して fs_write しなかった場合、`hestia-ai-cli` が以下の synthetic レスポンスを生成します:

```json
{
  "run_id": "...",
  "status": "error",
  "aborted_reason": "timeout",
  "aborted_message": "AI conductor LLM did not write ... within Ns. ...max_iterations = 8...",
  "synthesized_by": "hestia-ai-cli"
}
```

対処: ペルソナの「5 ステップ以下=send_to+shell、6 以上=shell のみ」規則に従いステップ数を絞る、または agent-cli 側で `max_iterations` を引き上げる（upstream PR が必要）。

### LLM が claude プロバイダで API クレジット切れ

`~/.config/agent-cli/config.toml` の `provider = "claude"` が `.hestia/config.toml` の `backend = "ollama"` を上書きする問題は **Phase 24 で解消済**。`hestia start` が `agent-cli run --provider <backend> --model <model>` を明示的に渡すため、現在は `.hestia/config.toml` の設定が確実に反映されます。

## 関連リソース

- [README.md](./README.md) — Hestia 全体像
- [.aiprj/AI_PRJ_DESIGN.md](./.aiprj/AI_PRJ_DESIGN.md) — Phase 1-32 の設計記録
- [.hestia/personas/ai.md](./.hestia/personas/ai.md) — AI Workflow Orchestrator ペルソナ仕様
