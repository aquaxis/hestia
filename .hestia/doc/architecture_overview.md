# アーキテクチャ概要（Architecture Overview）

**対象領域**: Hestia 全体
**ソース**: 設計仕様書 §1（設計思想・8原則）, §2（5層+9 Conductor アーキテクチャ）, §1.4（技術スタック）, §1.7（実行環境）

---

## 1. 設計思想と根本的課題

ハードウェア開発（FPGA・ASIC・PCB）には複数のベンダーツールが並存している。FPGA だけでも AMD Vivado、Intel Quartus Prime、Efinix Efinity、Lattice Radiant がそれぞれ独自のプロジェクト形式・制約記述・CLI インターフェースを持つ。ASIC では OpenLane 2 / Yosys / OpenROAD / Magic といった OSS ツールチェーンを組み合わせる必要があり、PCB では KiCad / SKiDL / Freerouting が独立して存在する。

これらの異種ツールを扱うプロジェクトでは膨大なコンテキストスイッチコストが発生する。さらに各ツールは年1〜2回のメジャーリリースがあり、バージョンアップのたびにスクリプト・ログパーサー・制約形式の修正が必要になる。Hestia はこれらの課題を AI を活用した統合環境で解決する。

---

## 2. 設計原則（8原則）

### 原則1: 置き換えではなく抽象化

ベンダーツールは FPGA ビットストリーム生成や ASIC GDSII 出力の認定チェーンであり、完全置換は不可能。「統一インターフェースでオーケストレートするレイヤー」を構築する。各ツールの固有機能は VendorAdapter / ToolAdapter トレイトを通じて抽象化し、上位層からは同一の API で操作可能とする。

### 原則2: ゼロ変更での拡張

新しいベンダーツールを追加する際にコアコードを一切変更しない。`adapter.toml` を書くだけでアダプターを追加できる。Script アダプター戦略として、TOML ファイル内にコマンド、ログパースルール、レポート抽出ルールを正規表現で定義する。

### 原則3: 持続可能な維持管理

ツールのバージョンアップへの対応を AI エージェント（WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent）が自動化し、人間の維持管理コストを最小化する。

### 原則4: セキュリティ

ツール実行はコンテナ実行とローカル実行のいずれも選択可能。コンテナ実行時は Podman rootless により非特権実行、`--network=none` でネットワーク隔離、`--security-opt=no-new-privileges` で権限昇格を防止する。

### 原則5: 再現性

fpga.lock / asic.lock によるビルドの完全再現を保証。コンテナ実行ではコンテナイメージのハッシュ固定により、ローカル実行ではツールバージョン・実行パス・環境変数の lock 記録により同一結果を確保。

### 原則6: メーカー非依存

OSS ツールを優先し、プラグインシステムにより任意のベンダーツールを統合可能。特定メーカーへのロックインを排除。

### 原則7: AI 活用

仕様書駆動開発、生成 AI の Tool Use 機能を活用し、設計プロセス全体を AI で支援。

### 原則8: 統一インターフェース

全 conductor 間およびフロントエンド ↔ ai-conductor の通信を agent-cli ネイティブ IPC に統一する。各 conductor 自身が agent-cli プロセスとして起動された AI エージェントであり、フロントエンドも agent-cli の peer として参加する。

---

## 3. 5層 + 9 Conductor アーキテクチャ

