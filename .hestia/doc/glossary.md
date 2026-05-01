# 用語集（Glossary）

**対象領域**: Hestia 全体
**ソース**: 設計仕様書 §1〜§20 横断

---

## A

| 用語 | 読み方 | 定義 |
|------|--------|------|
| adapter.toml | アダプター・トムル | 新規ベンダーツールをコード変更なしで統合するための宣言型設定ファイル（§1.2 原則2）|
| AdapterRegistry | アダプター・レジストリ | Capability ベースのアダプター登録・解決エンジン（§2.2）|
| agent-cli | エージェント・クライ | Hestia の各 conductor が AI エージェントとして起動する Rust 製 CLI バイナリ。peer 間 IPC を提供（§20）|
| ai-conductor | AI コンダクター | メタオーケストレーター。全 conductor を統括し、人間との唯一の入口（§3）|
| apps-conductor | アップス・コンダクター | アプリケーションソフトウェア（FW / RTOS / ベアメタル）開発オーケストレーター（§9）|
| ASIC | エーシック | Application-Specific Integrated Circuit。特定用途向け集積回路（§6）|
| asic-conductor | エーシック・コンダクター | ASIC 設計フロー（RTL-to-GDSII 13ステップ）オーケストレーター（§6）|
| AsicToolAdapter | エーシック・ツール・アダプター | ASIC ツールの統一インターフェースを定義するトレイト（§6.4）|

## B

| 用語 | 読み方 | 定義 |
|------|--------|------|
| Backend Switching | バックエンド・スイッチング | LLM バックエンド（Claude / Codex / Ollama / llama.cpp）の切替機構（§20）|

## C

| 用語 | 読み方 | 定義 |
|------|--------|------|
| Capability | ケイパビリティ | アダプターが提供する機能の宣言。AdapterRegistry が Capability ベースでアダプターを解決（§2.2）|
| CLI | シーエルアイ | Command Line Interface。Hestia は統合 CLI（`hestia`）+ 各 conductor 個別 CLI（10種）を提供（§15）|
| Conductor | コンダクター | 各設計領域の専用オーケストレーター。Hestia では9種（ai / rtl / fpga / asic / pcb / hal / apps / debug / rag）が存在（§2.2）|
| ConductorId | コンダクター・アイディ | conductor を一意に識別する文字列。agent-cli の peer 名と同一（`ai` / `rtl` / `fpga` 等）（§2.3）|
| conductor-sdk | コンダクター・エスディーケー | conductor 共通 SDK クレート（§2.2）|
| Constraint Bridge | コンストレイント・ブリッジ | XDC ⇔ SDC ⇔ PCF ⇔ Efinity XML 間の制約ファイル変換サービス（§13.3）|
| container-manager | コンテナ・マネージャー | 全 conductor のコンテナライフサイクルを管理するモジュール（§12）|

## D

| 用語 | 読み方 | 定義 |
|------|--------|------|
| DAG | ダグ | Directed Acyclic Graph。有向非巡回グラフ。WorkflowEngine がワークフローを DAG として定義（§1.3.5）|
| debug-conductor | デバッグ・コンダクター | デバッグ環境オーケストレーター。ローカル実行専用（USB プローブアクセス用）（§10）|
| DesignSpec | デザイン・スペック | SpecParser が仕様書から生成する構造化設計仕様オブジェクト（§1.3.1）|
| DRC | ディーアールシー | Design Rule Check。設計ルールチェック（§6, §7）|

## E

| 用語 | 読み方 | 定義 |
|------|--------|------|
| Efinity | エフィニティ | Efinix 社の FPGA 開発ツール。Python API ベース（§5.7）|

## F

| 用語 | 読み方 | 定義 |
|------|--------|------|
| fpga-conductor | エフピージーエー・コンダクター | FPGA 設計フローオーケストレーター（§5）|
| fpga.lock | エフピージーエー・ロック | FPGA ビルドの完全再現を保証するロックファイル（§1.2 原則5）|

## G

