# Hestia システム調査 — report_qa.md

**作成日**: 2026-05-04
**対象**: `.aiprj/instructions.md`（2026-05-04 リセット版）「Hestia システムの調査」6 設問
**調査範囲**: 設計仕様書 (`.hestia/design/hestia_design.md`) / 全 52 ペルソナ (`.hestia/personas/`) / 9 conductor 実装 (`.hestia/tools/conductors/`) / 統合 CLI (`.hestia/tools/clis/`) / Phase 22 (`.aiprj/rules` 取込判断 P-1) / Phase 44 (ai-conductor 責務境界 7 領域 C-1)
**判定凡例**: ✅ YES / ❌ NO / ⚠ 部分 YES（設計と実装にギャップあり）

---

## Q1. Hestia システムで人間がアクセスするのは ai-conductor であるのは合っていますか？

**回答**: ✅ YES（合っています）

**根拠**:

1. **設計仕様書 §1.2 原則 8（統一インターフェース）** — `.hestia/design/hestia_design.md:73`
   > 「全 conductor 間および フロントエンド ↔ ai-conductor の通信を **agent-cli ネイティブ IPC** に統一する」

2. **設計仕様書 §2.1 アーキテクチャ図** — `.hestia/design/hestia_design.md:586`
   フロントエンド層から「agent-cli IPC (peer "ai")」を経由して **メタオーケストレーション層 (ai-conductor)** に接続する図示。フロントエンドから直接 fpga / rtl / asic 等の各 conductor に接続する経路は描かれていない。

3. **設計仕様書 §2.2 9 Conductor の役割** — `.hestia/design/hestia_design.md:644`
   > 「ai-conductor: メタオーケストレーター（全 conductor 統括／**人間との唯一の入口**）」と明記。

4. **実装根拠** — `.hestia/tools/clis/hestia/src/main.rs:80-84`
   ```rust
   /// Dispatch to hestia-ai-cli
   Ai {
       #[arg(trailing_var_arg = true)]
       args: Vec<String>,
   },
   ```
   `hestia ai run --file instructions.md` のサブコマンド経路は `hestia-ai-cli` を経由して ai-conductor に到達する設計。

**補足**:

開発者向けの低レベル経路として `hestia rtl lint` / `hestia fpga build` 等の各ドメイン直接コマンド（`Commands::Rtl` / `Commands::Fpga` 等、`main.rs:85〜`）も提供されている。これらは設計上も「CLI 体験の一部」(§2.1 表 HD-003 フロントエンド層) として許容されているが、**主要動線（自然言語指示・ワークフロー実行）は ai-conductor 経由が前提**。よってユーザー設問の「人間がアクセスするのは ai-conductor」は本質的に正しい。

---

## Q2. ai-conductor は人間からの指示を apps, hal, rtl, fpga, asic, pcb, debug の各 conductor の **サブエージェントのタスクに分けることではなく**、**各 conductor へ大まかに割り振ることが責務** なのは合っていますか？

**回答**: ✅ YES（合っています）

**根拠**:

1. **AiHandler の dispatch 経路** — `.hestia/tools/conductors/hestia-ai-conductor/src/handler.rs:237-265`
   ```rust
   async fn dispatch_to_conductor(
       config: &HestiaClientConfig,
       conductor: ConductorId,   // ← Conductor 単位の ID（Hal/Rtl/Fpga/...）
       method: &str,
       params: serde_json::Value,
   ) -> Result<serde_json::Value, String>
   ```
   引数 `conductor: ConductorId` は **conductor 単位** の識別子であり、`HalPlanner` / `RtlCoder` 等のサブエージェント識別子は型として存在しない。すなわち API レベルで「サブエージェント直接振り分け」は不可能。

