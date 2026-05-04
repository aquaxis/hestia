# Hestia 全 conductor エージェント階層表 — report_agent_map.md

**作成日**: 2026-05-04
**対象**: Hestia システムの全 9 conductor + 全 43 サブエージェント = 計 52 エージェント
**調査根拠**:
- 設計仕様書 `.hestia/design/hestia_design.md` 表 HD-030〜HD-038
- ペルソナ `.hestia/personas/*.md`（YAML frontmatter `name` / `role` / `description`）
- Phase 14 サブエージェントペルソナ作成完了状態（`.aiprj/AI_PRJ_REQUIREMENTS.md` §10.3）

**多重度凡例**: `1` = 常駐 1 instance / `1*` = 常駐 1（高負荷時 N 並列可）/ `N` = 動的並列起動 / `1/N` = target/言語/モジュールごとに並列化

---

## 1. 全体階層ツリー

```
Hestia システム
│
├── ai-conductor (peer "ai")  ★ 人間との唯一の入口（§2.2 表 HD-002）
│   ├── ai-planner (peer "ai-planner")                  [1]
│   └── ai-designer (peer "ai-designer")                [1]
│
├── rtl-conductor (peer "rtl")
│   ├── rtl-planner (peer "rtl-planner")                [1]
│   ├── rtl-designer (peer "rtl-designer")              [1]
│   ├── rtl-coder (peer "rtl-coder-{module}")           [N — モジュール数だけ動的起動、最大 16]
│   └── rtl-tester (peer "rtl-tester")                  [1]
│
├── fpga-conductor (peer "fpga")
│   ├── fpga-planner (peer "fpga-planner")              [1]
│   ├── fpga-designer (peer "fpga-designer")            [1]
│   ├── fpga-synthesizer (peer "fpga-synthesizer")      [1/N — target 並列時 N]
│   ├── fpga-implementer (peer "fpga-implementer")      [1/N — target 並列時 N]
│   ├── fpga-tester (peer "fpga-tester")                [1]
│   └── fpga-programmer (peer "fpga-programmer")        [1]   ※ペルソナのみ／HD-032 未掲載
│
├── asic-conductor (peer "asic")
│   ├── asic-planner (peer "asic-planner")              [1]
│   ├── asic-designer (peer "asic-designer")            [1]
│   ├── asic-synthesizer (peer "asic-synthesizer")      [1]
│   ├── asic-implementer (peer "asic-implementer")      [1]
│   ├── asic-signoff-checker (peer "asic-signoff")      [1]
│   └── asic-tester (peer "asic-tester")                [1]   ※ペルソナのみ／HD-033 未掲載
│
├── pcb-conductor (peer "pcb")
│   ├── pcb-planner (peer "pcb-planner")                [1]
│   ├── pcb-designer (peer "pcb-designer")              [1]
│   ├── pcb-schematic (peer "pcb-schematic")            [1]
│   ├── pcb-layout (peer "pcb-layout")                  [1]
│   └── pcb-tester (peer "pcb-tester")                  [1]
│
├── hal-conductor (peer "hal")
│   ├── hal-planner (peer "hal-planner")                [1]
│   ├── hal-designer (peer "hal-designer")              [1]
│   ├── hal-coder (peer "hal-coder-{lang}")             [N — c / rust / python / svd 等、出力言語数だけ並列]
│   └── hal-validator (peer "hal-validator")            [1]
│
├── apps-conductor (peer "apps")
│   ├── apps-planner (peer "apps-planner")              [1]
│   ├── apps-designer (peer "apps-designer")            [1]
│   ├── apps-coder (peer "apps-coder-{module}")         [N — モジュール数だけ動的起動、最大 16]
│   ├── apps-builder (peer "apps-builder")              [1]
│   └── apps-tester (peer "apps-tester")                [1]
│
├── debug-conductor (peer "debug")  ※ローカル専用（USB プローブ）
│   ├── debug-planner (peer "debug-planner")            [1]
│   ├── debug-designer (peer "debug-designer")          [1]
│   ├── debug-session-manager (peer "debug-session")    [1/N — target ごとに並列可]
│   ├── debug-analyzer (peer "debug-analyzer")          [1]
│   └── debug-programmer (peer "debug-programmer")      [1]
│
└── rag-conductor (peer "rag")
    ├── rag-planner (peer "rag-planner")                [1]
    ├── rag-designer (peer "rag-designer")              [1]
    ├── rag-ingest (peer "rag-ingest-{source}")         [N — ソース数だけ並列]
    ├── rag-search (peer "rag-search")                  [1*]
    ├── rag-quality (peer "rag-quality")                [1]
    └── rag-archivist (peer "rag-archivist")            [1*]
```