| 用語 | 読み方 | 定義 |
|------|--------|------|
| GDSII | ジーディーエスツー | IC レイアウトの業界標準フォーマット。ASIC 最終出力（§6）|

## H

| 用語 | 読み方 | 定義 |
|------|--------|------|
| hal-conductor | ハル・コンダクター | Hardware Abstraction Layer 生成オーケストレーター（§8）|
| HDL | エイチディーエル | Hardware Description Language。ハードウェア記述言語（Verilog / VHDL / SystemVerilog）（§4）|
| Hestia | ヘスティア | 本プロジェクト名。Hardware Engineering Stack for Tool Integration and Automation（§1.5）|
| HumanReviewGate | ヒューマン・レビュー・ゲート | PatcherAgent のパッチ適用可否を信頼度スコアに基づき判定する機構（§1.3.7）|

## I

| 用語 | 読み方 | 定義 |
|------|--------|------|
| ILA | アイエルエー | Integrated Logic Analyzer。Xilinx 社のオンチップロジックアナライザ（§10）|
| IPC | アイピーシー | Inter-Process Communication。Hestia では agent-cli ネイティブ IPC に統一（§2.3）|

## J

| 用語 | 読み方 | 定義 |
|------|--------|------|
| JTAG | ジェイタグ | Joint Test Action Group。ボードレベル・チップレベルデバッグ用インターフェース（§10.5）|

## K

| 用語 | 読み方 | 定義 |
|------|--------|------|
| KiCad | キカッド | オープンソース PCB 設計ツール（§7.5）|

## L

| 用語 | 読み方 | 定義 |
|------|--------|------|
| LSP | エルエスピー | Language Server Protocol。HDL LSP Broker が svls / vhdl_ls / verilog-ams-ls を統合（§13.1）|
| LVS | エルブイエス | Layout Versus Schematic。レイアウトと回路図の一致検証（§6）|

## M

| 用語 | 読み方 | 定義 |
|------|--------|------|
| MCP | エムシーピー | Model Context Protocol。LLM から外部ツールを呼び出すためのプロトコル（§19）|

## N

| 用語 | 読み方 | 定義 |
|------|--------|------|
| nextpnr | ネクストピーエヌアール | オープンソース配置配線ツール。iCE40/ECP5/Gowin 対応（§18.1）|

## O

| 用語 | 読み方 | 定義 |
|------|--------|------|
| OpenLane 2 | オープンレーン・ツー | RTL-to-GDSII 自動化フレームワーク。Python ベース（§6.2）|
| Ollama | オラマ | ローカル LLM 実行エンジン。Hestia の OSS バックエンド（§18.5）|
| Orchestration | オーケストレーション | 複数 conductor にまたがるワークフローを自動制御する機構（§1.3.5）|

## P

| 用語 | 読み方 | 定義 |
|------|--------|------|
| PatcherAgent | パッチャー・エージェント | AI がベンダーツールの非互換パッチを自動生成するエージェント（§1.3.7）|
| PCB | ピーシービー | Printed Circuit Board。プリント基板（§7）|
| pcb-conductor | ピーシービー・コンダクター | PCB 設計フローオーケストレーター + AI 回路図生成（§7）|
| PDK | ピーディーケー | Process Design Kit。ASIC 製造プロセスの設計キット（Sky130 / GF180MCU）（§6.3）|
| Peer | ピア | agent-cli IPC ネットワーク上の通信エンドポイント。各 conductor が peer として参加（§2.3）|
| Podman | ポッドマン | Docker 代替の rootless コンテナランタイム（§11）|
| ProbeAgent | プローブ・エージェント | 新バージョンのベンダーツールでテストビルドを実行し非互換を検出するエージェント（§1.3.7）|

## Q

| 用語 | 読み方 | 定義 |
|------|--------|------|
| Quartus | クワータス | Intel 社の FPGA 開発ツール（§5.6）|

## R