2. **`build_workflow` のワークフロー要素** — 同 `handler.rs:48-234`
   ```rust
   steps.push(WorkflowStep {
       step: step_num,
       conductor: ConductorId::Hal,         // ← conductor 単位
       method: "hal.parse.v1".to_string(),
       ...
   });
   ```
   `ConductorId::Hal/Rtl/Fpga/Asic/Pcb/Apps/Debug/Rag` の 8 ターゲットしか存在せず、サブエージェント peer (`hal-planner`, `rtl-coder-uart` 等) は登場しない。

3. **設計仕様書 §3.3.1 タスク振り分けフロー** — `.hestia/design/hestia_design.md:916-955`
   > 「Step 3. 振り分け (routing via conductor-router) — 適切な peer に `agent-cli send <peer> <payload>`」
   表 HD-029（行 947〜955）の「振り分け先」列はすべて conductor 単位（fpga-conductor / rtl-conductor → fpga-conductor 等）でサブエージェント peer は記載されていない。

4. **ai persona 責務境界節** — `.hestia/personas/ai.md:198`
   > 「`hestia-ai-cli exec / spec.* / agent_* / container.* / system.*` 等の単一メソッド呼び出しは AiHandler が in-process 実行するため本ペルソナを経由しません。本ペルソナの責務は `hestia-ai-cli run --file` 経由の自然言語オーケストレーションのみ。」

5. **Phase 44 §37.4 (B) 既存調査結果** — `.aiprj/AI_PRJ_REQUIREMENTS.md` Phase 44 段落
   > 「ai-conductor は AiHandler / ai-core / workflow-engine / spec-driven / multi-agent / persona の 7 領域全てで C-1（責務境界合格）— 他 conductor のサブモジュールに直接振り分けする経路は **0 件**」

**補足**:

`ai-conductor` には例外的に「自身の」サブエージェント (`ai-planner` / `ai-designer`) を `agent_spawn` メソッドで起動する経路は存在する（`handler.rs:459-475`、`.hestia/design/hestia_design.md:1189-1199`）。しかしこれは ai-conductor の **内部委譲**（自分のプランニング / デザイニング能力を補強する目的）であって、**他 conductor のサブエージェントを直接呼び出すことではない**。設問が問う「他 conductor のサブエージェントへの直接振り分け」という意味では、AiHandler の API・実装ともに完全に分離されている。

---

## Q3. ai-conductor は各 conductor に大まかなタスクを割り振るということで合っていますか？

**回答**: ✅ YES（合っています）

**根拠**:

1. **設計仕様書 §3.3 メタオーケストレーション機能** — `.hestia/design/hestia_design.md:858-914`
   ai-conductor の機能ブロック図に `task-router` / `conductor-router` を明記:
   > 「conductor-router — 下流 conductor への agent-cli IPC ルーティング — 通信: `agent-cli send <peer> <payload>` — peer 名: rtl / fpga / asic / pcb / hal / apps / debug / rag」

2. **設計仕様書 §3.3.1 表 HD-029（振り分け例）** — `.hestia/design/hestia_design.md:947-955`
   | 入力 | 振り分け先 |
   |-----|----------|
   | "Vivado で artix7 用にビルドして" | fpga-conductor |
   | "RTL を lint して合成可能か確認して" | rtl-conductor → fpga-conductor |
   | `meta.dualBuild.v1` | workflow-engine 経由 / 複数 conductor |

3. **ai persona ステップ 4 shell 起動規約** — `.hestia/personas/ai.md:111-123`
   ```
   HESTIA_RUN_ID=<RUN_ID> hestia-hal-cli  --output json parse
   HESTIA_RUN_ID=<RUN_ID> hestia-rtl-cli  --output json lint
   HESTIA_RUN_ID=<RUN_ID> hestia-rtl-cli  --output json simulate
   HESTIA_RUN_ID=<RUN_ID> hestia-fpga-cli --output json build artix7
   HESTIA_RUN_ID=<RUN_ID> hestia-fpga-cli --output json program --execute
   HESTIA_RUN_ID=<RUN_ID> hestia-debug-cli --output json connect
   ```
   ai persona は各ドメイン CLI を「conductor 単位」で起動する。method (build / program / connect 等) は粒度の大きいオペレーションで、内部の細分タスクには立ち入らない。

