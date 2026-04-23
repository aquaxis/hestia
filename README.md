# Hestia

**Hardware Engineering Stack for Tool Integration and Automation**

---

## 概要

FPGA・ASIC・PCB の設計開発で並存する複数ベンダーツール（Vivado / Quartus / Efinity / Radiant / OpenLane 2 / KiCad など）を、AI 活用による統一オーケストレーションで一元管理する、持続可能なハードウェア開発環境。5 つの Conductor（fpga / asic / pcb / debug / ai）が領域ごとのツール群を抽象化し、JSON-RPC 2.0 over Unix Socket で協調動作する。

## 特徴

- **5 Conductor による統一オーケストレーション**: fpga-conductor（Vivado / Quartus / Efinity / Yosys+nextpnr / Radiant）、asic-conductor（OpenLane 2 / Yosys / OpenROAD / Magic / Netgen）、pcb-conductor（KiCad / SKiDL / AI 回路図生成 / Freerouting）、debug-conductor（JTAG / SWD / ILA / SignalTap / Reveal / sigrok）、ai-conductor（全 conductor 統括 + エージェント + コンテナ管理 + RAG）の 5 つのデーモンが、ベンダーごとの独自形式・独自 CLI を抽象化し、JSON-RPC 2.0 over Unix Socket の統一プロトコルで協調動作する。
- **AI 駆動開発 11 プラクティス**: 仕様書駆動（SDD）・テスト駆動（TDD）・RAG・オーケストレーション・チームエージェント・思考の記録（CoT）・作業の記録（Action）・プロンプト保存・オブザーバビリティ・デバッガビリティ・フィードバックループを体系的に組み込み、ハードウェア設計における AI 活用を実務プラクティスとして確立する。
- **持続可能アップグレード**: `WatcherAgent` → `ProbeAgent` → `PatcherAgent`（Anthropic Claude + Tool Use）→ `ValidatorAgent` → `HumanReviewGate` のパイプラインで、ベンダーツールの新バージョンを自動検証・自動パッチ適用する。信頼度スコアに基づき Canary / Staging / Production の段階的ロールアウトを制御し、自動ロールバック（300 秒タイムアウト、最大 3 リトライ）を備える。
- **OSS 第一・メーカー非依存**: OSS ツール（Yosys / nextpnr / OpenLane 2 / KiCad / SKiDL / sigrok 等）を第一候補とし、商用ツールは `adapter.toml` の宣言的記述のみで Rust コード変更なしに統合できる。プラグインシステムは Static / Dynamic / Script / Remote の 4 戦略に対応し、特定メーカーへのロックインを排除する。
- **セキュリティ・再現性**: Podman rootless を基盤とし、`--userns=keep-id` / `--network=none` / `--security-opt=no-new-privileges` / SELinux ラベルによる多重隔離を行う。`fpga.lock` / `asic.lock` によるビルドの完全再現とコンテナベースの環境依存排除で再現性を保証し、HDL / ビットストリーム / GDSII の read-only マウントと API キーの OS キーチェーン管理で知的財産を保護する。

## 解決する課題

ハードウェア開発の現場が抱える 5 つの根本課題に対応する:

- 異種ベンダーツール（Vivado / Quartus / Efinity / Radiant / OpenLane 2 / KiCad など）の独自プロジェクト形式・制約記述・CLI に起因するコンテキストスイッチコスト
- 年 1〜2 回のメジャーリリースに伴う累積的な維持管理コスト
- HDL ソース・ビットストリーム・GDSII 等の知的財産の保護
- 仕様書から HDL コード / 回路図への翻訳工数と仕様・実装の乖離
- 検証工程（全工程の 60〜70%）の工数

## 構成

### アーキテクチャ（5 層）

1. **フロントエンド層** — VSCode 拡張 / Tauri IDE / CLI クライアント（fpga-cli / asic-cli / pcb-cli / debug-cli / ai-cli）
2. **メタオーケストレーション層** — ai-conductor + チームエージェント階層 + container-manager
3. **Conductor 層** — fpga / asic / pcb / debug conductor（JSON-RPC 2.0 over Unix Socket で統一通信）
4. **コンテナ実行層** — Podman rootless（対応イメージ 8 種: vivado / quartus / efinity / radiant / oss / openlane / kicad / debug）
5. **共有サービス層** — HDL LSP ブローカー / 波形ビューア（WASM）/ constraint-bridge / IP マネージャ / CI/CD API / ObservabilityLayer

### ディレクトリ配置（概略）

```
.
├── .hestia/            # Hestia プロジェクト本体
│   ├── design/         # 一次情報源（hestia_design.md + 画像 15 枚）
│   ├── doc/            # 派生ドキュメント（全体フロー / ガイド / Conductor 詳細）
│   └── tools/          # 生成ツール（Rust ワークスペース、5 Conductor + SDK）
└── README.md           # 本ファイル
```

配下の詳細は [`./.hestia/README.md`](./.hestia/README.md) を参照。

## 技術スタック

| 層 | 技術 |
|----|------|
| コアデーモン | Rust + tokio |
| UI / AI | TypeScript（VSCode 拡張 / Tauri + React） |
| コンテナ | Podman rootless |
| AI エージェント | TypeScript + Anthropic SDK（Tool Use） |
| RAG | TypeScript + LangChain + Ollama + Chroma/Qdrant |
| 永続化 | sled（KV）+ SQLite |
| ASIC フロー | Python（OpenLane 2 Step-based Execution） |
| PCB 設計 | Python（SKiDL） |

## ドキュメント

- **使い方・構成**: [`./.hestia/README.md`](./.hestia/README.md)
- **全体フロー**: [`./.hestia/doc/hestia_flow.md`](./.hestia/doc/hestia_flow.md)
- **セットアップガイド**: [`./.hestia/doc/hestia_guide.md`](./.hestia/doc/hestia_guide.md)
- **ドキュメントインデックス**: [`./.hestia/doc/README.md`](./.hestia/doc/README.md)
- **超詳細設計仕様書（一次情報源）**: [`./.hestia/design/hestia_design.md`](./.hestia/design/hestia_design.md)

## ライセンス

HESTIA は利用形態に応じて以下の **3 つのうち 1 つ** を選択する **トリプルライセンス方式** で提供される。

| 利用形態 | ライセンス | 料金 | 成果物公開 | サポート |
|---------|----------|------|-----------|---------|
| 非商用利用 | License A: **AGPL-3.0** | 無償 | 義務なし | コミュニティ |
| 商用利用（無償）| License B: **相互主義ライセンス** | 無償 | 公開義務あり | コミュニティ |
| 商用利用（サブスク）| License C: **商用サブスクリプションライセンス** | 国内 100 万円／年<br>国外 USD 10,000／年 | 義務なし | 標準サポート |

- **Copyright (C) 2026 AQUAXIS TECHNOLOGY**
- ライセンス、商用 / 非商用の区分、小規模事業者救済、評価目的の 180 日ルール 等は [`./LICENSE.md`](./LICENSE.md) を参照