| 用語 | 読み方 | 定義 |
|------|--------|------|
| rag-conductor | ラグ・コンダクター | 知識基盤の構築・管理・検索オーケストレーター（§13.7）|
| RAG | ラグ | Retrieval-Augmented Generation。外部知識を LLM コンテキストに注入する手法（§1.3.9）|
| rtl-conductor | アールティーエル・コンダクター | RTL 設計フローオーケストレーター（HDL Lint / Sim / Formal / Transpile）（§4）|
| RtlToolAdapter | アールティーエル・ツール・アダプター | RTL ツールの統一インターフェースを定義するトレイト（§4.2）|

## S

| 用語 | 読み方 | 定義 |
|------|--------|------|
| SDD | エスディーディー | Spec-Driven Development。仕様書駆動開発（§1.3.1）|
| SkillSystem | スキル・システム | AI エージェントの専門的設計能力をプラグインとして管理する機構（§1.3.2）|
| SKiDL | スキドル | Python ベースの回路記述言語。LLM との親和性が高い（§7.2）|
| sled | スレッド | Rust ネイティブ KV ストア。messages / agent_state / task_queue 等に使用（§18.9）|
| SpecParser | スペック・パーサー | 自然言語仕様書を構造化 DesignSpec に変換するパーサー（§1.3.1）|
| SWD | エスダブリューディー | Serial Wire Debug。ARM のデバッグインターフェース（§10.6）|

## T

| 用語 | 読み方 | 定義 |
|------|--------|------|
| ToolAdapter | ツール・アダプター | ツールの統一インターフェースを定義するトレイトの総称（§1.2 原則1）|
| Tool Use | ツール・ユース | 生成 AI が外部ツールを呼び出しながら反復的に問題を解決するエージェントループ機構（§1.3.8）|
| trace_id | トレース・アイディ | ワークフロー横断の分散トレース ID（§14.1, §19）|

## U

| 用語 | 読み方 | 定義 |
|------|--------|------|
| UpgradeManager | アップグレード・マネージャー | セマンティックバージョニングに基づく段階的ロールアウト（Canary → Staging → Production）を制御（§3.4）|

## V

| 用語 | 読み方 | 定義 |
|------|--------|------|
| ValidatorAgent | バリデーター・エージェント | サンドボックス環境でパッチを検証し信頼度スコアを算出するエージェント（§1.3.7）|
| VendorAdapter | ベンダー・アダプター | ベンダーツールの統一インターフェースを定義するトレイト（§5.2）|
| Vivado | ビバド | AMD（Xilinx）社の FPGA 開発ツール（§5.5）|

## W

| 用語 | 読み方 | 定義 |
|------|--------|------|
| WatcherAgent | ウォッチャー・エージェント | 6時間ごとにベンダーサイトを監視し新バージョンを検出するエージェント（§1.3.7）|
| WorkflowEngine | ワークフロー・エンジン | DAG として定義されたワークフローをトポロジカルソートで実行制御（§3.5）|

## 略語一覧

| 略語 | 正式名称 |
|------|---------|
| ASIC | Application-Specific Integrated Circuit |
| DRC | Design Rule Check |
| EDA | Electronic Design Automation |
| ERC | Electrical Rule Check |
| FPGA | Field-Programmable Gate Array |
| GDSII | Graphic Data System II |
| HDL | Hardware Description Language |
| HLS | High-Level Synthesis |
| IPC | Inter-Process Communication |
| JTAG | Joint Test Action Group |
| LSP | Language Server Protocol |
| LVS | Layout Versus Schematic |
| MCP | Model Context Protocol |
| PCB | Printed Circuit Board |
| PDK | Process Design Kit |
| RAG | Retrieval-Augmented Generation |
| RTL | Register Transfer Level |
| SDC | Synopsys Design Constraints |
| SDD | Spec-Driven Development |
| SWD | Serial Wire Debug |
| UVM | Universal Verification Methodology |
| VCD | Value Change Dump |
| XDC | Xilinx Design Constraints |

---

## 関連ドキュメント

- [architecture_overview.md](architecture_overview.md) — アーキテクチャ概要
- [agent_communication.md](agent_communication.md) — 通信仕様