4. **AiHandler `build_workflow`** — `.hestia/tools/conductors/hestia-ai-conductor/src/handler.rs:48-234`
   キーワード（UART / LED / lint / simulate / FPGA / ASIC / 検証 等）を検出して **conductor + method** のペアで step を構築する。各 method はその conductor の代表メソッド (`hal.parse.v1` / `rtl.lint.v1` / `fpga.build.v1.start` 等) であり、conductor 内の細部は受領側 conductor に委ねる。

**補足**:

「大まかな」の度合いは設計上 method 名前空間（`<conductor>.<operation>.v<N>` 形式、§14）の粒度で決まる。例えば `fpga.build.v1.start` は「FPGA ビルドを開始する」という大粒度タスクであり、合成 / 配置 / 配線 / ビットストリーム生成等の内訳は fpga-conductor が自身の state machine（§5.3）で展開する。よって ai-conductor は **大粒度のオペレーション境界で割り振る** という設問の表現と一致している。

---

## Q4. 各 conductor は ai-conductor からの指示を **各自のサブエージェントに割り振り** タスクを実行するということで合っていますか？

**回答**: ⚠ 部分 YES（**設計上は YES、現行実装は NO**）

**根拠**（設計）:

1. **設計仕様書 §4.8 rtl-conductor サブエージェント構成** — `.hestia/design/hestia_design.md:1336-1394`
   > 「rtl-conductor は **planner / designer / coder（複数）/ tester** の 4 種類のサブエージェントを持ち、planner や designer が作成した **機能モジュール単位に複数の coder を並列割当** することで RTL 開発を効率化する」
   並列開発フロー（行 1351〜1376）が明記され、rtl-conductor が rtl-planner / rtl-designer / rtl-coder-* / rtl-tester に `agent-cli send` で割り振る図が示されている。

2. **同様の設計が全 conductor に存在** — `.hestia/design/hestia_design.md`
   - §3.10 ai-conductor: planner / designer
   - §4.8 rtl-conductor: planner / designer / coder(N) / tester
   - §5（fpga）/ §6（asic）/ §7（pcb）/ §8（hal）/ §9（apps）/ §10（debug）/ §13.7（rag）にも同型のサブエージェント構成

3. **ペルソナファイルの存在** — `.hestia/personas/`（52 件）
   `ls .hestia/personas/` で `ai-{planner,designer}.md`、`rtl-{planner,designer,coder,tester}.md`、`fpga-{planner,designer,synthesizer,implementer,tester,programmer}.md` 等、全 conductor 分のサブエージェントペルソナが Phase 14 で作成済（43 サブエージェント）。

**根拠**（実装）:

1. **各 conductor handler にサブエージェント spawn 経路が存在しない** —
   `grep -rln "spawn\|agent-cli\|planner\|designer\|coder" .hestia/tools/conductors/`
   ```
   /work/home/hidemi/hestia/.hestia/tools/conductors/hestia-ai-conductor/src/handler.rs
   /work/home/hidemi/hestia/.hestia/tools/conductors/hestia-ai-conductor/crates/multi-agent/src/agent_manager.rs
   /work/home/hidemi/hestia/.hestia/tools/conductors/hestia-debug-conductor/crates/protocol-analyzer/src/lib.rs
   /work/home/hidemi/hestia/.hestia/tools/conductors/hestia-debug-conductor/crates/adapter-jtag/src/lib.rs
   ```
   ai-conductor を除く 8 conductor のメイン handler には sub-agent / planner / designer / coder への spawn・dispatch 経路が **実装されていない**（debug-conductor のヒットは JTAG プロトコル解析の文脈で「sub-agent」の意味ではない）。