```
┌─────────────────────────────────────────────────────────────────────┐
│                        フロントエンド層                               │
│    VSCode 拡張 (TypeScript)  /  Tauri IDE (Rust + React)             │
│    CLI: hestia + {ai,rtl,fpga,asic,pcb,hal,apps,debug,rag}-cli │
└─────────────────────────┬───────────────────────────────────────────┘
                          │ agent-cli IPC (peer "ai")
┌─────────────────────────▼───────────────────────────────────────────┐
│                   メタオーケストレーション層                           │
│    ai-conductor (全 conductor 統括 + 持続可能アップグレード)           │
│    ┌──────────────────────────────────────────────────────────────┐ │
│    │  container-manager (全 conductor のコンテナライフサイクル管理)  │ │
│    └──────────────────────────────────────────────────────────────┘ │
└───┬──────────┬──────────┬──────────┬──────────┬────────────────────┘
    │          │          │          │          │  agent-cli IPC (peer ごと)
┌───▼───┐  ┌──▼───┐  ┌──▼───┐  ┌──▼───┐  ┌──▼───┐
│ fpga  │  │ asic │  │ pcb  │  │debug │  │ rag  │   Conductor 層
│ cond. │  │ cond.│  │ cond.│  │cond. │  │cond. │   (各領域専用オーケストレーター)
│       │  │      │  │      │  │      │  │      │
│Vivado │  │Open  │  │KiCad │  │JTAG  │  │Chroma│
│Quartus│  │Lane 2│  │SKiDL │  │SWD   │  │Qdrant│
│Efinity│  │Yosys │  │AI 設計│  │ILA   │  │Ollama│
│nextpnr│  │Open  │  │Free- │  │Signal│  │Embed │
│Radiant│  │ROAD  │  │routing│  │Tap   │  │ ing  │
│Gowin  │  │Magic │  │      │  │sigrok│  │      │
└───┬───┘  └──┬───┘  └──┬───┘  └──┬───┘  └──┬───┘
    │         │         │         │         │
┌───▼─────────▼─────────▼─────────▼─────────▼──────────────────────────┐
│         ツール実行層 (コンテナ実行 [Podman rootless] / ローカル実行)     │
│    fpga/vivado:2025.2  │  asic/openlane:latest  │  pcb/kicad:latest   │
│    fpga/quartus:25.1   │  asic/magic:latest     │  debug/tools:latest │
│    fpga/efinity:2025.2 │  fpga/oss:latest       │  (debug は local 専用)│
│    fpga/radiant:2024.2 │                        │                     │
└─────────────────────────┬───────────────────────────────────────────-┘
                          │
┌─────────────────────────▼───────────────────────────────────────────┐
│                      共有サービス層（Layer 5）                          │
│    HDL LSP Broker (svls/vhdl_ls/verilog-ams-ls)                      │
│    WASM 波形ビューア (VCD/FST/GHW/EVCD)                              │
│    Constraint Bridge (XDC ⇔ SDC ⇔ PCF ⇔ Efinity XML)                │
│    IP Manager (OSS / VendorProprietary)                              │
│    CI/CD API (GitHub Actions / GitLab CI / Local)                    │
│    Observability (Prometheus + tracing + OpenTelemetry)              │
└─────────────────────────────────────────────────────────────────────┘
```

### 5層の責務

| 層 | 責務 | 主要コンポーネント |
|----|------|------------------|
| フロントエンド層 | ユーザーインタラクション、エディタ統合、CLI 体験 | VSCode 拡張 / Tauri IDE / `hestia` 統合 CLI / 各 conductor 個別 CLI |
| メタオーケストレーション層 | 全 conductor 統括、コンテナライフサイクル管理 | ai-conductor / container-manager |
| Conductor 層 | 各設計領域の専用オーケストレーション、ツール抽象化、ビルドステートマシン | rtl / fpga / asic / pcb / hal / apps / debug / rag conductor |
| ツール実行層 | 各ベンダーツールの実行（コンテナまたはローカル選択可）、再現性保証、セキュリティ境界 | Podman rootless 対応コンテナイメージ 8 種 + ローカルインストール対応 |
| 共有サービス層 | 層横断機能 6 種（LSP / 波形 / 制約変換 / IP / CI/CD / Observability）| HDL LSP Broker / WASM 波形ビューア / Constraint Bridge / IP Manager / CI/CD API / Observability |

---

## 4. 9 Conductor の役割