---

## 2. Conductor 別 サブエージェント一覧表

### 2.1 ai-conductor（メタオーケストレーター）— 設計 §3.10 / 表 HD-030

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `ai-planner` | 1 | Hestia AI planner — タスク分解・実行計画・DAG 構築 | `.hestia/personas/ai-planner.md` |
| **designer** | `ai-designer` | 1 | Hestia AI designer — 仕様設計・HW/SW 統合トップレベル設計 | `.hestia/personas/ai-designer.md` |

**conductor 本体ペルソナ**: `.hestia/personas/ai.md` — Hestia メタオーケストレーター — 全 conductor を統括する AI Workflow Orchestrator

---

### 2.2 rtl-conductor — 設計 §4.8 / 表 HD-031

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `rtl-planner` | 1 | RTL planner — RTL 設計フローの計画・スケジューリング | `.hestia/personas/rtl-planner.md` |
| **designer** | `rtl-designer` | 1 | RTL designer — RTL 設計・アーキテクチャ定義 | `.hestia/personas/rtl-designer.md` |
| **coder** | `rtl-coder-{module}` | **N** | RTL coder — RTL コード生成・モジュール実装（モジュールごとに動的並列起動）| `.hestia/personas/rtl-coder.md`（共通テンプレート、起動時に `--name rtl-coder-<module>` で個別 peer 名を割当）|
| **tester** | `rtl-tester` | 1 | RTL tester — RTL テストベンチ作成・検証 | `.hestia/personas/rtl-tester.md` |

**conductor 本体ペルソナ**: `.hestia/personas/rtl.md` — RTL conductor — RTL 設計フローを管理する AI エージェント

---

### 2.3 fpga-conductor — 設計 §5.x / 表 HD-032

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `fpga-planner` | 1 | FPGA planner — FPGA 設計フローの計画・スケジューリング | `.hestia/personas/fpga-planner.md` |
| **designer** | `fpga-designer` | 1 | FPGA designer — FPGA アーキテクチャ設計 | `.hestia/personas/fpga-designer.md` |
| **synthesizer** | `fpga-synthesizer` | 1/N | FPGA synthesizer — FPGA 論理合成・最適化（target 並列時 N） | `.hestia/personas/fpga-synthesizer.md` |
| **implementer** | `fpga-implementer` | 1/N | FPGA implementer — FPGA 配置配線・インプリメンテーション（target 並列時 N） | `.hestia/personas/fpga-implementer.md` |
| **tester** | `fpga-tester` | 1 | FPGA tester — FPGA テスト・検証 | `.hestia/personas/fpga-tester.md` |
| **programmer** ※ | `fpga-programmer` | 1 | FPGA programmer — FPGA デバイスプログラミング（ペルソナのみ存在、表 HD-032 未掲載）| `.hestia/personas/fpga-programmer.md` |

**conductor 本体ペルソナ**: `.hestia/personas/fpga.md` — FPGA conductor — FPGA 設計フローを管理する AI エージェント

> ※ **設計仕様とペルソナの差分**: `fpga-programmer` は表 HD-032（行 1735-1742）に列挙されていないが、ペルソナファイルは Phase 14 で作成済（`.hestia/personas/fpga-programmer.md`）。`fpga.program --execute`（Phase 21 追加）の実起動主体として運用上必要なため、ペルソナ側は実態を反映している。設計仕様書の HD-032 に追記して整合させるのが理想（Phase 56 候補として登記可能）。

---