2. **ai-conductor のみが `agent_spawn` を実装** — `.hestia/tools/conductors/hestia-ai-conductor/crates/multi-agent/src/agent_manager.rs:69-118`
   ```rust
   pub async fn spawn(&mut self, agent_id: String, conductor_id: String) -> Result<(), String> {
       ...
       let child = Command::new("agent-cli")
           .args([...])
           ...
   }
   ```
   `AgentManager::spawn` は ai-conductor の `agent_spawn` メソッド（`handler.rs:284, 459-475`）からのみ呼ばれ、明示的な API 要求でサブエージェントプロセスを起動する経路。**自動的にサブエージェントへ振り分ける機構ではない**。

3. **Workflow Orchestrator 経路は in-process Handler 直接実行** — Phase 16 §12.2.2 / Phase 17 §13.2.2 で明文化済
   > 「shell 経由 in-process 実行のため hal-conductor を経由しない」(`.aiprj/AI_PRJ_REQUIREMENTS.md` Phase 17 段落)
   ai-conductor の LLM が `shell` ツールで `hestia-{domain}-cli` を起動し、CLI 内で Handler が in-process 実行されるため、各 conductor 自身（agent-cli プロセス）も経由せず、当然サブエージェントへの振り分けも発生しない。

**ギャップサマリ**:

| 観点 | 設計 (`hestia_design.md` §3.10/§4.8/§5〜§13) | 実装 (`tools/conductors/`) |
|-----|----------------------------------------|--------------------|
| サブエージェントペルソナ | 52 件 (Phase 14 完了) | ✅ 配置済 |
| conductor handler からの spawn / dispatch | 必須 | ❌ 未実装（ai-conductor の `agent_spawn` のみ）|
| agent-cli IPC `agent-cli send <sub-peer>` | 設計図示あり | ❌ 経路なし |
| Workflow Orchestrator から conductor 経由 | 設計上は経由 | ❌ shell 経由 in-process が現実 |

**補足**:

設問は「設計上の責務境界」を問うているとも、「現行 Hestia の実動作」を問うているとも解釈できる。本回答では両解釈に応えるため部分 YES とし、ギャップを明示した。**正常進化の方向性としては Q4 = YES** が設計の意図であり、サブエージェント階層の活性化は将来 Phase の課題である（後述「次 Phase 候補」参照）。

---

## Q5. Hestia は `.aiprj/rules/setup_project.md` / `.aiprj/rules/update_project.md` / `.aiprj/rules/exec_job_project.md`（実体は `exec_job.md`）のルールを取り込めていますか？

**回答**: ⚠ 部分 NO（**設計上は §20.5 で取込が規定されているが、現行実装では取り込まれていない**）

**根拠**（設計上は取込予定）:

1. **設計仕様書 §20.5 aiprj ワークスペース統合** — `.hestia/design/hestia_design.md:5293-5295`
   > 「各 agent-cli プロセス（**9 conductor + 各 conductor のサブエージェント、計 50+ instance**）は、それぞれ専用ワークスペースディレクトリを持ち、初回起動時に [`aiprj`](https://github.com/aquaxis/aiprj) コマンドで AI プロジェクト管理環境を整備する。上位エージェント（フロントエンドや親 conductor）からの指示は各ワークスペースの `.aiprj/instruction.md` に記載され、agent-cli 自身が `.aiprj/rules/{setup_ai,update_ai,exec_job}.md` を順次自己実行する」

2. **設計仕様書 §20.5.1 ワークスペースレイアウト** — `.hestia/design/hestia_design.md:5299-5325`
   ```
   .hestia/workspaces/
   ├── ai/.aiprj/                # peer "ai" のワークスペース
   │   ├── instruction.md        # 上位からの指示
   │   ├── AI_PRJ_REQUIREMENTS.md
   │   ├── AI_PRJ_DESIGN.md
   │   ├── AI_PRJ_TASKS.md
   │   ├── AI_LOG/YYYY-MM-DD_NNN.md
   │   └── rules/{setup_ai,update_ai,exec_job,close_ai}.md
   ```

3. **設計仕様書 §20.5.2 初回起動時の初期化フロー** — `.hestia/design/hestia_design.md:5331-5344`
   > 「Step 2: 初回起動なら aiprj 環境を初期化（既存なら skip）— `curl -fsSL https://raw.githubusercontent.com/aquaxis/aiprj/main/install.sh | sh`」

**根拠**（現行実装は未取込）:

1. **Hestia ランタイムソース・ペルソナへの `.aiprj` 言及が 0 件** —
   ```
   $ grep -rln "aiprj\|setup_ai\|setup_project\|update_ai\|exec_job" \
        .hestia/personas/ .hestia/tools/
   (no output)
   ```
   全 52 ペルソナと 9 conductor + 共通クレートの Rust ソース・TypeScript ソースに **`.aiprj` 関連の参照が一切存在しない**。本リポジトリ直下の `.aiprj/` は Hestia プロジェクトを管理する **メタ AI（本セッション）専用**。

2. **Phase 22 §29.4〜§29.8 既存調査の結論（判断 P-1）** — `.aiprj/AI_PRJ_REQUIREMENTS.md` Phase 22 段落
   > 「全 52 ペルソナ + 9 conductor Rust ソースの `.aiprj` 言及を網羅調査し **0 件**を確認、判断 **P-1**（rules はプロジェクト管理 AI 専用）採択」

3. **`hestia start` / `hestia init` に `.aiprj` 初期化処理なし** — `.hestia/tools/clis/hestia/src/main.rs:203-348`
   `start_conductor` / `start_all_conductors` 関数は agent-cli を `--persona-file` と `--name` で起動するのみで、`curl install.sh` 等の `.aiprj` 初期化は実行していない。設計 §20.5.2 の Step 2（aiprj 初期化）はランタイムに組み込まれていない。

**ギャップサマリ**:

| 観点 | 設計 §20.5 | 現行実装 |
|-----|----------|--------|
| `.aiprj/` ワークスペース内自動生成 | 必須（初回起動時）| ❌ 未実装 |
| `setup_ai.md` / `update_ai.md` / `exec_job.md` 自己実行 | 必須 | ❌ 未実装 |
| 各 agent-cli の `AI_PRJ_REQUIREMENTS/DESIGN/TASKS.md` 生成 | 必須 | ❌ 未実装 |
| ペルソナでの `.aiprj` 参照 | 必須（指示読込元）| ❌ 0 件 |

**補足**:

設問の Q5 は「取り込めていますか？」と現状確認を問うているため **回答主軸は「現行は取り込めていない」(NO)**。ただし設計レベルで §20.5 として明記されているため「取込み計画は存在する」(設計 YES) という補足を加え、部分 NO と判定した。Phase 22 P-1 の趣旨は「**現行は** 取り込んでいない」を確認したものであり、設計の意図そのものを否定していない点に注意（記述上は「rules はプロジェクト管理 AI 専用」と要約しているが、これは設計 §20.5 が未実装である現実を踏まえた暫定運用ルール）。

設問本文は `exec_job_project.md` と記載されているが、リポジトリ実体は `.aiprj/rules/exec_job.md`（`.aiprj/rules/` ディレクトリ内に `setup_project.md` / `update_project.md` / `exec_job.md` / `close_ai.md` の 4 ファイルが存在）であることをあわせて記録する。

---

## Q6. ai-conductor と各 conductor は受け取った指示から最初は `setup_project.md` のルールと同様に **要件仕様書、設計仕様書、作業仕様書（実装タスクと作業指示）の作成** を行っていますか？

**回答**: ⚠ 部分 NO（**設計 §20.5.3 では要件 / 設計 / タスク 3 文書の自己生成が規定されているが、現行実装では行われていない**）

**根拠**（設計）:

1. **設計仕様書 §20.5.3 自己実行サイクル** — `.hestia/design/hestia_design.md:5359-5395`
   表 HD-039:
   | 規約 | 役割 | 生成 / 更新 |
   |-----|------|-----------|
   | `setup_ai.md` | プロジェクト初期化（**初回 + 構成変更時**）| `AI_PRJ_REQUIREMENTS.md` / `AI_PRJ_DESIGN.md` / `AI_PRJ_TASKS.md` の 3 文書を新規作成 |
   | `update_ai.md` | プロジェクト文書更新（上位指示変更時）| 既存 3 文書の改訂 |
   | `exec_job.md` | タスク実行（通常実行時）| `AI_PRJ_TASKS.md` の TODO を進行 |

2. **自己実行フロー** — `.hestia/design/hestia_design.md:5372-5395`
   > 「(a) `.aiprj/AI_PRJ_REQUIREMENTS.md` 不在 / 構成変更検出 → setup_ai.md 自己実行 → 要件定義 / 設計 / タスクリスト 3 文書を生成」

**根拠**（現行実装は未対応）:

1. **ai persona には 3 文書生成ステップが存在しない** — `.hestia/personas/ai.md:42-145`
   ai persona のステップ列:
   - ステップ 1: 指示解析（キーワード検出）
   - ステップ 2: ワークフロー DAG 構築
   - ステップ 3: **成果物の設計と fs_write**（HDL / xdc / build.tcl 等を直接生成）
   - ステップ 4: shell 起動
   - ステップ 5: status 値域 + halt-on-error
   - ステップ 6: 結果集約

   **「要件仕様書 / 設計仕様書 / 作業仕様書を最初に作成する」ステップが存在しない**。`fs_write` 対象は HDL ソース / xdc / TCL 等の実装成果物のみで、メタ文書 (`AI_PRJ_REQUIREMENTS.md` 等) は生成しない。

2. **AiHandler `handle_exec` も同様** — `.hestia/tools/conductors/hestia-ai-conductor/src/handler.rs:333-399`
   `build_workflow` → `dispatch_to_conductor` ループで実装系メソッドを直接実行するのみ。`AI_PRJ_REQUIREMENTS.md` 等の生成ステップは存在しない。

3. **`spec-driven` クレートは存在するが `ai.exec` で未活性** — `.hestia/tools/conductors/hestia-ai-conductor/src/handler.rs:402-456`
   ```rust
   async fn handle_spec_init(...) { ... SpecParser::parse(...) ... }
   async fn handle_spec_update(...) { ... }
   async fn handle_spec_review(...) { ... }
   ```
   `ai.spec.init/update/review` メソッドは存在するが、これは **明示的に呼ばれた場合のみ起動**（`hestia-ai-cli spec init` 等）。`ai.exec`（`hestia ai run --file`）の経路では呼ばれない。

4. **各 conductor handler も同様** — 例 `.hestia/tools/conductors/hestia-hal-conductor/src/handler.rs:12-200`
   `handle_init` / `handle_parse` / `handle_validate` / `handle_generate` / `handle_export` / `handle_diff` / `handle_status` の 7 メソッドのみ。要件 / 設計 / タスク 3 文書を生成する `handle_setup_ai` 相当のメソッドは存在しない。

**ギャップサマリ**:

| 観点 | 設計 §20.5.3 | 現行実装 |
|-----|------------|--------|
| ai-conductor が初回 instruction 受領で 3 文書生成 | 必須 | ❌ 未実装 |
| 各 conductor が初回 instruction 受領で 3 文書生成 | 必須 | ❌ 未実装 |
| `setup_ai` ⇄ `update_ai` ⇄ `exec_job` の自己判定ループ | 必須 | ❌ 未実装 |
| `spec-driven` クレート活用 | 必須 | ⚠ 存在するが `ai.exec` 経路では未使用 |

**補足**:

ai persona は `Phase 42`（テンプレート埋め込み禁止 + AI 駆動 design-first 化）以降「LLM が必要な成果物を `fs_write` で動的設計する」モデルを採用しているが、これは **個々の HW 設計ファイル**（HDL / xdc / TCL 等）に対する設計ファースト化であり、**メタレベルの要件 / 設計 / タスク 3 文書の自己生成**とは目的が異なる。設問 Q6 が指す「`setup_project.md` 同様の 3 文書作成」は §20.5.3 の `setup_ai.md` 自己実行に相当し、これは現状未実装。

---

## ギャップ分析と次 Phase 候補

| 設問 | 判定 | ギャップ概要 | 次 Phase 候補 |
|-----|-----|------------|------------|
| Q1 | ✅ YES | なし | — |
| Q2 | ✅ YES | なし（責務境界 C-1 達成済）| — |
| Q3 | ✅ YES | なし | — |
| Q4 | ⚠ 部分 YES | サブエージェント階層が設計上はあるが Workflow Orchestrator 経路で未活性 | **Phase 52 候補**: 各 conductor handler に `agent_spawn` 経路を追加し、ai-conductor 由来の指示を sub-agent (planner/designer/coder/tester) に分解・割当する経路を実装 |
| Q5 | ⚠ 部分 NO | 設計 §20.5 で `.aiprj/rules` 取込が規定されているが、ペルソナ・conductor 実装は未対応 | **Phase 53 候補**: `hestia start` の前段に `.hestia/workspaces/<peer>/.aiprj/` 自動初期化処理を追加し、ペルソナに `.aiprj/rules/setup_ai.md` 自己実行ステップを組み込む（Phase 22 P-1 を P-2「設計に合わせて実装側を進化」に上書き）|
| Q6 | ⚠ 部分 NO | 設計 §20.5.3 で 3 文書自己生成が規定されているが ai persona / 各 conductor handler に未実装 | **Phase 54 候補**: ai persona ステップ 0 に「instruction.md 受領 → 3 文書生成」を追加。`spec-driven` クレートを `ai.exec` パスから利用するよう改修 |

> **Phase 22 P-1 の再評価**: P-1 は「現行運用上 `.aiprj/rules` はプロジェクト管理 AI 専用とする」という暫定判断であり、設計 §20.5 の意図そのものを否定するものではない。Phase 53 / 54 によって P-1 を「設計に合わせて Hestia ランタイムへ統合」に進化させることが本質的解決と考えられる。ただし、これらは本 Phase 51 のスコープ外であり、ユーザーの追加指示がない限り実装は行わない。

---

## 結論サマリ

| 設問 | 一行回答 |
|-----|---------|
| Q1 | ✅ 人間アクセスは ai-conductor で正しい |
| Q2 | ✅ ai-conductor の責務は conductor 単位の大まか割り振りで正しい（サブエージェント直接振り分けは行わない）|
| Q3 | ✅ ai-conductor は各 conductor に大まかなタスクを割り振る |
| Q4 | ⚠ 設計上は各 conductor がサブエージェントへ割り振るが、現行実装の Workflow Orchestrator 経路では未活性（Phase 52 候補）|
| Q5 | ⚠ 設計 §20.5 で取込予定だが、現行は 0 件（Phase 53 候補）|
| Q6 | ⚠ 設計 §20.5.3 で 3 文書自己生成予定だが、現行は未実装（Phase 54 候補）|

**全体所感**: ai-conductor の責務境界（Q1〜Q3）は設計と実装が一致して合格状態。一方で **サブエージェント階層 (Q4)** および **`.aiprj/rules` 取込みによる自己ホスト型 AI プロジェクト管理 (Q5/Q6)** は設計仕様書 §3.10 / §4.8 / §20.5 に明記されているが、Hestia ランタイムへの実装は未着手。ユーザーが本領域の進化を意図する場合、Phase 52〜54 のいずれを優先するかを判断材料として提示する。

---

**付記**: 本報告書は `.aiprj/AI_PRJ_REQUIREMENTS.md` §44 / `.aiprj/AI_PRJ_DESIGN.md` §36 / `.aiprj/AI_PRJ_TASKS.md` Phase 51 で計画した調査の成果物。本セッション (`exec_job.md` 規約 1〜7 準拠) で作成。Hestia core ソース改修は行っていない。
