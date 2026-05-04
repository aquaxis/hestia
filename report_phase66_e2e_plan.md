# Phase 66 E2E 実機検証手順書

**作成日**: 2026-05-04
**対象**: cloud LLM usage limit / ARTY-A7 USB JTAG 接続が復旧した時点で実施する Phase 53〜65 統合 E2E 検証の完全手順書
**位置づけ**: Phase 62 (`report_phase53_62_e2e.md`) と Phase 63 (`report_phase63_static_e2e.md`) で「環境制約により未実施」と記録した実 E2E 検証を、user が実機で再現可能な手順として書き下す。

---

## 1. 前提条件

### 1.1 環境準備

| 項目 | 必要 | 確認コマンド |
|-----|-----|-----------|
| agent-cli バイナリ | PATH に存在 | `which agent-cli` |
| LLM provider 利用可能 | usage limit 内 | `agent-cli list` で起動成功 |
| `hestia` バイナリ群 (19 個) | install 済 | `hestia status` |
| ARTY-A7-100T USB JTAG (Phase 50/65 段で必要) | 物理接続 | `lsusb \| grep -i digilent` |
| Vivado (任意) | fpga.build --execute で必要 | `vivado -version` |
| verilator (任意) | rtl.simulate.v1 で実呼出 | `verilator --version` |

### 1.2 テスト環境の clean 化

```bash
# 1. 旧プロセス全停止
pkill -9 'agent-cli' || true
pkill -9 'hestia' || true

# 2. registry 残骸除去
rm -rf "$XDG_RUNTIME_DIR/agent-cli/" 2>/dev/null

# 3. workspace clean
cd /home/hidemi/hestia-test/  # or any test root
rm -rf .hestia/workspaces/
```

---

## 2. Phase 53〜65 統合 E2E 検証シーケンス

### 2.1 Step A — `hestia init` + `hestia start`（Phase 14/55/57 統合）

```bash
cd /home/hidemi/hestia-test/
hestia init
hestia start
```

**期待される挙動**:
- `.hestia/workspaces/<peer>/.aiprj/instruction.md` placeholder が 27 件作成される（Phase 57 `init_aiprj_workspace`、9 conductor + 18 sub-agent = 27）
- `.hestia/workspaces/<peer>/.aiprj/rules` symlink → `<root>/.aiprj/rules/` が 27 件作成される（Phase 57）
- 9 conductor + 18 主要 sub-agent (planner/designer 各 9) = 27 agent-cli プロセス常駐起動（Phase 55 `RESIDENT_SUB_AGENTS`）
- 27 mirror プロセスが detached spawn（Phase 49）

**検証コマンド**:
```bash
# 全 27 peer が registry に登録されているか
agent-cli list | wc -l   # 期待: 27 + α (frontend peer 等)

# .aiprj/ skeleton が全件生成されているか
find .hestia/workspaces -name "instruction.md" | wc -l   # 期待: 27
find .hestia/workspaces -name "rules" -type l | wc -l    # 期待: 27（symlink）

# mirror プロセスが detached しているか
ps aux | grep 'hestia mirror' | grep -v grep | wc -l   # 期待: 27
```

### 2.2 Step B — persona 自己実行ループ起動確認（Phase 57b/61）

各 peer の agent-cli に最初の prompt が届いた時点（Phase 49 mirror で観測可）、persona の自己実行規約節に従って `.aiprj/instruction.md` を fs_read する挙動が起きるはず。

**検証コマンド**:
```bash
# ai persona の agent.log で fs_read .aiprj/instruction.md が観測されるか
hestia tail ai 2>&1 | grep -E "fs_read.*\.aiprj/instruction\.md" | head -3

# rtl-designer も同様
hestia tail rtl-designer 2>&1 | grep -E "fs_read.*\.aiprj/" | head -3
```

**期待される挙動（実 LLM 推論依存）**:
- persona の §5（自己実行規約）に従い、LLM が `fs_read .aiprj/instruction.md` を呼ぶ
- 空 instruction.md なので「通常業務へ遷移」の判断
- AI_LOG への自動記録は instruction が空のため発生しない