### 2.4 asic-conductor — 設計 §6.x / 表 HD-033

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `asic-planner` | 1 | ASIC planner — ASIC 設計フローの計画・スケジューリング | `.hestia/personas/asic-planner.md` |
| **designer** | `asic-designer` | 1 | ASIC designer — ASIC アーキテクチャ設計 | `.hestia/personas/asic-designer.md` |
| **synthesizer** | `asic-synthesizer` | 1 | ASIC synthesizer — ASIC 論理合成・最適化 | `.hestia/personas/asic-synthesizer.md` |
| **implementer** | `asic-implementer` | 1 | ASIC implementer — ASIC 物理設計・配置配線 | `.hestia/personas/asic-implementer.md` |
| **signoff_checker** | `asic-signoff` | 1 | ASIC signoff checker — ASIC サインオフ検証 | `.hestia/personas/asic-signoff-checker.md` |
| **tester** ※ | `asic-tester` | 1 | ASIC tester — ASIC テスト・検証（ペルソナのみ存在、表 HD-033 未掲載）| `.hestia/personas/asic-tester.md` |

**conductor 本体ペルソナ**: `.hestia/personas/asic.md` — ASIC conductor — ASIC 設計フローを管理する AI エージェント

> ※ **設計仕様とペルソナの差分**: `asic-tester` は表 HD-033（行 1967-1974）に列挙されていないが、ペルソナは Phase 14 で作成済。テストパターン生成・検証フローが ASIC Tape-out 工程で必要なためペルソナ側は実態反映。
> ※ **peer 名規約の差分**: 設計仕様書の表 HD-033 では `asic-signoff` と短縮形を peer 名としているが、ペルソナファイル名は `asic-signoff-checker.md`。実運用時の `--name` 引数は **設計仕様書側（peer 名 `asic-signoff`）** が正規（`agent-cli list` での discoverability）。

---

### 2.5 pcb-conductor — 設計 §7.x / 表 HD-034

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `pcb-planner` | 1 | PCB planner — PCB 設計フローの計画・スケジューリング | `.hestia/personas/pcb-planner.md` |
| **designer** | `pcb-designer` | 1 | PCB designer — PCB アーキテクチャ設計 | `.hestia/personas/pcb-designer.md` |
| **schematic** | `pcb-schematic` | 1 | PCB schematic — 回路図生成・AI 支援合成 | `.hestia/personas/pcb-schematic.md` |
| **layout** | `pcb-layout` | 1 | PCB layout — PCB レイアウト・配線 | `.hestia/personas/pcb-layout.md` |
| **tester** | `pcb-tester` | 1 | PCB tester — PCB テスト・検証 | `.hestia/personas/pcb-tester.md` |

**conductor 本体ペルソナ**: `.hestia/personas/pcb.md` — PCB conductor — PCB 設計フローを管理する AI エージェント

---

### 2.6 hal-conductor — 設計 §8.x / 表 HD-035

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `hal-planner` | 1 | HAL planner — HAL 設計フローの計画・スケジューリング | `.hestia/personas/hal-planner.md` |
| **designer** | `hal-designer` | 1 | HAL designer — HAL アーキテクチャ設計 | `.hestia/personas/hal-designer.md` |
| **coder** | `hal-coder-{lang}` | **N** | HAL coder — 言語ごとのドライバコード生成（c / rust / python / svd 等、出力言語数だけ並列）| `.hestia/personas/hal-coder.md`（共通テンプレート、起動時に `--name hal-coder-<lang>` で個別 peer 名を割当）|
| **validator** | `hal-validator` | 1 | HAL validator — HAL 検証・差分確認 | `.hestia/personas/hal-validator.md` |

**conductor 本体ペルソナ**: `.hestia/personas/hal.md` — HAL conductor — HAL（Hardware Abstraction Layer）生成を管理する AI エージェント

---