| Conductor | 役割 | 対象ツール / 機能 | 実行モード |
|-----------|------|------------------|-----------|
| ai-conductor | メタオーケストレーター（全 conductor 統括／人間との唯一の入口） | Skill / Workflow / Spec-Driven / Backend Switching | コンテナ + ローカル |
| rtl-conductor | RTL 設計フローオーケストレーション（HDL Lint / Sim / Formal / Transpile）| Verilator, Verible, iverilog, GHDL, SymbiYosys, cocotb, Chisel/SpinalHSD/Amaranth bridges | コンテナ + ローカル |
| fpga-conductor | FPGA 設計フローオーケストレーション | Vivado, Quartus, Efinity, Radiant, Gowin, Yosys+nextpnr, OSS | コンテナ + ローカル |
| asic-conductor | ASIC 設計フローオーケストレーション（13 ステップ RTL-to-GDSII） | OpenLane 2, Yosys, OpenROAD, OpenSTA, TritonCTS, TritonRoute, Magic, Netgen, KLayout, Ngspice, SymbiYosys | コンテナ + ローカル |
| pcb-conductor | PCB 設計フローオーケストレーション + AI 回路図生成 | KiCad 9, SKiDL, Freerouting, kicad-mcp-python | コンテナ + ローカル |
| hal-conductor | Hardware Abstraction Layer 生成（レジスタマップ / 多言語ドライバ）| peakrdl, peakrdl-rust, ipyxact, csr2regs, cmsis-svd-gen, svd2rust | コンテナ + ローカル |
| apps-conductor | アプリケーションソフトウェア（FW / RTOS / ベアメタル）開発 | arm-gcc, riscv-gcc, cargo-embed, west-zephyr, freertos-builder, embassy-builder, qemu-system, probe-rs | コンテナ + ローカル |
| debug-conductor | デバッグ環境オーケストレーション | OpenOCD/pyOCD/JTAG/SWD, ILA/SignalTap/Reveal, sigrok/PulseView, WASM 波形ビューア | **ローカル専用** |
| rag-conductor | 知識基盤の構築・管理・検索（ai-conductor から分離） | Chroma/Qdrant, Ollama (nomic-embed-text), PyPDF/pdfplumber, Tesseract OCR, Camelot, trafilatura | コンテナ + ローカル |

### Conductor 共通アーキテクチャパターン

各 conductor は同一アーキテクチャパターンを踏襲する:

- Rust ワークスペース構成（`.hestia/tools/` 配下、`Cargo.toml` resolver = 2）
- ToolAdapter / VendorAdapter による抽象化
- Capability ベースのアダプター登録・解決エンジン（AdapterRegistry）
- adapter.toml 宣言方式による拡張（Rust コード変更不要）
- Podman rootless コンテナ統合（debug-conductor を除く）
- agent-cli ネイティブ IPC による通信
- CLI クライアントバイナリ（`hestia-{conductor}-cli`）
- conductor-sdk / adapter-core 等の共通クレート利用

---

## 5. 技術スタック

| 層 | 技術 | 選定理由 |
|---|---|---|
| コアデーモン | Rust | メモリ安全・非同期処理（tokio）・クロスプラットフォームバイナリ・高速実行 |
| フロントエンド | TypeScript (VSCode 拡張 / Tauri) | エコシステム成熟・Monaco Editor 統合・デスクトップアプリ対応 |
| コンテナ | Podman (rootless) | デーモンレス・rootless で非特権実行・SELinux 対応 |
| AI エージェント | TypeScript + Anthropic SDK | Claude Sonnet の Tool Use 機能によるエージェントループ |
| 永続化 | sled (KV) + SQLite | Rust ネイティブ・軽量・組み込み可能・互換性マトリクス DB |
| ASIC フロー | Python (OpenLane 2) | OpenLane 2 の Python ベース Step-based Execution |
| PCB 設計 | Python (SKiDL) | LLM との親和性が高い回路記述言語 |
| PCB AI | TypeScript + LangChain | LLM フレームワークによる回路図合成 |

---

## 6. 実行環境

HESTIA は **Linux** を実行環境とする。

| 区分 | 要件 |
|------|------|
| ホスト OS | Linux（x86_64 カーネル 5.x 以降を推奨）|
| 推奨ディストリビューション | Ubuntu 22.04 LTS 以降 / RHEL 8 以降 / Debian 12 以降 |
| 必須カーネル機能 | user namespace（rootless Podman 用）/ cgroup v2 / SELinux or AppArmor / Unix Domain Socket |
| 非対応 OS | Windows / macOS（ホスト OS としてはサポート対象外）|
| 開発環境（補助的許容）| Windows + WSL2 は開発補助として利用可能。ただし CI / 本番は Linux ネイティブ |

Linux を前提とする具体的な依存要素:

- **コンテナランタイム**: Podman rootless は Linux の user namespace / cgroup / SELinux に依存
- **IPC**: agent-cli ネイティブ IPC は POSIX/Linux のプリミティブ
- **セキュリティ**: SELinux label は Linux Security Module の機能
- **非同期ランタイム**: tokio は Linux epoll を主要バックエンドとして動作
- **コンテナイメージ**: 8 種すべてが Linux ベース

---

## 関連ドキュメント

- [glossary.md](glossary.md) — 用語集
- [agent_communication.md](agent_communication.md) — 通信仕様
- [security.md](security.md) — セキュリティ方針
- [shared_services.md](shared_services.md) — 共有サービス層