### 2.3 Step C — `hestia ai run --file instructions.md`（Phase 16/53/59 統合）

```bash
cat > instructions.md <<'EOF'
ARTY-A7-100T で UART を使用して LED を制御する回路を作成し、
シミュレーションと実機検証を行ってください。
EOF

hestia ai run --file instructions.md --timeout-secs 1200
```

**期待される挙動**:
- AiHandler::handle_exec が `spec_driven_emit_skeleton` で `<root>/.aiprj/AI_PRJ_REQUIREMENTS.md` / `DESIGN.md` / `TASKS.md` を best-effort 生成（Phase 59）
- ai persona ステップ 4 shell 起動規約に従い、各 conductor の `<domain>-cli design` を呼出（Phase 53/54）
- 各 design.v1 が `delegated`（designer alive）または `input_required`（designer offline）を返却（Phase 55b/55c/58）
- delegated の場合、handler が `agent-cli send <designer> <prompt>` で fire-and-forget dispatch（Phase 55c）
- ai-conductor LLM が designer の `expected_artifacts` ファイル fs_write 完了を観測後、後続の hal.parse / rtl.lint / fpga.build / fpga.program / debug.uart_loopback を順次起動（Phase 16）
- 集約 JSON が `.hestia/run_log/<run-id>.json` に書込（Phase 28/50）

**検証コマンド**:
```bash
# 3 文書 skeleton が生成されているか
ls -la .aiprj/AI_PRJ_*.md  # 期待: 3 ファイル

# run_log が出力されているか
ls -la .hestia/run_log/   # 期待: <run-id>.json

# aggregate JSON の構造
jq '{run_id, status, halted_reason, workflow_steps_count: (.workflow_steps | length), step_statuses: [.results[].status]}' .hestia/run_log/*.json | head -20
```

### 2.4 Step D — sub-agent 動的並列起動確認（Phase 60/60b/65）

ai-conductor が `<domain>.dispatch_coders.v1` 等を呼んだ場合、`hestia spawn-subagent` 経由で動的 sub-agent が起動するはず（Phase 55 `SpawnSubagent` + Phase 60+60b+65 dispatch）。

**検証コマンド**（ai persona が dispatch を呼んだ場合）:
```bash
# rtl-coder-{module} が動的起動されているか
agent-cli list | grep -E '^(rtl|hal|apps|rag|fpga|asic|pcb|debug)-(coder|ingest|synthesizer|implementer|signoff|session|schematic|layout|tester)-' | head

# mirror が sub-agent log を捕捉しているか
ls -la .hestia/workspaces/rtl-coder-* 2>/dev/null
```

### 2.5 Step E — 退行確認（Phase 50 sim_warnings 維持）

```bash
# rtl.simulate.v1 で sim_warnings が exit 0 で継続成功になるか
hestia rtl simulate --top uart_led_top
echo "exit code: $?"   # 期待: 0（warning が出ても sim_warnings 扱い）
```

---

## 3. 期待される完全達成シナリオ

| ステップ | 期待 status | 期待 exit code | 期待 artifact |
|---------|-----------|--------------|------------|
| 1. hal.design.v1 → hal-designer fs_write | delegated | 0 | hal/register_map.json |
| 2. hal.parse.v1 | ok | 0 | run_log entry |
| 3. rtl.design.v1 → rtl-designer fs_write | delegated | 0 | rtl/uart_led_top.sv, rtl/tb_uart_led_top.sv |
| 4. rtl.lint.v1 | ok or lint_failed (warn 扱い) | 0 | sim/lint.log |
| 5. rtl.simulate.v1 | ok or sim_warnings | 0 | sim/sim.log |
| 6. fpga.design.v1 → fpga-designer fs_write | delegated | 0 | fpga/constraints/uart_led_top.xdc, fpga/scripts/build.tcl |
| 7. fpga.build artix7 --execute | ok or build_failed | 0/1 | fpga/output/uart_led_top.bit (Vivado 実行時)|
| 8. fpga.program --execute | ok or program_failed | 0/1 | (ARTY-A7 接続時) bit 書込完了 |
| 9. debug.uart_loopback --execute | ok or device_unavailable | 0 | debug/loopback.log |
| 集約 | status: ok or partial | 0/1 | run_log/<run-id>.json |