### 2.7 apps-conductor — 設計 §9.x / 表 HD-036

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `apps-planner` | 1 | Apps planner — ファームウェア開発フローの計画・スケジューリング | `.hestia/personas/apps-planner.md` |
| **designer** | `apps-designer` | 1 | Apps designer — ファームウェアアーキテクチャ設計 | `.hestia/personas/apps-designer.md` |
| **coder** | `apps-coder-{module}` | **N** | Apps coder — 機能モジュール単位のアプリケーションコード（C / C++ / Rust）実装（最大 16 並列）| `.hestia/personas/apps-coder.md`（共通テンプレート、起動時に `--name apps-coder-<module>` で個別 peer 名を割当）|
| **builder** | `apps-builder` | 1 | Apps builder — クロスコンパイル / リンカスクリプト適用 / バイナリサイズ最適化 | `.hestia/personas/apps-builder.md` |
| **tester** | `apps-tester` | 1 | Apps tester — SIL（QEMU）/ HIL（実機）/ 単体テスト + カバレッジ集計 | `.hestia/personas/apps-tester.md` |

**conductor 本体ペルソナ**: `.hestia/personas/apps.md` — Apps conductor — ファームウェア / アプリケーション開発フローを管理する AI エージェント

---

### 2.8 debug-conductor — 設計 §10.x / 表 HD-037

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `debug-planner` | 1 | Debug planner — デバッグセッション計画・スケジューリング | `.hestia/personas/debug-planner.md` |
| **designer** | `debug-designer` | 1 | Debug designer — デバッグ戦略設計 | `.hestia/personas/debug-designer.md` |
| **session_manager** | `debug-session` | 1/N | Debug session manager — デバッグセッション管理・制御（target ごとに並列可）| `.hestia/personas/debug-session-manager.md` |
| **analyzer** | `debug-analyzer` | 1 | Debug analyzer — 信号解析・波形分析 | `.hestia/personas/debug-analyzer.md` |
| **programmer** | `debug-programmer` | 1 | Debug programmer — デバイスプログラミング・フラッシュ書き込み | `.hestia/personas/debug-programmer.md` |

**conductor 本体ペルソナ**: `.hestia/personas/debug.md` — Debug conductor — デバッグ・検証フローを管理する AI エージェント（**ローカル専用**、USB プローブアクセス）

> ※ **peer 名規約の差分**: 設計仕様書の表 HD-037 では `debug-session` と短縮形を peer 名としているが、ペルソナファイル名は `debug-session-manager.md`。実運用時の `--name` 引数は設計仕様書側（peer 名 `debug-session`）が正規。

---

### 2.9 rag-conductor — 設計 §13.7.x / 表 HD-038

| サブエージェント | peer 名 | 多重度 | 役割（YAML role）| ペルソナファイル |
|----------------|---------|-------|-----------------|----------------|
| **planner** | `rag-planner` | 1 | RAG planner — ドキュメント検索・管理フローの計画 | `.hestia/personas/rag-planner.md` |
| **designer** | `rag-designer` | 1 | RAG designer — ドキュメント構造設計・インデックス設計 | `.hestia/personas/rag-designer.md` |
| **ingest** | `rag-ingest-{source}` | **N** | RAG ingest — ドキュメントインジェスト・チャンキング・エンベディング（ソースごとに動的並列）| `.hestia/personas/rag-ingest.md`（共通テンプレート、起動時に `--name rag-ingest-<source>` で個別 peer 名を割当）|
| **search** | `rag-search` | 1* | RAG search — セマンティック検索・類似設計検索（高負荷時 N）| `.hestia/personas/rag-search.md` |
| **quality_gate** | `rag-quality` | 1 | RAG quality gate — インジェスト品質検証 | `.hestia/personas/rag-quality.md` |
| **archivist** | `rag-archivist` | 1* | RAG archivist — インデックス管理・クリーンアップ + 自己学習 conductor-work-logs 蓄積（高負荷時 N）| `.hestia/personas/rag-archivist.md` |

**conductor 本体ペルソナ**: `.hestia/personas/rag.md` — RAG conductor — ナレッジベース検索・管理を行う AI エージェント

---

## 3. 集計サマリ

### 3.1 conductor + サブエージェント数

| Conductor | 本体 | サブエージェント | 計 | 設計表 |
|-----------|------|----------------|----|--------|
| ai | 1 | 2 (planner / designer) | 3 | HD-030 |
| rtl | 1 | 4 (planner / designer / coder(N) / tester) | 5 | HD-031 |
| fpga | 1 | 6 (planner / designer / synthesizer / implementer / tester / programmer※) | 7 | HD-032 |
| asic | 1 | 6 (planner / designer / synthesizer / implementer / signoff_checker / tester※) | 7 | HD-033 |
| pcb | 1 | 5 (planner / designer / schematic / layout / tester) | 6 | HD-034 |
| hal | 1 | 4 (planner / designer / coder(N) / validator) | 5 | HD-035 |
| apps | 1 | 5 (planner / designer / coder(N) / builder / tester) | 6 | HD-036 |
| debug | 1 | 5 (planner / designer / session_manager / analyzer / programmer) | 6 | HD-037 |
| rag | 1 | 6 (planner / designer / ingest(N) / search / quality_gate / archivist) | 7 | HD-038 |
| **計** | **9** | **43** | **52** | — |

ペルソナファイル数 (`ls .hestia/personas/ \| wc -l`) = **52** で完全一致。

### 3.2 サブエージェント役割の共通パターン

全 9 conductor で共通する役割パターン:

| 役割 | 出現 conductor | 多重度 | 共通の責務 |
|-----|--------------|-------|-----------|
| **planner** | 全 9 | 1 | 開発フローの計画・スケジューリング・DAG 化 |
| **designer** | 全 9 | 1 | 詳細仕様 / アーキテクチャ設計 |
| **coder** / **coder-like**（synthesizer / implementer / schematic / layout / ingest / session_manager 等）| 全 9 | 1 または N | 実装ステップ実行（ドメイン固有のコード / バイナリ / 成果物生成）|
| **tester / quality_gate / signoff_checker / validator / analyzer** | 全 9 | 1 | 検証・品質保証 |
| **builder / programmer / archivist**（特定 conductor のみ）| apps / fpga / debug / rag | 1 | デプロイ・書込・蓄積（terminal step）|

### 3.3 動的並列起動（多重度 N）のサブエージェント

| Conductor | サブエージェント | peer 名規約 | スケーリング |
|-----------|----------------|------------|------------|
| rtl | rtl-coder | `rtl-coder-{module}` | モジュール数だけ並列、最大 16 |
| hal | hal-coder | `hal-coder-{lang}` | 出力言語数（c / rust / python / svd 等）|
| apps | apps-coder | `apps-coder-{module}` | モジュール数だけ並列、最大 16 |
| rag | rag-ingest | `rag-ingest-{source}` | 取り込みソース数 |
| fpga | fpga-synthesizer | `fpga-synthesizer` | target 並列時のみ N |
| fpga | fpga-implementer | `fpga-implementer` | target 並列時のみ N |
| debug | debug-session-manager | `debug-session` | target ごとに並列可 |

---

## 4. 実装状態（現行 Hestia ランタイム）

| 観点 | 状態 |
|-----|-----|
| ペルソナファイル配置 (`.hestia/personas/*.md`) | ✅ 52 件すべて存在（Phase 14 完了）|
| 設計仕様書 §3.10 / §4.8 / §5.x〜§13.7.x | ✅ サブエージェント階層・並列開発フロー・起動コマンド例まで記述済 |
| ai-conductor `agent_spawn` 実装 | ⚠ 部分実装 — `AgentManager::spawn` (`hestia-ai-conductor/crates/multi-agent/src/agent_manager.rs:69-118`) は存在するが、明示的な API 要求 (`agent_spawn` メソッド呼出) でのみ起動。自動振り分けではない |
| 各 conductor handler からのサブエージェント起動 | ❌ 未実装 — 8 conductor (rtl/fpga/asic/pcb/hal/apps/debug/rag) の handler に `spawn`/`agent-cli run` 経路 0 件（Phase 51 §3.4 既調査）|
| Workflow Orchestrator 経路 | ❌ 経由しない — ai persona は `shell` ツールで `hestia-{domain}-cli` を起動し、CLI 内で in-process Handler 実行（Phase 16 §12.2.2）|
| サブエージェント間 IPC (`agent-cli send <sub-peer>`) | ❌ 経路なし |