---

## 4. 失敗時のトリアージ

### 4.1 cloud LLM usage limit エラー

```
[error] HTTP 429 ... weekly usage limit exceeded
```
→ 別 provider に切替（claude / codex / llama_cpp）。`.hestia/config.toml` の `[agent_cli] backend` を編集。

### 4.2 designer 不在 → 全 design.v1 が input_required

```bash
# RESIDENT_SUB_AGENTS の起動失敗確認
hestia tail rtl-designer --path-only   # log path が無いなら起動失敗
hestia start rtl-designer  # 個別 spawn 試行
```

### 4.3 ARTY-A7 USB JTAG 未接続 → fpga.program が program_failed

期待動作（Phase 50）。`halted_reason: "halt_on_error"` + `error_log_excerpt` で JTAG 未検出を明示報告される。

### 4.4 spec-driven 3 文書が生成されない

```bash
# HESTIA_DISABLE_SPEC_DRIVEN 環境変数が誤設定されていないか
env | grep HESTIA_DISABLE_SPEC_DRIVEN
# 設定されていれば unset
unset HESTIA_DISABLE_SPEC_DRIVEN
```

---

## 5. Phase 53〜65 全達成項目の E2E チェックリスト

| Phase | 検証項目 | E2E 検証コマンド |
|-------|--------|--------------|
| 53 | ai persona 責務越境是正 | persona ファイル grep + agent.log 観測 |
| 54 | rtl/fpga/hal handler design.v1 | `hestia rtl design --instruction "..."` |
| 55 | sub-agent 27 プロセス常駐 | `agent-cli list \| wc -l` |
| 55b | designer 二値判定 | designer 停止して `hestia rtl design` 試行 |
| 55c | agent-cli send dispatch | `hestia tail rtl-designer` で peer_prompt 観測 |
| 56 | peer 名規約 | `agent-cli list \| grep -E '(asic-signoff\|debug-session)$'` |
| 57 | `.aiprj/` skeleton | `find .hestia/workspaces -name '.aiprj' -type d \| wc -l` = 27 |
| 57b | persona 自己実行（conductor）| `cat .hestia/workspaces/rtl/agent.log \| grep '\.aiprj/instruction\.md'` |
| 58 | 全 8 ドメイン design.v1 | 8 ドメインで `hestia <d> design` 試行 |
| 59 | spec-driven 3 文書 | `ls .aiprj/AI_PRJ_*.md` |
| 60 | rtl.dispatch_coders.v1 | rtl persona が dispatch を呼ぶ instruction で確認 |
| 60b | hal/apps/rag dispatch | 同上 |
| 61 | persona 自己実行（sub-agent）| 全 27 workspace の agent.log 確認 |
| 62 | E2E 静的検証 | この document |
| 63 | 環境非依存検証 | `cargo test --release` 88 件 pass |
| 64 | persona 整合性 | `grep "起動時の.*自己実行規約" personas/*.md \| wc -l` = 52 |
| 65 | 全 8 ドメイン dispatch | fpga.dispatch_targets / asic.dispatch_steps / pcb.dispatch_phases / debug.dispatch_sessions も実行可能 |

---

## 6. 結論

本 E2E 検証手順書は cloud LLM usage limit + ARTY-A7 USB JTAG 接続要件が解消した時点で user が手動実施するためのチェックリスト。Phase 53〜65 で構築した責務境界モデル + sub-agent 階層 + 自己実行統合の全機能が **実機で完全に動作する** ことを確認する根拠資料となる。

実 E2E 結果は完了次第 `<root>/report_phase66_e2e_results.md` として記録することを推奨。