→ **設計上は階層が完備、実装上は未活性**（report_qa.md Q4 と同じ結論）。

---

## 5. 次 Phase 候補（既登記）

`report_qa.md` 末尾に登記済の候補:

- **Phase 52**: 各 conductor handler に `agent_spawn` 経路を追加し、ai-conductor 由来の指示を sub-agent (planner / designer / coder / tester) に分解・割当する経路を実装

本表でさらに細分化して候補を提示する場合:

- **Phase 52a**: 全 9 conductor で planner / designer の常駐起動を実装（多重度 1 のみ）。ペルソナは既存利用
- **Phase 52b**: rtl / hal / apps の `coder-{module/lang}` 動的並列起動を実装（多重度 N、`--name` 動的割当が必要）
- **Phase 52c**: fpga / asic の synthesizer / implementer / signoff_checker / tester / programmer 各役割の起動と協調
- **Phase 52d**: debug の session_manager の target 並列、rag の ingest の source 並列、search / archivist の高負荷時 N

これらは本 Phase 51（調査と報告書作成）の **スコープ外** であり、ユーザーの追加指示なしには実装しない。

---

## 6. 既知の差分・整合性課題（Phase 56 で整合化済）

**Phase 56 完了（2026-05-04）**: 設計仕様書再精査の結果、当初 Phase 52 で報告した 4 件のうち 2 件（`fpga-programmer` / `asic-tester` の HD 表掲載）は **誤判定**（表末尾を読み切れていなかったため）であり、実際には設計仕様書側に既に存在していた。残り 2 件（peer 名 ↔ ファイル名乖離）は Phase 56 で `.hestia/design/hestia_design.md` §3.11「peer 名とペルソナファイル名の対応規約」（表 HD-039a）として明文化済。

| 差分 | 当初判定 | 実際の状態 | Phase 56 対応 |
|-----|---------|-----------|--------------|
| `fpga-programmer` 行 | HD-032 に未掲載 | **誤判定** — HD-032 行 1744 に既に掲載済 | 対応不要（誤判定の訂正のみ）|
| `asic-tester` 行 | HD-033 に未掲載 | **誤判定** — HD-033 行 1976 に既に掲載済 | 対応不要（誤判定の訂正のみ）|
| `asic-signoff` peer 名 vs `asic-signoff-checker.md` | 規約乖離 | 設計上は peer 名 `asic-signoff` が正規、ペルソナファイル名は記述的命名 | §3.11 表 HD-039a で対応規約を明文化 ✅ |
| `debug-session` peer 名 vs `debug-session-manager.md` | 規約乖離 | 同上 | §3.11 表 HD-039a で対応規約を明文化 ✅ |

**規約のポイント** (§3.11 より):
- `agent-cli list` / `agent-cli send <peer>` で使用する peer 名は **設計上の peer 名（表 HD-030〜HD-038）が正規**
- ペルソナファイル名は役割を端的に表す独立命名として存在
- 起動時 `agent-cli run --persona-file <ファイル名>.md --name <peer 名>` の分離指定で対応する

これにより設計仕様書とペルソナの整合化が完了し、Hestia アーキテクチャ文書は正規ソースとして一貫した状態になりました。

---

## 7. 結論

- Hestia は **9 conductor + 43 サブエージェント = 計 52 エージェント**で構成される設計
- 全 conductor が `planner / designer / *executor / *verifier` の 4 系統共通パターンを採用
- 多重度 N の動的並列起動は rtl / hal / apps / rag / (fpga / debug) で計 7 サブエージェントに採用
- ペルソナファイル 52 件は完備（Phase 14 完了）— 設計仕様書 HD-030〜038 とほぼ一致するが軽微な差分あり（§6 参照）
- **実装は ai-conductor の `agent_spawn` が条件付き起動を提供するのみで、各 conductor 階層は未活性**（report_qa.md Q4 結論を再確認）

---

**付記**: 本報告書は `<root>/report_qa.md`（Phase 51 設問 Q4 への回答）を補完する詳細階層図として作成された。Hestia core ソース改修は本セッションで一切行われていない。詳細は `.aiprj/AI_LOG/2026-05-04_006.md` を参照。
